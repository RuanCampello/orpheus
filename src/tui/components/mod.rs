pub(super) mod player;
pub(super) mod playlist;
pub(super) mod search;

use crate::tui::colours::Palette;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, BorderType, Borders, List, Row};

fn pad(content: &str, size: usize) -> String {
    let padding = " ".repeat(size);
    format!("{padding}{content}{padding}")
}

trait ToRow<'a> {
    fn to_row(&self, idx: usize) -> Row<'a>;
}

trait BlockExt {
    fn bordered_section(self, active: bool) -> Self;
    fn secondary_border(self) -> Self;
}

trait ListExt {
    fn highlightable_section(self, active: bool) -> Self;
}

impl<'a> BlockExt for Block<'a> {
    fn bordered_section(self, active: bool) -> Self {
        self.borders(Borders::ALL).border_style(match active {
            true => Style::new().fg(Palette::Secondary.into()),
            false => Style::new().fg(Palette::Foreground.into()),
        })
    }
    fn secondary_border(self) -> Self {
        self.borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(Palette::Foreground.into()))
    }
}

impl<'a> ListExt for List<'a> {
    fn highlightable_section(self, active: bool) -> Self {
        self.highlight_style(match active {
            true => Style::new().bg(Palette::Secondary.into()).bold(),
            false => Style::new().bg(Palette::Foreground.into()),
        })
    }
}
