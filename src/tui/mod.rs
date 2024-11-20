use crate::tui::components::player::draw_player;
use crate::tui::components::playlist::{draw_playlist_screen, draw_playlists_sidebar};
use crate::tui::components::search::{draw_search_input, draw_search_results};
use crate::tui::components::volume::draw_volume_widget;
use crate::tui::state::{State, Tab};
use ratatui::layout::{Constraint, Layout};
use ratatui::Frame;

mod colours;
mod components;
mod keyboard;
pub mod state;

fn draw(frame: &mut Frame, state: &mut State) {
    let [header_area, remaining_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(frame.area());
    let [search_area, volume_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Percentage(12)]).areas(header_area);

    let [playlist_area, main_area, queue_area] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Min(0),
        Constraint::Percentage(25),
    ])
    .areas(remaining_area);

    draw_search_input(frame, state, search_area);
    draw_volume_widget(frame, state, volume_area);

    #[allow(clippy::op_ref)]
    match state.tab {
        Tab::SearchResults => draw_search_results(frame, state, main_area),
        Tab::PlaylistPage => draw_playlist_screen(
            frame,
            &mut state.playlist_state.selected_playlist,
            state.playlist_state.offset,
            state.playlist_state.offset_step,
            main_area,
        ),
        Tab::Home => {}
    }

    draw_playlists_sidebar(frame, state, playlist_area);
    draw_player(frame, state, queue_area);
}
