use crate::tui::components::{pad, BlockExt};
use crate::tui::state::State;
use ratatui::layout::{Position, Rect};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

pub fn draw_search_input<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    let input = Block::new()
        .title(pad("What do you wanna listen?", 2))
        .bordered_section(state.search_state.active);
    let input = Paragraph::new(state.search_state.input.as_str()).block(input);

    if state.search_state.active {
        frame.set_cursor_position(Position::new(
            area.x + state.search_state.cursor_position.saturating_add(1) as u16,
            area.y + 1,
        ));
    }

    frame.render_widget(input, area);
}
