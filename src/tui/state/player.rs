use crate::internal::image::image_url_to_ascii;
use crate::tui::state::WindowSize;
use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};
use ratatui::widgets::ScrollbarState;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use regex::Regex;
use rspotify::model::context::CurrentlyPlaybackContext;
use rspotify::model::{track, PlayingItem};
use std::collections::HashMap;

pub(in crate::tui) struct PlayerState {
    pub playing: Option<CurrentlyPlaybackContext>,
    pub image: PlayerImage,
}

#[derive(Default)]
pub(in crate::tui) struct Image {
    pub ascii: String,
    pub image_url: String,
    rendered_at_size: WindowSize,
}

#[derive(Default)]
pub(in crate::tui) struct LyricState {
    is_dragging: bool,
    drag_start: Option<u16>,
    pub active: bool,
    pub offset: usize,
    pub length: usize,
    pub area: Rect,
    pub lyrics: String,
    pub scrollbar_state: ScrollbarState,

    pub timed_lyrics: Option<HashMap<u32, String>>,
    pub(crate) ordered_timestamps: Option<Vec<u32>>,
    pub(crate) current_time: u32,
}

pub(in crate::tui) enum PlayerImage {
    Ascii(Image),
    Image(StatefulProtocol),
}

pub(in crate::tui) trait UpdateImage {
    async fn apply(self, player: &mut PlayerState);
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            playing: None,
            image: PlayerImage::default(),
        }
    }

    /// Create and update ascii image if the window size or the image source has changed.
    pub async fn update_current_image<Up: UpdateImage>(&mut self, image: Up) {
        image.apply(self).await;
    }

    pub fn get_artist_name(&self) -> Option<&str> {
        let Some(playing) = &self.playing else {
            return None;
        };

        if let Some(artist) = playing
            .item
            .as_ref()
            .and_then(|item| item.as_track())
            .and_then(|track| track.artists.first())
        {
            return Some(artist.name.as_str());
        }

        None
    }

    pub fn tick_progress(&mut self, new_progress: u32) {
        let Some(ctx) = &mut self.playing else { return };
        if !ctx.is_playing {
            return;
        };
        let Some(progress) = &mut ctx.progress_ms else {
            return;
        };

        *progress += new_progress;

        let duration = ctx
            .item
            .as_ref()
            .and_then(|item| item.as_track())
            .map(|track| track.duration_ms);

        if let Some(duration) = duration {
            if *progress > duration {
                *progress = duration;
                ctx.is_playing = false;
            }
        }
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

    /// Updates the actual value of the lyrics and parse it if needed.
    pub(super) fn update(&mut self, new_lyrics: String, is_synced: bool) {
        self.lyrics = new_lyrics;

        if is_synced {
            self.parse_lyrics();
        } else {
            self.length = self.lyrics.lines().count();
        }
    }

    fn parse_lyrics(&mut self) {
        if self.timed_lyrics.is_none() {
            self.timed_lyrics = Some(HashMap::new());
        }
        if self.ordered_timestamps.is_none() {
            self.ordered_timestamps = Some(Vec::new());
        }

        let timed_lyrics = self.timed_lyrics.as_mut().unwrap();
        let ordered_timestamps = self.ordered_timestamps.as_mut().unwrap();

        timed_lyrics.clear();
        ordered_timestamps.clear();

        let re = Regex::new(r"\[(\d+):(\d+\.\d+)\](.*)").unwrap();

        for line in self.lyrics.lines() {
            if let Some(caps) = re.captures(line) {
                let minutes = caps[1].parse::<u32>().unwrap();
                let seconds = caps[2].parse::<f64>().unwrap();
                let timestamp = (minutes * 60 * 1000) + (seconds * 1000.0) as u32;
                let text = caps[3].trim().to_string();

                timed_lyrics.insert(timestamp, text);
                ordered_timestamps.push(timestamp);
            }
        }

        ordered_timestamps.sort_unstable();
        self.length = ordered_timestamps.len();
    }

    pub fn update_time(&mut self, current_time: &u32) {
        self.current_time = *current_time;
    }
}

impl UpdateImage for (&str, u16, u16) {
    async fn apply(self, player: &mut PlayerState) {
        let (url, height, width) = self;

        if let PlayerImage::Ascii(ascii_image) = &player.image {
            let same_size = ascii_image.rendered_at_size.height == height
                && ascii_image.rendered_at_size.width == width;

            if ascii_image.image_url == url && same_size {
                return;
            }

            player.image = PlayerImage::Ascii(Image {
                ascii: image_url_to_ascii(url, &height, &width)
                    .await
                    .unwrap_or_default(),
                image_url: url.to_string(),
                rendered_at_size: WindowSize { height, width },
            });
        }
    }
}

impl UpdateImage for &str {
    async fn apply(self, player: &mut PlayerState) {
        let picker = Picker::from_fontsize((8, 12));

        let req = reqwest::get(self).await.unwrap();
        let bytes = req.bytes().await.unwrap();

        let image = image::load_from_memory(&bytes).unwrap();
        // let width = img.width();
        // let height = img.height();

        let protocol = picker.new_resize_protocol(image);

        player.image = PlayerImage::Image(protocol);
    }
}

impl Default for PlayerImage {
    fn default() -> Self {
        PlayerImage::Ascii(Image::default())
    }
}

pub(in crate::tui) trait AsTrack {
    fn as_track(&self) -> Option<&track::FullTrack>;
}

impl AsTrack for PlayingItem {
    fn as_track(&self) -> Option<&track::FullTrack> {
        if let PlayingItem::Track(ref track) = *self {
            Some(track)
        } else {
            None
        }
    }
}
