use crate::internal::config::ImageKind;
use crate::internal::image::image_url_to_ascii;
use crate::tui::state::WindowSize;
use image::DynamicImage;
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
    Ascii(AsciiImage),
    Image(ResizableImage),
}

#[derive(Debug, Default, PartialEq)]
pub(in crate::tui) struct AsciiImage {
    pub ascii: String,
    pub image_url: String,
    rendered_at_size: WindowSize,
}

pub(in crate::tui) struct ResizableImage {
    pub(in crate::tui) protocol: StatefulProtocol,
    image_buffer: DynamicImage,
    is_black_and_white: bool,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            playing: None,
            image: PlayerImage::default(),
        }
    }

    /// Create and update ascii image if the window size or the image source has changed.
    pub async fn update_current_image(
        &mut self,
        url: &str,
        height: u16,
        width: u16,
        kind: &ImageKind,
    ) {
        match kind {
            ImageKind::Ascii => {
                if let PlayerImage::Ascii(current) = &self.image {
                    let same_size = current.rendered_at_size.height == height && current.rendered_at_size.width == width;
                    
                    if same_size && current.image_url == url {
                        return;
                    }
                };
                self.image = PlayerImage::Ascii(AsciiImage {
                    ascii: image_url_to_ascii(url, &height, &width)
                        .await
                        .unwrap_or_default(),
                    image_url: url.to_string(),
                    rendered_at_size: WindowSize { height, width },
                });
            }
            ImageKind::Image => {
                let req = reqwest::get(url).await.unwrap();

                self.image =
                    PlayerImage::Image(ResizableImage::new(req.bytes().await.unwrap().to_vec()))
            }
        }
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

        match is_synced {
            true => self.parse_lyrics(),
            false => self.length = self.lyrics.lines().count(),
        }
    }

    pub(super) fn reset_lyrics(&mut self) {
        self.timed_lyrics = None;
        self.ordered_timestamps = None;
        self.length = 0;
        self.lyrics = String::new();
        self.offset = 0;
        self.scrollbar_state = ScrollbarState::default();
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

impl ResizableImage {
    fn new(buf: Vec<u8>) -> Self {
        let picker = Picker::from_query_stdio().unwrap();
        let image = image::load_from_memory(&buf).unwrap();
        let protocol = picker.new_resize_protocol(image.clone());

        ResizableImage {
            protocol,
            image_buffer: image,
            is_black_and_white: false,
        }
    }

    pub(in crate::tui) fn black_and_white(&mut self) {
        let picker = Picker::from_query_stdio().unwrap();
        self.image_buffer = self.image_buffer.grayscale();
        let protocol = picker.new_resize_protocol(self.image_buffer.clone());

        self.protocol = protocol;
        self.is_black_and_white = true;
    }

    pub(in crate::tui) fn is_black_and_white(&self) -> bool {
        self.is_black_and_white
    }
}

impl Default for PlayerImage {
    fn default() -> Self {
        PlayerImage::Ascii(AsciiImage::default())
    }
}

impl std::fmt::Debug for PlayerImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerImage::Ascii(ascii) => f.debug_tuple("Ascii").field(ascii).finish(),
            PlayerImage::Image(_) => f.debug_tuple("Image").field(&"<protocol>").finish(),
        }
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
