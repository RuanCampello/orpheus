use crate::spotify::Client;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType};
use ratatui::{Frame, Terminal};
use rspotify::model::user::PrivateUser;
use std::io;
use std::io::Stdout;
use std::time::{Duration, Instant};

/// Interface that reflects and calls the client in order to generate the UI.
pub(super) struct State {
    client: Client,
    user: PrivateUser,
    // search_state: SearchState<'a>
    should_quit: bool,
    // _marker: PhantomData<&'a ()>,
}

// pub struct SearchState<'a> {}

impl State {
    pub async fn new(client: Client) -> Self {
        let user = client.spotify.current_user().await.expect("Current user not found");

        Self { client, user, should_quit: false }
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
    let username = state.user.display_name.as_deref().unwrap_or("Some");

    let block = Block::bordered()
        .title(username)
        .border_style(Style::new().fg(Color::Red))
        .border_type(BorderType::Rounded);

    frame.render_widget(block, title_area);
}