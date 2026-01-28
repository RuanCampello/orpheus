use crate::{
    state::{State, handler::Active},
    ui::style::Palette,
    ui::{Highlight, pad},
};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, List, ListItem, ListState},
};

pub fn draw_playlist_sidebar(frame: &mut Frame, state: &State, palette: &Palette, area: Rect) {
    let items = match &state.playlists {
        Some(playlist) => playlist
            .items
            .iter()
            .map(|item| item.name.clone())
            .collect(),
        None => vec![],
    };

    let (active, hovered) = state.currently_active();

    let highlight = Highlight::new(active == Active::Playlists, hovered == Active::Playlists);

    draw_selectable(
        frame,
        palette,
        area,
        "Playlists",
        &items,
        highlight,
        Some(0),
    );
}

fn draw_selectable<I: AsRef<str>>(
    frame: &mut Frame,
    palette: &Palette,
    area: Rect,
    title: &str,
    items: &[I],
    highlight: Highlight,
    index: Option<usize>,
) {
    let mut state = ListState::default();
    state.select(index);

    let items = items
        .iter()
        .map(|item| ListItem::new(Span::raw(item.as_ref())))
        .collect::<Vec<_>>();

    let block = Block::bordered()
        .border_style(highlight.get(&palette))
        .title(Span::styled(pad(title, 1), highlight.get(&palette)))
        .style(Style::default().fg(palette.muted));

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight.get(&palette).add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(list, area, &mut state);
}

impl Highlight {
    pub fn new(is_active: bool, is_hovered: bool) -> Self {
        Self {
            is_active,
            is_hovered,
        }
    }

    pub fn get(&self, palette: &Palette) -> Style {
        match (self.is_active, self.is_hovered) {
            (true, _) => Style::new().fg(palette.accent),
            (false, true) => Style::new().fg(palette.subtext),
            _ => Style::new().fg(palette.muted),
        }
    }
}
