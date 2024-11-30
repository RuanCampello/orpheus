use crate::tui::components::{pad, BlockExt};
use crate::tui::state::State;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw_volume_widget<'a>(frame: &'a mut Frame, state: &'a mut State, area: Rect) {
    let percentage = state.device.volume_percent;
    let blocks = (percentage / 10) as usize;
    let half_blocks = if percentage % 10 >= 5 { 1 } else { 0 };

    let mut bar = String::new();
    bar.push_str(&"█ ".repeat(blocks));

    if half_blocks > 0 {
        bar.push('▄')
    }

    let volume_widget = Paragraph::new(Line::from(bar))
        .block(
            Block::new()
                .secondary_border()
                .padding(Padding::left(1))
                .title(pad(&format!("Volume {percentage}%"), 2))
                .title_alignment(Alignment::Center),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(volume_widget, area);
}
