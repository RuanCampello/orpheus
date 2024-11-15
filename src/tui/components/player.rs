use crate::tui::state::State;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, BorderType, Paragraph};
use ratatui::Frame;

pub fn draw_player<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    let [image_area, remaining_area] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Min(0)]).areas(area);
    let player = Block::bordered().border_type(BorderType::Rounded);

    if let Some(image) = &state.player.image {
        let image = Paragraph::new(image.to_string()).block(player);
        frame.render_widget(image, image_area);
    }
}
