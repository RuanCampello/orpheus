use crate::internal::image::{colour_from_image, image_url_to_ascii, Rgb};
use crate::tui::state::WindowSize;
use rspotify::model::playing::Playing;

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
