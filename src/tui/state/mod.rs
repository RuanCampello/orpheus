pub(super) mod search;

use crate::spotify::Client;
use crate::tui::draw;
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
    client: Client,
    user: PrivateUser,
    pub(super) playlist_state: PlaylistState,
    pub(super) search_state: SearchState,
    pub(super) should_quit: bool,
}

pub(super) struct PlaylistState {
    pub playlists: Vec<SimplifiedPlaylist>,
    pub state: ListState,
    pub active: bool,
}

impl PlaylistState {
    fn new(playlists: Vec<SimplifiedPlaylist>) -> Self {
        let state = ListState::default().with_selected(Some(0));
        Self {
            playlists,
            state,
            active: false,
        }
    }
}

impl State {
    pub async fn new(client: Client) -> Self {
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
            user,
            playlist_state,
            search_state: SearchState::new(),
            should_quit: false,
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        tick_rate: Duration,
    ) -> io::Result<()> {
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|frame| draw(frame, self))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.on_key(key.code).await;
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }

            if self.should_quit {
                return Ok(());
            }
        }
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
}
