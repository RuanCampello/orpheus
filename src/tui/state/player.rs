use crate::internal::image::image_url_to_ascii;
use rspotify::model::playing::Playing;

#[derive(Debug)]
pub(in crate::tui) struct PlayerState {
    pub playing: Option<Playing>,
    pub image: Option<Image>,
}

#[derive(Debug)]
pub(in crate::tui) struct Image {
    pub ascii: String,
    pub image_url: String,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            playing: None,
            image: None,
        }
    }

    pub fn update_current_image(&mut self, url: &str, height: &u16) {
        if let Some(current_image) = &self.image {
            if current_image.image_url == url {
                return;
            }
        }

        self.image = Some(Image {
            ascii: image_url_to_ascii(url, height).unwrap_or_default(),
            image_url: url.to_string(),
        });
    }
}
