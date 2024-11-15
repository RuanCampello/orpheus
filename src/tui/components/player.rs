use crate::tui::colours::Palette;
use crate::tui::components::BlockExt;
use crate::tui::state::State;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

pub fn draw_player<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    let [image_area, remaining_area] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Min(0)]).areas(area);

    if let Some(image) = &state.player.image {
        let image = Paragraph::new(image.to_string())
            .block(Block::new().secondary_border())
            .style(Style::new().fg(Color::White));
        frame.render_widget(image, image_area);
    }

    if let Some(playing) = &state.player.playing {
        let song = playing.item.as_ref().unwrap();

        let lines = vec![
            Line::from(Span::styled(song.name.as_str(), Style::new().bold())),
            Line::from("\n"),
            Line::from(Span::styled(
                song.artists.first().unwrap().name.as_str(),
                Style::new().fg(Palette::Foreground.into()),
            )),
        ];

        let info = Paragraph::new(Text::from(lines))
            .block(Block::new().secondary_border())
            .alignment(Alignment::Center);
        frame.render_widget(info, remaining_area);
    }
}
