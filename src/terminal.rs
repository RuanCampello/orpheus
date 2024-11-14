use crate::spotify::Client;
use crate::tui::state::State;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
};
use ratatui::Terminal;
use std::io;
use std::time::Duration;

/// Starts the native terminal.
pub async fn run(tick_rate: Duration, client: Client) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = State::new(client).await;
    let app_res = app.run(&mut terminal, tick_rate).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = app_res {
        eprintln!("Application encountered an error: {:?}", err);
    }

    Ok(())
}
