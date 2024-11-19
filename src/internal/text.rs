use font8x8::UnicodeFonts;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, StyledGrapheme};
use ratatui::widgets::Widget;
use std::cmp::min;

#[derive(Default)]
pub enum Size {
    #[default]
    Full,
    HalfHeight,
    HalfWidth,
    Quarter,
    Sixth,
}

impl Size {
    pub fn pixels_per_cell(&self) -> (u16, u16) {
        match self {
            Size::Full => (1, 1),
            Size::HalfHeight => (1, 2),
            Size::HalfWidth => (2, 1),
            Size::Quarter => (2, 2),
            Size::Sixth => (2, 3),
        }
    }

    /// Returns a symbol for a given position
    pub fn symbol(&self, glyph: &[u8; 8], row: usize, col: i32) -> char {
        match self {
            Size::Full => match glyph[row] & 1 << col {
                0 => ' ',
                _ => '█',
            },
            Size::HalfHeight => {
                let top = glyph[row] & 1 << col;
                let bottom = glyph[row + 1] & 1 << col;

                symbol_for_half_height(top, bottom)
            }
            Size::HalfWidth => {
                let left = glyph[row] & 1 << col;
                let right = glyph[row] & 1 << (col + 1);

                symbol_for_half_width(left, right)
            }
            Size::Quarter => {
                let tl = glyph[row] & 1 << col;
                let tr = glyph[row] & 1 << (col + 1);
                let bl = glyph[row + 1] & 1 << col;
                let br = glyph[row + 1] & 1 << (col + 1);

                symbol_for_quarter(tl, tr, bl, br)
            }
            Size::Sixth => {
                let tl = glyph[row] & 1 << col;
                let tr = glyph[row] & 1 << (col + 1);

                let (mid_l, mid_r) = if (row + 1) < glyph.len() {
                    (glyph[row + 1] & 1 << col, glyph[row + 1] << (col + 1))
                } else {
                    (0, 0)
                };

                let (bl, br) = if (row + 2) < glyph.len() {
                    (glyph[row + 2] & 1 << col, glyph[row + 2] & 1 << (col + 1))
                } else {
                    (0, 0)
                };

                symbol_for_sixth(tl, tr, mid_l, mid_r, bl, br)
            }
        }
    }
}

fn symbol_for_half_height(top: u8, bottom: u8) -> char {
    match top {
        0 => match bottom {
            0 => ' ',
            _ => '▄',
        },
        _ => match bottom {
            0 => '▀',
            _ => '█',
        },
    }
}

fn symbol_for_half_width(left: u8, right: u8) -> char {
    match left {
        0 => match right {
            0 => ' ',
            _ => '▐',
        },
        _ => match right {
            0 => '▌',
            _ => '█',
        },
    }
}

const QUADRANT_SYMBOLS: [char; 16] = [
    ' ', '▘', '▝', '▀', '▖', '▌', '▞', '▛', '▗', '▚', '▐', '▜', '▄', '▙', '▟', '█',
];

fn symbol_for_quarter(top_left: u8, top_right: u8, bottom_left: u8, bottom_right: u8) -> char {
    let tl = get_position_for_value(top_left);
    let tr = get_position_for_value(top_right);
    let bl = get_position_for_value(bottom_left);
    let br = get_position_for_value(bottom_right);

    let char_i = tl + (tr << 1) + (bl << 2) + (br << 3);
    QUADRANT_SYMBOLS[char_i]
}

fn get_position_for_value(pos: u8) -> usize {
    if pos > 0 {
        1
    } else {
        0
    }
}

const SIXTH_SYMBOLS: [char; 64] = [
    ' ', '🬀', '🬁', '🬂', '🬃', '🬄', '🬅', '🬆', '🬇', '🬈', '🬉', '🬊', '🬋', '🬌', '🬍', '🬎', '🬏', '🬐', '🬑',
    '🬒', '🬓', '▌', '🬔', '🬕', '🬖', '🬗', '🬘', '🬙', '🬚', '🬛', '🬜', '🬝', '🬞', '🬟', '🬠', '🬡', '🬢', '🬣',
    '🬤', '🬥', '🬦', '🬧', '▐', '🬨', '🬩', '🬪', '🬫', '🬬', '🬭', '🬮', '🬯', '🬰', '🬱', '🬲', '🬳', '🬴', '🬵',
    '🬶', '🬷', '🬸', '🬹', '🬺', '🬻', '█',
];

fn symbol_for_sixth(
    top_left: u8,
    top_right: u8,
    mid_left: u8,
    mid_right: u8,
    bottom_left: u8,
    bottom_right: u8,
) -> char {
    let tl = get_position_for_value(top_left);
    let tr = get_position_for_value(top_right);
    let ml = get_position_for_value(mid_left);
    let mr = get_position_for_value(mid_right);
    let bl = get_position_for_value(bottom_left);
    let br = get_position_for_value(bottom_right);

    let char_i = tl + (tr << 1) + (ml << 2) + (mr << 3) + (bl << 4) + (br << 5);

    SIXTH_SYMBOLS[char_i]
}

macro_rules! method_builder {
    ($name:ident, $ty:ty, $field:ident) => {
        pub fn $name(mut self, value: $ty) -> Self {
            self.$field = value;
            self
        }
    };
}

pub struct Text<'a> {
    lines: &'a [&'a Line<'a>],
    style: Style,
    size: &'a Size,
    alignment: Alignment,
}

impl<'a> Text<'a> {
    pub fn new() -> Text<'a> {
        static EMPTY_LINES: &[&Line] = &[];
        static DEFAULT_SIZE: Size = Size::Full;

        Text {
            lines: EMPTY_LINES,
            style: Style::default(),
            size: &DEFAULT_SIZE,
            alignment: Alignment::Left,
        }
    }

    fn layout(
        &'a self,
        area: Rect,
    ) -> impl IntoIterator<Item = impl IntoIterator<Item = Rect>> + 'a {
        let (x, y) = self.size.pixels_per_cell();
        let width = 8_u16.div_ceil(x);
        let height = 8_u16.div_ceil(y);

        (area.top()..area.bottom())
            .step_by(height as usize)
            .zip(self.lines.iter())
            .map(move |(y, l)| {
                let offset = get_align_offset(area.width, width, self.alignment, l);
                (area.left() + offset..area.right())
                    .step_by(width as usize)
                    .map(move |x| {
                        let width = min(area.right() - x, width);
                        let height = min(area.bottom() - y, height);
                        Rect::new(x, y, width, height)
                    })
            })
    }

    method_builder!(lines, &'a [&'a Line<'a>], lines);
    method_builder!(style, Style, style);
    method_builder!(size, &'a Size, size);
    method_builder!(alignment, Alignment, alignment);
}

impl<'a> Widget for Text<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let layout = self.layout(area);
        for (line, layout_line) in self.lines.iter().zip(layout) {
            for (gr, cell) in line.styled_graphemes(self.style).zip(layout_line) {
                render_symbol(gr, cell, buf, self.size);
            }
        }
    }
}

fn get_align_offset<'a>(
    total_width: u16,
    width: u16,
    alignment: Alignment,
    line: &'a Line<'a>,
) -> u16 {
    let line_width = line.width() as u16 * width;
    match alignment {
        Alignment::Center => (total_width / 2).saturating_sub(line_width / 2),
        Alignment::Right => total_width.saturating_sub(line_width),
        Alignment::Left => 0,
    }
}

fn render_symbol(graph: StyledGrapheme, area: Rect, buff: &mut Buffer, size: &Size) {
    buff.set_style(area, graph.style);

    let c = graph.symbol.chars().next().unwrap_or_default();
    if let Some(glyph) = font8x8::BASIC_FONTS.get(c) {
        render_glyph(glyph, area, buff, size);
    }
}

fn render_glyph(glyph: [u8; 8], area: Rect, buff: &mut Buffer, size: &Size) {
    let (x, y) = size.pixels_per_cell();

    let glyph_y_index = (0..glyph.len()).step_by(y as usize);

    for (y_idx, row) in glyph_y_index.zip(area.rows()) {
        let mut glyph_x_selector = (0..8).step_by(x as usize);
        for col in row.columns() {
            if let Some(x_idx) = glyph_x_selector.next() {
                buff[col].set_char(size.symbol(&glyph, y_idx, x_idx));
            } else {
                break;
            }
        }
    }
}
