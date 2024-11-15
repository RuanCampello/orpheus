use crate::tui::state::State;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, Paragraph};
use ratatui::Frame;

pub fn draw_player<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    let player = Block::bordered().border_type(BorderType::Rounded);

    if let Some(image) = &state.player.image {
        let image = Paragraph::new(image.to_string());
        frame.render_widget(image, area);
    }
}
