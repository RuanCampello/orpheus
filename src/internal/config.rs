use rspotify::model::device::DevicePayload;

pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub port: Option<u16>,

    pub player_image_kind: ImageKind,
    pub device_id: Option<String>,
}

#[derive(Debug)]
pub enum ImageKind {
    Ascii,
    Image,
}

const PORT: u16 = 8888;

impl Config {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        let client_id = dotenv::var("CLIENT_ID").expect("CLIENT_ID must be set");
        let client_secret = dotenv::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set");
        let port: u16 = dotenv::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(PORT);
        let player_image_kind = dotenv::var("PLAYER_IMAGE_KIND")
            .ok()
            .map(ImageKind::from)
            .unwrap_or(ImageKind::Ascii);

        Self {
            client_id,
            client_secret,
            port: Some(port),
            device_id: None,
            player_image_kind,
        }
    }

    pub fn set_default_device(&mut self, payload: DevicePayload) {
        if let Some(dev) = payload.devices.into_iter().next() {
            self.device_id = Some(dev.id);
        }
    }

    pub fn get_port(&self) -> u16 {
        self.port.unwrap_or(PORT)
    }

    pub fn get_redirect_uri(&self) -> String {
        format!("http://localhost:{}/callback", self.get_port())
    }
}

impl From<String> for ImageKind {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "image" => Self::Image,
            _ => Self::Ascii,
        }
    }
}
