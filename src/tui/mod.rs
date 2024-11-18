use crate::tui::components::player::draw_player;
use crate::tui::components::playlist::{draw_playlist_screen, draw_playlists_sidebar};
use crate::tui::components::search::{draw_search_input, draw_search_results};
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

    match &state.playlist_state.selected_playlist {
        None => draw_search_results(frame, state, main_area),
        Some(playlist) => draw_playlist_screen(frame, playlist, main_area),
    }

    draw_playlists_sidebar(frame, state, playlist_area);
    draw_player(frame, state, queue_area);
}
