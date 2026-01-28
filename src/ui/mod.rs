//! UI components to be rendered from [state](crate::state::State).

mod playlist;

use crate::{
    config::Palette,
    state::{State, handler::Active},
    ui::playlist::draw_playlist_sidebar,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
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

    let [playlist, main, player] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Min(0),
        Constraint::Length(0),
    ])
    .areas(middle);

    let (active, hovered) = state.currently_active();
    let highlight = Highlight::new(active == Active::Home, hovered == Active::Home);

    frame.render_widget(
        Block::bordered()
            .border_style(highlight.get(&palette))
            .title("Main"),
        main,
    );

    frame.render_widget(
        Block::bordered()
            .style(highlight.get(&palette))
            .border_style(Style::new().fg(palette.muted))
            .title(pad("Playing", 1)),
        bottom,
    );

    draw_search(frame, state, &palette, header);
    draw_playlist_sidebar(frame, state, &palette, playlist);
}

fn draw_playing(frame: &mut Frame, state: &State, palette: &Palette, area: Rect) {}

fn draw_search(frame: &mut Frame, state: &State, palette: &Palette, area: Rect) {
    let input = Block::bordered()
        .title(pad("What do you wanna listen?", 2))
        .title_alignment(Alignment::Center);
    let input = Paragraph::new("").block(input);

    frame.render_widget(input, area);
}
