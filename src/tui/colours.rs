use ratatui::style::Color;

#[allow(dead_code)]
pub(super) enum Palette {
    Primary,
    Secondary,
    Tertiary,
    Accent,
    Background,
    Foreground,
}

impl From<Palette> for Color {
    fn from(colour: Palette) -> Color {
        match colour {
            Palette::Primary => Color::Rgb(107, 0, 230),
            Palette::Secondary => Color::Rgb(135, 75, 252),
            Palette::Tertiary => Color::Rgb(255, 77, 148),
            Palette::Accent => Color::Rgb(59, 226, 156),
            Palette::Background => Color::Rgb(52, 52, 51),
            Palette::Foreground => Color::Rgb(97, 95, 100),
        }
    }
}
