mod player;
pub(super) mod playlist;
pub(super) mod search;

use crate::internal::config::Config;
use crate::internal::Client;
use crate::tui::draw;
use crate::tui::state::player::PlayerState;
use crate::tui::state::playlist::PlaylistState;
use crate::tui::state::search::{ResultItem, SearchState};
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event};
use ratatui::Terminal;
use rspotify::model::device::Device;
use rspotify::model::search::SearchResult;
use rspotify::senum::SearchType;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

/// Defines the page that should be rendered in main area.
#[derive(PartialEq, Debug, Default)]
pub enum Tab {
    #[default]
    Home,
    SearchResults,
    PlaylistPage,
}

#[derive(PartialEq)]
pub(super) enum VolumeAction {
    Increase,
    Decrease,
}

/// Interface that reflects and calls the client in order to generate the UI.
pub(crate) struct State {
    pub tab: Tab,

    pub client: Client,
    config: Config,

    pub(in crate::tui) playlist_state: PlaylistState,
    pub(in crate::tui) device: Device,

    pub(super) search_state: SearchState,
    pub(super) should_quit: bool,
    pub(super) player: PlayerState,

    pub(in crate::tui) window: WindowSize,
}

pub(in crate::tui) struct WindowSize {
    pub height: u16,
    pub width: u16,
}

macro_rules! create_search_future {
    ($client:expr, $query:expr, $_type:expr) => {
        $client.spotify.search($query, $_type, 20, 0, None, None)
    };
}

impl State {
    pub async fn new(client: Client, config: Config) -> Self {
        let playlists_future = client.spotify.current_user_playlists(50, 0);
        let device_future = client.spotify.device();

        let (playlists, devices) = tokio::join!(playlists_future, device_future);
        let device: Option<Device> = match devices {
            Ok(payload) => payload.devices.into_iter().next(),
            Err(_) => None,
        };

        let playlist_state = match playlists {
            Ok(page) => PlaylistState::new(page.items),
            Err(_) => PlaylistState::new(Vec::new()),
        };

        Self {
            client,
            config,
            device: device.expect("Failed to get your device"),
            playlist_state,
            tab: Tab::default(),
            player: PlayerState::new(),
            search_state: SearchState::new(),
            should_quit: false,
            window: WindowSize {
                height: 43,
                width: 230,
            },
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        tick_rate: Duration,
    ) -> io::Result<()> {
        let mut last_tick = Instant::now();
        let mut last_state_update = Instant::now();

        // fetches the currently playing state on the launch.
        self.update_playing_state().await;

        // updates the window size on first launch.
        let size = terminal.size()?;
        self.handle_resize(size.width, size.height);

        // tries to set the first playlist as selected on launch.
        if let Some(first_playlist) = self.playlist_state.as_ref().playlists.first() {
            let uri = first_playlist.uri.to_string();
            self.select_playlist(uri).await;
        }

        loop {
            terminal.draw(|frame| draw(frame, self))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                match event::read()? {
                    Event::Resize(x, y) => self.handle_resize(x, y),
                    Event::Key(key) => self.handle_key(key.code).await,
                    _ => continue,
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }

            if last_state_update.elapsed() >= Duration::from_secs(5) {
                self.get_playing_state().await;
                last_state_update = Instant::now();
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }

    /// Fetch a given playlist uri and updates the respective states.
    pub(super) async fn select_playlist(&mut self, uri: String) {
        let Ok(playlist) = self.client.spotify.playlist(&uri, None, None).await else {
            return;
        };

        self.tab = Tab::PlaylistPage;
        self.playlist_state.active = false;
        self.playlist_state.selected_playlist.state.max_size = playlist.tracks.items.len();
        self.playlist_state.selected_playlist.state.active = true;
        self.playlist_state.selected_playlist.playlist = Some(playlist);
    }

    /// Tries to update the currently playing state every 5 seconds.
    pub(super) async fn get_playing_state(&mut self) {
        if let Ok(playing) = self.client.spotify.current_user_playing_track().await {
            let image_url = playing
                .as_ref()
                .and_then(|playing| {
                    playing
                        .item
                        .as_ref()
                        .and_then(|item| item.album.images.first())
                        .map(|image| image.url.as_ref())
                })
                .unwrap_or("default_image_url");

            self.player
                .update_current_image(image_url, self.window.height, self.window.width);

            self.player.playing = playing;
        }
    }

    /// Manual currently playing update.
    pub(super) async fn update_playing_state(&mut self) {
        self.get_playing_state().await;
    }

    pub(super) async fn search(&mut self) {
        let query = self.search_state.input.as_str();
        let tracks_future = create_search_future!(self.client, query, SearchType::Track);
        let artists_future = create_search_future!(self.client, query, SearchType::Artist);
        // let albums_future = create_search_future!(self.client, query, SearchType::Album);

        #[allow(clippy::single_match)]
        match tokio::try_join!(tracks_future, artists_future) {
            Ok((
                SearchResult::Tracks(tracks),
                SearchResult::Artists(artists),
                // SearchResult::Albums(albums),
            )) => {
                self.search_state.results.songs = ResultItem::new(tracks);
                self.search_state.results.artists = ResultItem::new(artists);
                // self.search_state.results.albums = Some(ResultItem::new(albums));
            }
            _ => {}
        };
    }

    /// Tries to play the currently selected track in the search results.
    pub(super) async fn play_selected_track(&mut self, uri: Option<String>) {
        // TODO: why does when using ctx+device_id not working?

        let device_id = self.config.device_id.take();
        if self
            .client
            .spotify
            .start_playback(
                device_id,
                None,
                Some(vec![uri.unwrap_or_default()]),
                None,
                None,
            )
            .await
            .is_ok()
        {
            self.update_playing_state().await
        };
    }

    pub(super) async fn update_volume(&mut self, action: VolumeAction) {
        let mut volume = self.device.volume_percent;
        match action {
            VolumeAction::Decrease => volume = volume.saturating_sub(5),
            VolumeAction::Increase => volume = volume.saturating_add(5).min(100),
        }

        if self
            .client
            .spotify
            .volume(volume as u8, Some(self.device.id.to_string()))
            .await
            .is_ok()
        {
            self.device.volume_percent = volume;
        }
    }

    fn handle_resize(&mut self, x: u16, y: u16) {
        if self.window.height != y {
            self.window.height = y;
            self.playlist_state.offset_step = self.window.height.saturating_sub(8) as u32;
        }
        self.window.width = x;
    }
}
