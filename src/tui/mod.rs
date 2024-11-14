use crate::tui::components::playlist::draw_playlists_section;
use crate::tui::components::search::draw_search_input;
use crate::tui::state::State;
use ratatui::layout::{Constraint, Layout};
use ratatui::Frame;

mod colours;
mod components;
mod keyboard;
pub mod state;

fn draw(frame: &mut Frame, state: &mut State) {
    let [search, remaining_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(frame.area());

    let [playlist_area, main_area, queue_area] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Min(0),
        Constraint::Percentage(25),
    ])
    .areas(remaining_area);

    draw_search_input(frame, state, search);
    draw_playlists_section(frame, state, playlist_area);
}
