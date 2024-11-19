use crate::internal::text::{Size, Text};
use crate::tui::components::BlockExt;
use crate::tui::state::State;
use deunicode::deunicode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

pub fn draw_player<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    let [image_area, remaining_area] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Min(0)]).areas(area);

    if let Some(image) = &state.player.image {
        let image = Paragraph::new(image.ascii.to_string())
            .block(Block::new().secondary_border())
            .style(Style::new().fg(Color::White));
        frame.render_widget(image, image_area);
    }

    let [info_area, remaining_area] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Min(0)]).areas(remaining_area);

    if let Some(playing) = &state.player.playing {
        let song = playing.item.as_ref().unwrap();
        let song_name = match song.name.is_ascii() {
            true => song.name.as_str(),
            false => &deunicode(&song.name),
        };

        let lines = &[&Line::from(Span::styled(song_name, Style::new().bold()))];

        let info = Text::new().size(&Size::Quarter).lines(lines);
        frame.render_widget(info, info_area);
    }
}
