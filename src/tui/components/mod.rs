pub(in crate::tui) mod player;
pub(in crate::tui) mod playlist;
pub(in crate::tui) mod search;
pub(in crate::tui) mod volume;

use crate::internal::image::Rgb;
use crate::tui::colours::Palette;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, BorderType, Borders, List, Row};
use std::ops::{Div, Rem};

fn pad(content: &str, size: usize) -> String {
    let padding = " ".repeat(size);
    format!("{padding}{content}{padding}")
}

fn time_from_ms(ms: &u32) -> String {
    let secs = ms.div(1000);
    let minutes = secs.div(60);
    let seconds = secs.rem(&60);

    format!("{minutes}:{seconds:02}")
}

trait ToRow<'a> {
    fn to_row(&self, idx: usize) -> Row<'a>;
}

trait BlockExt {
    fn bordered_section(self, colour: &Rgb, active: bool) -> Self;
    fn secondary_border(self) -> Self;
}

trait ListExt {
    fn highlightable_section(self, colour: &Rgb, active: bool) -> Self;
}

impl<'a> BlockExt for Block<'a> {
    fn bordered_section(self, colour: &Rgb, active: bool) -> Self {
        self.borders(Borders::ALL).border_style(match active {
            true => Style::new().fg(colour.into()),
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
    fn highlightable_section(self, colour: &Rgb, active: bool) -> Self {
        self.highlight_style(match active {
            true => Style::new().bg(colour.into()).bold(),
            false => Style::new().bg(Palette::Foreground.into()),
        })
    }
}
