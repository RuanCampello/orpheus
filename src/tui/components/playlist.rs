use crate::internal::image::Rgb;
use crate::tui::components::search::draw_results_table;
use crate::tui::components::{pad, time_from_ms, BlockExt, ListExt, ToRow};
use crate::tui::state::playlist::SelectedPlaylist;
use crate::tui::State;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, List, ListItem, Row};
use ratatui::{text, Frame};
use rspotify::model::playlist::PlaylistTrack;

impl<'a> ToRow<'a> for &'a PlaylistTrack {
    fn to_row(&self, idx: usize) -> Row<'a> {
        let track = self.track.as_ref().unwrap();
        let album = &track.album.name;
        let time = time_from_ms(&track.duration_ms);

        let artists = self
            .track
            .as_ref()
            .unwrap()
            .artists
            .iter()
            .take(3)
            .map(|artist| artist.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        Row::new(vec![
            idx.to_string(),
            track.name.to_string(),
            artists,
            album.to_string(),
            time,
        ])
    }
}

pub fn draw_playlists_sidebar<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    let playlists_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(pad("Playlist", 2))
        .bordered_section(&state.colour, state.playlist_state.active);

    let playlists: Vec<ListItem> = state
        .playlist_state
        .playlists
        .iter()
        .map(|i| ListItem::new(vec![text::Line::from(Span::raw(&i.name))]))
        .collect();

    let playlists = List::new(playlists)
        .block(playlists_block)
        .highlightable_section(&state.colour, state.playlist_state.active);

    frame.render_stateful_widget(playlists, area, &mut state.playlist_state.state);
}

pub fn draw_playlist_screen<'a>(
    frame: &'a mut Frame,
    selected_playlist: &'a mut SelectedPlaylist,
    offset: u32,
    offset_step: u32,
    active_colour: &'a Rgb,
    area: Rect,
) {
    const WIDTHS: &[Constraint] = &[
        Constraint::Length(5),
        Constraint::Percentage(40),
        Constraint::Percentage(25),
        Constraint::Min(5),
        Constraint::Length(5),
    ];
    const HEADERS: &[&str; 5] = &["#", "Title", "Artist", "Album", "Time"];
    let playlist = selected_playlist.playlist.as_ref().unwrap();

    let playlist_table = draw_results_table(
        &playlist
            .tracks
            .items
            .iter()
            .take(offset_step as usize)
            .collect::<Vec<&PlaylistTrack>>(),
        &playlist.name,
        WIDTHS,
        &selected_playlist.state,
        active_colour,
        HEADERS,
        Some(offset),
    );

    frame.render_stateful_widget(playlist_table, area, &mut selected_playlist.state.state);
}
