//! User configuration.

use ratatui::style::Color as Colour;

#[derive(Debug)]
pub struct Config {
    theme: Theme,
}

#[derive(Debug, Default)]
/// Colour theme used to derive the UI schema.
pub enum Theme {
    /// From: <https://catppuccin.com/palette/> mauve variant
    #[default]
    Catppuccin,
}

#[derive(Debug)]
pub struct Palette {
    accent: Colour,
    text: Colour,
    subtext: Colour,
}

impl From<Theme> for Palette {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Catppuccin => Palette {
                accent: Colour::Rgb(203, 166, 247),
                text: Colour::Rgb(205, 214, 244),
                subtext: Colour::Rgb(166, 173, 200),
            },
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        todo!()
    }
}
