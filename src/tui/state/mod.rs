pub(super) mod player;
pub(super) mod playlist;
pub(super) mod search;

use crate::internal::config::{Config, ImageKind};
use crate::internal::image::{colour_from_image, Rgb};
use crate::internal::Client;
use crate::tui::draw;
use crate::tui::state::player::{AsTrack, LyricState, PlayerState};
use crate::tui::state::playlist::PlaylistState;
use crate::tui::state::search::{ResultItem, SearchState};
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event};
use ratatui::Terminal;
use rspotify::model::context::CurrentlyPlaybackContext;
use rspotify::model::device::Device;
use rspotify::model::offset::for_position;
use rspotify::model::search::SearchResult;
use rspotify::model::PlayingItem;
use rspotify::senum::SearchType;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

/// Defines the page that should be rendered in the main area.
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

/// Interface that reflects and calls the client to generate the UI.
pub(crate) struct State {
    config: Config,

    pub tab: Tab,
    pub client: Client,

    pub(in crate::tui) playlist_state: PlaylistState,
    pub(in crate::tui) device: Device,
    pub(in crate::tui) window: WindowSize,
    pub(in crate::tui) lyrics_state: LyricState,

    pub(super) should_quit: bool,
    pub(super) colour: Rgb,
    pub(super) search_state: SearchState,
    pub(super) player: PlayerState,
}

#[derive(Debug, Default, PartialEq)]
pub(in crate::tui) struct WindowSize {
    pub height: u16,
    pub width: u16,
}

pub(in crate::tui) struct PlayingInfo {
    playing: Option<CurrentlyPlaybackContext>,
    colour: Rgb,
    image_url: String,
}

macro_rules! create_search_future {
    ($client:expr, $query:expr, $_type:expr) => {
        $client.spotify.search($query, $_type, 20, 0, None, None)
    };
}

impl State {
    pub async fn new(client: Client, config: Config) -> Self {
        let playlists = client.spotify.current_user_playlists(50, 0);
        let devices = client.spotify.device();
        let (playlists, devices) = tokio::join!(playlists, devices);

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
            playlist_state,
            colour: Rgb::default(),
            device: device.expect("Failed to get your device"),
            tab: Tab::default(),
            lyrics_state: LyricState::default(),
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

        self.update_playing_state().await;

        // updates the window size on the first launch.
        let size = terminal.size()?;
        self.handle_resize(size.width, size.height);

        self.playlist_state.sort_playlists();

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
                    Event::Mouse(mouse) => self.lyrics_state.handle_scroll(&mouse),
                    _ => continue,
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }

            let elapsed = last_state_update.elapsed();
            if elapsed >= Duration::from_millis(100) {
                self.player.tick_progress(elapsed.as_millis() as _);
                last_state_update = Instant::now();
            }

            if self.should_quit {
                return Ok(());
            }
        }
    }

    /// Fetch a given playlist uri and updates the respective states.
    pub(super) async fn select_playlist(&mut self, uri: String) {
        // ignore if the user re-selects the current selected playlist
        if let Some(playlist) = self.playlist_state.selected_playlist.playlist.as_ref() {
            if playlist.uri == uri {
                self.playlist_page_selected();

                return;
            }
        }

        let Ok(playlist) = self.client.spotify.playlist(&uri, None, None).await else {
            return;
        };

        self.playlist_page_selected();
        self.playlist_state.offset = 0;
        self.playlist_state.selected_playlist.state.max_size = playlist.tracks.items.len();
        self.playlist_state.selected_playlist.playlist = Some(playlist);
    }

    /// Tries to update the currently playing state.
    pub(super) async fn get_playing_state(&self) -> PlayingInfo {
        let playing = self
            .client
            .spotify
            .current_playback(None, None)
            .await
            .ok()
            .flatten();

        let image_url = playing
            .as_ref()
            .and_then(|p| p.item.as_ref())
            .and_then(|item| match item {
                PlayingItem::Track(track) => {
                    track.album.images.first().map(|img| img.url.to_string())
                }
                _ => None,
            })
            .unwrap_or_default();

        let colour = colour_from_image(&image_url).await.unwrap_or_default();

        PlayingInfo {
            image_url,
            colour,
            playing,
        }
    }

    pub(super) async fn search(&mut self) {
        let query = self.search_state.input.as_str();
        let tracks_future = create_search_future!(self.client, query, SearchType::Track);
        let artists_future = create_search_future!(self.client, query, SearchType::Artist);
        let albums_future = create_search_future!(self.client, query, SearchType::Album);

        #[allow(clippy::single_match)]
        match tokio::try_join!(tracks_future, artists_future, albums_future) {
            Ok((
                SearchResult::Tracks(tracks),
                SearchResult::Artists(artists),
                SearchResult::Albums(albums),
            )) => {
                self.search_state.results.songs = ResultItem::new(tracks);
                self.search_state.results.artists = ResultItem::new(artists);
                self.search_state.results.albums = ResultItem::new(albums);
            }
            _ => {}
        };
    }

    /// Tries to play the currently selected track in the search results.
    pub(super) async fn play_selected_track(
        &mut self,
        uri: Option<String>,
        offset: Option<usize>,
        identifier: Option<String>,
    ) {
        let device_id = self.config.device_id.take();
        let context_uri = self
            .player
            .playing
            .as_ref()
            .and_then(|playing| playing.context.as_ref().map(|ctx| ctx.uri.to_string()));
        let uris = Some(vec![uri.unwrap_or_default()]);
        let is_from_new_ctx = context_uri.ne(&identifier);

        let (uris, context_uri, offset) = if context_uri.is_some() && !is_from_new_ctx {
            (None, context_uri, offset)
        } else if self.tab.eq(&Tab::SearchResults) {
            (uris, None, None)
        } else {
            let new_context = self
                .playlist_state
                .selected_playlist
                .playlist
                .as_ref()
                .map(|playlist| playlist.uri.to_string());

            (None, new_context, offset)
        };

        let offset = offset.and_then(|o| for_position(o as u32));

        if self
            .client
            .spotify
            .start_playback(device_id, context_uri, uris, offset, None)
            .await
            .is_ok()
        {
            self.update_playing_state().await;
        };
    }

    pub(super) async fn toggle_playing_state(&mut self) {
        let Some(playing) = &self.player.playing else {
            return;
        };
        let device_id = Some(self.device.id.to_string());

        let result = match playing.is_playing {
            true => self.client.spotify.pause_playback(device_id).await,
            false => {
                self.client
                    .spotify
                    .start_playback(device_id, None, None, None, playing.progress_ms)
                    .await
            }
        };

        self.update_playing_state().await;
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

    pub(super) async fn get_current_song_lyrics(&self) -> (String, bool) {
        let (content, is_synced) = match async {
            let song = self.player.playing.as_ref()?;
            let artist = self.player.get_artist_name()?;
            let name = &song.item.as_ref()?.as_track()?.name;
            self.client.lyra.get_song_lyrics(artist, name).await.ok()
        }
        .await
        {
            Some(tuple) => tuple,
            None => (
                "Looks like we couldn't find the lyrics for this song.".into(),
                false,
            ),
        };

        (content, is_synced)
    }

    pub(super) fn get_player_image_kind(&self) -> &ImageKind {
        &self.config.player_image_kind
    }

    /// Fetches and updates the [currently playing state](PlayerState) and [lyrics](LyricState).
    async fn update_playing_state(&mut self) {
        let playing_info = self.get_playing_state().await;
        let update_colour = |colour| self.colour = colour;
        let image_kind = &self.config.player_image_kind;
        let window = &self.window;

        // TODO: we may find a way to do that it in parallel if one doesn't need the other
        self.player
            .apply_playing_info(image_kind, playing_info, window, update_colour)
            .await;

        let (content, is_synced) = self.get_current_song_lyrics().await;
        self.lyrics_state.update(content, is_synced);
    }

    /// Changes the necessary states to reflect a playlist being selected.
    fn playlist_page_selected(&mut self) {
        self.tab = Tab::PlaylistPage;
        self.playlist_state.selected_playlist.state.active = true;
        self.playlist_state.active = false;
    }

    fn handle_resize(&mut self, x: u16, y: u16) {
        if self.window.height != y {
            self.window.height = y;
            self.playlist_state.offset_step = self.window.height.saturating_sub(8) as u32;
        }
        self.window.width = x;
    }
}
