//! State related module.
//!
//! The [state](self::State) holds the information we need
//! in order to correctly (re)generate the UI.
//!
//! It's read by the [io manager](crate::io::Io) that queues the events
//! inputted by the user so that we can build the UI retaining data as possible
//! and non-blocking.

use std::sync::mpsc::Sender;

use rspotify::model::{Page, SimplifiedPlaylist};

use crate::{config::Config, io::Event};

/// All state that the application holds
/// in order to render the UI.
#[derive(Default)]
pub(crate) struct State {
    pub config: Config,

    sender: Option<Sender<Event>>,

    pub playlists: Option<Page<SimplifiedPlaylist>>,
}

impl State {
    pub fn new(config: Config, sender: Sender<Event>) -> Self {
        Self {
            config,
            sender: Some(sender),
            ..Default::default()
        }
    }

    pub fn dispatch(&mut self, event: Event) {
        if let Some(sender) = &self.sender {
            if let Err(err) = sender.send(event) {
                panic!("{err}")
            }
        }
    }
}
