use crate::tui::colours::Palette;
use crate::tui::components::pad;
use crate::tui::State;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, List, ListItem};
use ratatui::{text, Frame};

pub(in crate::tui) fn draw_playlists_section<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    let playlists_block = Block::bordered().border_type(BorderType::Rounded).title(pad("Playlist", 2));

    let playlists: Vec<ListItem> = state
        .playlist_state
        .playlists
        .iter()
        .map(|i| ListItem::new(vec![text::Line::from(Span::raw(&i.name))]))
        .collect();

    let playlists = List::new(playlists).block(playlists_block).highlight_style(Style::new().bg(Palette::Secondary.into()));

    frame.render_stateful_widget(playlists, area, &mut state.playlist_state.state);
}