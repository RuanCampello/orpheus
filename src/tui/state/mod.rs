mod player;
pub mod playlist;
pub(super) mod search;

use crate::internal::config::Config;
use crate::internal::Client;
use crate::tui::draw;
use crate::tui::state::player::PlayerState;
use crate::tui::state::playlist::{Playable, SelectedPlaylist};
use crate::tui::state::search::{ResultItem, SearchState, TableStateExt};
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event};
use ratatui::widgets::ListState;
use ratatui::Terminal;
use rspotify::model::playlist::SimplifiedPlaylist;
use rspotify::model::search::SearchResult;
use rspotify::model::user::PrivateUser;
use rspotify::senum::SearchType;
use std::io;
use std::io::Stdout;
use std::time::{Duration, Instant};

/// Interface that reflects and calls the client in order to generate the UI.
pub(crate) struct State {
    pub client: Client,
    user: PrivateUser,
    config: Config,

    pub(super) playlist_state: PlaylistState,
    pub(super) search_state: SearchState,
    pub(super) should_quit: bool,
    pub(super) player: PlayerState,

    pub(in crate::tui) window: WindowSize,
}

pub(in crate::tui) struct WindowSize {
    pub height: u16,
    pub width: u16,
}

pub(super) struct PlaylistState {
    pub playlists: Vec<SimplifiedPlaylist>,
    pub selected_playlist: SelectedPlaylist,
    pub offset: u32,
    pub state: ListState,
    pub active: bool,
}

impl State {
    pub async fn new(client: Client, config: Config) -> Self {
        let user_future = client.spotify.current_user();
        let playlists_future = client.spotify.current_user_playlists(50, 0);

        let (user, playlists) = tokio::join!(user_future, playlists_future);

        let user = user.expect("Current user not found");

        let playlist_state = match playlists {
            Ok(page) => PlaylistState::new(page.items),
            Err(_) => PlaylistState::new(Vec::new()),
        };

        Self {
            client,
            config,
            user,
            playlist_state,
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
        self.window.width = size.width;
        self.window.height = size.height;

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
        let tracks_future = self
            .client
            .spotify
            .search(query, SearchType::Track, 20, 0, None, None);
        let artists_future =
            self.client
                .spotify
                .search(query, SearchType::Artist, 20, 0, None, None);
        let albums_future = self
            .client
            .spotify
            .search(query, SearchType::Album, 20, 0, None, None);

        #[allow(clippy::single_match)]
        match tokio::try_join!(tracks_future, artists_future, albums_future) {
            Ok((
                SearchResult::Tracks(tracks),
                SearchResult::Artists(artists),
                SearchResult::Albums(albums),
            )) => {
                self.search_state.results.songs = Some(ResultItem {
                    table_state: TableStateExt::new(tracks.items.len()),
                    data: tracks,
                });
                self.search_state.results.artists = Some(ResultItem {
                    table_state: TableStateExt::new(artists.items.len()),
                    data: artists,
                });
                self.search_state.results.albums = Some(ResultItem {
                    table_state: TableStateExt::new(albums.items.len()),
                    data: albums,
                });
            }
            _ => {}
        };
    }

    /// Tries to play the currently selected track in the search results.
    pub(super) async fn play_selected_track(&mut self, uri: Option<String>) {
        let track_uri = uri.map(|uri| uri.to_string()).unwrap_or_default();

        // TODO: why does when using ctx+device_id not working?

        let device_id = self.config.device_id.take();
        if self
            .client
            .spotify
            .start_playback(device_id, None, Some(vec![track_uri]), None, None)
            .await
            .is_ok()
        {
            self.update_playing_state().await
        };
    }

    fn handle_resize(&mut self, x: u16, y: u16) {
        self.window.height = y;
        self.window.width = x;
    }
}
