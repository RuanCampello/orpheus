mod components;
mod colours;

use crate::spotify::Client;
use components::playlist::draw_playlists_section;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
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
}

impl PlaylistState {
    fn new(playlists: Vec<SimplifiedPlaylist>) -> Self {
        let state = ListState::default().with_selected(Some(0));
        Self {
            playlists,
            state,
        }
    }

    pub fn next(&mut self) {
        let i = self.state.selected().unwrap_or(usize::MAX).saturating_add(1) % self.playlists.len();
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = self.state.selected().unwrap_or(0).saturating_sub(1) % self.playlists.len();
        self.state.select(Some(i));
    }
}

// pub struct SearchState<'a> {}

impl State {
    pub async fn new(client: Client) -> Self {
        let user = client.spotify.current_user().await.expect("Current user not found");
        let playlists = client.spotify.current_user_playlists(50, 0).await.expect("Playlists not found");
        let playlist_state = PlaylistState::new(playlists.items);

        Self { client, user, playlist_state, should_quit: false }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>, tick_rate: Duration) -> io::Result<()> {
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|frame| draw(frame, self))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        self.should_quit = true;
                    }
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
    let [title_area, remaining_area] = Layout::vertical([Constraint::Length(5), Constraint::Min(0)]).areas(frame.area());

    let block = Block::bordered()
        .title("Orpheus")
        .border_style(Style::new().fg(Color::Red))
        .border_type(BorderType::Rounded);

    frame.render_widget(block, title_area);
    draw_playlists_section(frame, state, remaining_area);
}