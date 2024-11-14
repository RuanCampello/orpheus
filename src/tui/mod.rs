mod colours;
mod components;
mod keyboard;

use crate::spotify::Client;
use components::playlist::draw_playlists_section;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self,Event};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Block, BorderType, ListState};
use ratatui::{Frame, Terminal};
use rspotify::model::playlist::SimplifiedPlaylist;
use rspotify::model::user::PrivateUser;
use std::io;
use std::io::Stdout;
use std::time::{Duration, Instant};

/// Interface that reflects and calls the client in order to generate the UI.
pub(super) struct State {
    client: Client,
    user: PrivateUser,
    pub playlist_state: PlaylistState,
    should_quit: bool,
    // _marker: PhantomData<&'a ()>,
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
            active: true,
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
            should_quit: false,
        }
    }

    pub fn run(
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
                    self.on_key(key.code)
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
}

fn draw(frame: &mut Frame, state: &mut State) {
    let [title_area, remaining_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(frame.area());

    let block = Block::bordered()
        .title("Orpheus")
        .border_type(BorderType::Rounded);

    let [playlist_area, main_area, queue_area] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Min(0),
        Constraint::Percentage(25),
    ])
    .areas(remaining_area);

    frame.render_widget(block, title_area);
    draw_playlists_section(frame, state, playlist_area);
}
