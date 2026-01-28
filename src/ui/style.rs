use std::fmt::Display;

use ratatui::style::Color as Colour;

#[derive(Debug, Default, Clone, Copy)]
/// Colour theme used to derive the UI schema.
pub enum Theme {
    /// From: <https://catppuccin.com/palette/> mauve variant
    #[default]
    Catppuccin,
}

#[derive(Debug)]
#[allow(unused)]
pub struct Palette {
    pub accent: Colour,
    pub text: Colour,
    pub subtext: Colour,
    pub muted: Colour,
    pub background: Colour,
}

#[derive(Debug, Clone, Copy)]
/// This represents a icon and its active state for styling.
pub struct Icon {
    kind: IconKind,
    is_active: bool,
}

#[derive(Debug, Clone, Copy)]
/// Nerd fonts wrapper.
pub enum IconKind {
    Library,
    Home,
}

impl Icon {
    pub fn new(kind: IconKind) -> Self {
        Self {
            is_active: false,
            kind,
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active
    }
}

impl Display for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.kind, self.is_active) {
            (IconKind::Library, false) => f.write_str("\u{eaa5}"),
            (IconKind::Library, true) => f.write_str("\u{f02e}"),

            (IconKind::Home, false) => f.write_str("\u{f46d}"),
            (IconKind::Home, true) => f.write_str("\u{f4e2}"),
        }
    }
}

impl From<&Theme> for Palette {
    fn from(theme: &Theme) -> Self {
        match theme {
            Theme::Catppuccin => Palette {
                accent: Colour::Rgb(203, 166, 247),
                text: Colour::Rgb(205, 214, 244),
                subtext: Colour::Rgb(166, 173, 200),
                muted: Colour::Rgb(88, 91, 112),
                background: Colour::Rgb(17, 17, 27),
            },
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette::from(&Theme::Catppuccin)
    }
}
