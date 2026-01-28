//! UI components to be rendered from [state](crate::state::State).

#![allow(unused_variables)]

mod playlist;
pub(crate) mod style;

use crate::{
    state::{State, handler::Active},
    ui::{
        playlist::draw_playlist_sidebar,
        style::{Icon, IconKind, Palette},
    },
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

pub struct Highlight {
    is_active: bool,
    is_hovered: bool,
}

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

    frame.render_widget(ratatui::widgets::Clear, frame.area());
    frame.render_widget(
        Block::default().style(Style::default().bg(palette.background).fg(palette.muted)),
        frame.area(),
    );

    let [header, middle, bottom] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(5),
    ])
    .areas(frame.area());

    let [sidebar, main, player] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Min(0),
        Constraint::Length(0),
    ])
    .areas(middle);

    let [library, playlist] =
        Layout::vertical([Constraint::Length(4), Constraint::Min(0)]).areas(sidebar);

    let (active, hovered) = state.currently_active();
    let highlight = Highlight::new(active == Active::Home, hovered == Active::Home);

    frame.render_widget(
        Block::bordered()
            .border_style(highlight.get(&palette))
            .title(pad("Main", 1)),
        main,
    );

    draw_library(frame, state, &palette, library);
    draw_playing(frame, state, &palette, bottom);
    draw_search(frame, state, &palette, header);
    draw_playlist_sidebar(frame, state, &palette, playlist);
}

fn draw_library(frame: &mut Frame, state: &State, palette: &Palette, area: Rect) {
    let (active, hovered) = state.currently_active();
    let highlight = Highlight::new(active == Active::Library, hovered == Active::Library);

    let block = Block::bordered()
        .title(pad("Pages", 1))
        .style(highlight.get(&palette));

    let mut lines = vec![];

    lines.push(Line::from(Span::styled(
        Icon::new(IconKind::Home).to_string(),
        highlight.get(&palette),
    )));

    lines.push(Line::from(vec![Span::styled(
        Icon::new(IconKind::Library).to_string(),
        highlight.get(&palette),
    )]));

    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

fn draw_playing(frame: &mut Frame, state: &State, palette: &Palette, area: Rect) {
    let (active, hovered) = state.currently_active();
    let highlight = Highlight::new(active == Active::Playing, hovered == Active::Playing);

    let block = Block::bordered()
        .style(highlight.get(&palette))
        .title(pad("Playing", 1));

    frame.render_widget(block, area);
}

fn draw_search(frame: &mut Frame, state: &State, palette: &Palette, area: Rect) {
    let input = Block::bordered()
        .title(pad("What do you wanna listen?", 2))
        .title_alignment(Alignment::Center);
    let input = Paragraph::new("").block(input);

    frame.render_widget(input, area);
}
