use rspotify::model::playing::Playing;

pub(in crate::tui) struct PlayerState {
    pub playing: Option<Playing>,
    pub image: Option<String>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            playing: None,
            image: None,
        }
    }
}
