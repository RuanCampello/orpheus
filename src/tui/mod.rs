use crate::internal::debug;
use crate::tui::components::player::{draw_lyrics, draw_player};
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
        Layout::horizontal([Constraint::Min(0), Constraint::Length(23)]).areas(header_area);

    let [playlist_area, main_area, player_area] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Min(0),
        match state.player.playing {
            Some(_) => Constraint::Percentage(25),
            None => Constraint::Length(0),
        },
    ])
    .areas(remaining_area);

    draw_search_input(frame, state, search_area);
    draw_volume_widget(frame, state, volume_area);

    let [main_area, lyrics_area] = if state.lyrics_state.active {
        Layout::horizontal([Constraint::Percentage(65), Constraint::Percentage(35)])
            .areas(main_area)
    } else {
        Layout::horizontal([Constraint::Min(0), Constraint::Length(0)]).areas(main_area)
    };

    #[allow(clippy::op_ref)]
    match state.tab {
        Tab::SearchResults => draw_search_results(frame, state, main_area),
        Tab::PlaylistPage => draw_playlist_screen(
            frame,
            &mut state.playlist_state.selected_playlist,
            state.playlist_state.offset,
            state.playlist_state.offset_step,
            &state.colour,
            main_area,
        ),
        Tab::Home => {}
    }

    draw_playlists_sidebar(frame, state, playlist_area);
    draw_player(frame, state, player_area);
    if let Some(ctx) = &state.player.playing.as_ref() {
        if state.lyrics_state.active {
            draw_lyrics(
                frame,
                &mut state.lyrics_state,
                state.colour,
                ctx.progress_ms.unwrap_or_default(),
                lyrics_area,
            )
        }
    }
}
