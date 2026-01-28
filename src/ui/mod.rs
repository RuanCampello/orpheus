//! UI components to be rendered from [state](crate::state::State).

use std::{thread::sleep, time::Duration};

use crate::{config::Palette, state::State};
use ratatui::{
    Frame,
    layout::{Constraint, HorizontalAlignment, Layout, Rect},
    style::Style,
    text::{self, Span},
    widgets::{Block, BorderType, List, ListItem, Padding},
};

#[inline(always)]
fn pad(content: &str, size: usize) -> String {
    let mut out = String::with_capacity(content.len() + size * 2);
    out.extend(std::iter::repeat(' ').take(size));
    out.push_str(content);
    out.extend(std::iter::repeat(' ').take(size));
    out
}

pub(crate) fn draw(frame: &mut Frame, state: &State) {
    let palette = Palette::from(&state.config.theme);

    frame.render_widget(
        Block::default().style(Style::default().bg(palette.background).fg(palette.muted)),
        frame.area(),
    );

    let [header, bottom] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(frame.area());

    let [playlist, main, player] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Min(0),
        Constraint::Length(0),
    ])
    .areas(bottom);

    draw_playlist_sidebar(frame, state, palette, playlist);
}

fn draw_playlist_sidebar(frame: &mut Frame, state: &State, palette: Palette, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Plain)
        .padding(Padding::horizontal(1))
        .title_alignment(HorizontalAlignment::Center)
        .title(pad("Playlists", 1))
        .border_style(Style::new().fg(palette.muted));

    if let Some(playlists) = &state.playlists {
        let items = playlists
            .items
            .iter()
            .map(|playlist| ListItem::new(vec![text::Line::from(Span::raw(&playlist.name))]))
            .collect::<Vec<_>>();

        let playlists = List::new(items).block(block);
        frame.render_widget(playlists, area);
    }
}
