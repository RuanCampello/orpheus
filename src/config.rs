//! User configuration.

use std::time::Duration;

use ratatui::style::Color as Colour;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub theme: Theme,
    /// Duration in milliseconds between tick events.
    pub tick_rate: Duration,
}

#[derive(Debug, Default, Clone, Copy)]
/// Colour theme used to derive the UI schema.
pub enum Theme {
    /// From: <https://catppuccin.com/palette/> mauve variant
    #[default]
    Catppuccin,
}

#[derive(Debug)]
pub struct Palette {
    pub accent: Colour,
    pub text: Colour,
    pub subtext: Colour,
    pub muted: Colour,
    pub background: Colour,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(250),
            theme: Default::default(),
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
