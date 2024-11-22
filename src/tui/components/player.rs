use crate::internal::text::{Size, Text};
use crate::tui::components::{pad, BlockExt};
use crate::tui::state::State;
use deunicode::deunicode;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use std::ops::Div;

pub fn draw_player<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    let [image_area, remaining_area] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Min(0)]).areas(area);

    if let Some(image) = &state.player.image {
        let image = Paragraph::new(image.ascii.to_string())
            .block(Block::new().secondary_border())
            .style(Style::new().fg(Color::White));
        frame.render_widget(image, image_area);
    }

    let [title_area, remaining_area] =
        Layout::vertical([Constraint::Length(4), Constraint::Min(0)]).areas(remaining_area);

    if let Some(playing) = &state.player.playing {
        let song = playing.item.as_ref().unwrap();
        let song_name: &str = match song.name.is_ascii() {
            true => song.name.as_str(),
            false => &deunicode(&song.name),
        };

        let padding = ((title_area.width / 4) as usize).saturating_sub(song_name.len()) as f32;
        let padding = padding.div(1.25).round().min(5.0) as usize;

        let title = &[&Line::from(pad(song_name, padding))];
        let title = Text::new().size(&Size::Quarter).lines(title);
        frame.render_widget(title, title_area);

        let artist = state.player.get_artist_name().unwrap_or("Unknown");
        let paragraph = Paragraph::new(vec![Line::from(artist)]).alignment(Alignment::Center);

        frame.render_widget(paragraph, remaining_area);
    }
}
