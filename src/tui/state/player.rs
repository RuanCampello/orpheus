use crate::internal::image::{colour_from_image, image_url_to_ascii, Rgb};
use crate::tui::state::WindowSize;
use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};
use ratatui::widgets::ScrollbarState;
use rspotify::model::playing::Playing;
use std::ops::{Div, Mul};

pub(in crate::tui) struct PlayerState {
    pub playing: Option<Playing>,
    pub image: Option<Image>,
}

#[derive(Default)]
pub(in crate::tui) struct Image {
    pub ascii: String,
    pub image_url: String,
    pub colour: Rgb,
    rendered_at_size: WindowSize,
}

#[derive(Default)]
pub(in crate::tui) struct LyricState {
    pub active: bool,
    is_dragging: bool,
    pub offset: usize,
    pub length: usize,
    drag_start: Option<u16>,
    pub area: Rect,
    pub lyrics: String,
    pub scrollbar_state: ScrollbarState,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            playing: None,
            image: None,
        }
    }

    /// Create and update ascii image if the window size or the image source has changed.
    pub async fn update_current_image(&mut self, url: &str, height: u16, width: u16) {
        if let Some(current_image) = &self.image {
            let same_size = current_image.rendered_at_size.height == height
                && current_image.rendered_at_size.width == width;

            if current_image.image_url == url && same_size {
                return;
            }
        }

        self.image = Some(Image {
            ascii: image_url_to_ascii(url, &height, &width)
                .await
                .unwrap_or_default(),
            colour: colour_from_image(url).await.unwrap_or_default(),
            image_url: url.to_string(),
            rendered_at_size: WindowSize { height, width },
        });
    }

    pub fn get_artist_name(&self) -> Option<&str> {
        let Some(playing) = &self.playing else {
            return None;
        };

        if let Some(artist) = playing.item.as_ref()?.artists.first() {
            return Some(artist.name.as_str());
        }

        None
    }
}

impl LyricState {
    /// Updates the current state of the [scrollbar](ScrollbarState)
    /// and [lyrics](Self) based on mouse events.
    pub(super) fn handle_scroll(&mut self, mouse: &MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.area.contains(Position::new(mouse.column, mouse.row)) {
                    self.is_dragging = true;
                    self.drag_start = Some(mouse.row);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.is_dragging = false;
                self.drag_start = None;
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.is_dragging {
                    let Some(start_y) = self.drag_start else {
                        return;
                    };

                    let delta = mouse.row as isize - start_y as isize;

                    self.offset =
                        (self.offset as isize + delta).max(0).min(self.length as _) as usize;
                }
            }
            _ => {}
        }

        self.scrollbar_state = self.scrollbar_state.position(self.offset);
    }

    /// Updates the actual value of the lyrics and it's length.
    pub(super) fn update(&mut self, new_lyrics: String) {
        let count = new_lyrics.lines().count();
        self.length = count;
        self.lyrics = new_lyrics;
    }
}
