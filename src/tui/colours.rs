use ratatui::style::Color;
use crate::internal::image::Rgb;

#[allow(dead_code)]
pub(super) enum Palette {
    Primary,
    Tertiary,
    Accent,
    Background,
    Foreground,
}

impl From<Palette> for Color {
    fn from(colour: Palette) -> Color {
        match colour {
            Palette::Primary => Rgb::default().into(),
            Palette::Tertiary => Color::Rgb(255, 77, 148),
            Palette::Accent => Color::Rgb(59, 226, 156),
            Palette::Background => Color::Rgb(52, 52, 51),
            Palette::Foreground => Color::Rgb(97, 95, 100),
        }
    }
}