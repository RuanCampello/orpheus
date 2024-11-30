use rspotify::model::device::DevicePayload;

pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub port: Option<u16>,

    pub device_id: Option<String>,
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

        Self {
            client_id,
            client_secret,
            port: Some(port),
            device_id: None,
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
