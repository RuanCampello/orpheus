//! State related module.
//!
//! The [state](self::State) holds the information we need
//! in order to correctly (re)generate the UI.
//!
//! It's read by the [io manager](crate::io::Io) that queues the events
//! inputted by the user so that we can build the UI retaining data as possible
//! and non-blocking.

use std::sync::mpsc::Sender;

use crate::{config::Config, io::Event};

/// All state that the application holds
/// in order to render the UI.
#[derive(Default)]
pub(crate) struct State {
    pub config: Config,

    sender: Option<Sender<Event>>,
}

impl State {
    pub fn new(config: Config, sender: Sender<Event>) -> Self {
        Self {
            config,
            sender: Some(sender),
        }
    }
}
