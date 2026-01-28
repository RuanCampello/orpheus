//! Key events to be handled.

use crate::config::Config;
use ratatui::crossterm::event::{self, KeyCode, KeyEvent};
use std::sync::mpsc;

/// Handler to deal with input/tick events on its own thread.
pub(crate) struct EventHandler {
    receiver: mpsc::Receiver<Event<Key>>,
    _sender: mpsc::Sender<Event<Key>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum Key {
    Esc,
    Enter,
    Backspace,

    Left,
    Up,
    Right,
    Down,

    Char(char),

    Unmapped,
}

pub(crate) enum Event<I> {
    Input(I),
    Tick,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        EventHandler::with_config(Config {
            tick_rate: std::time::Duration::from_millis(tick_rate),
            ..Default::default()
        })
    }

    pub fn with_config(config: Config) -> Self {
        let (sender, receiver) = mpsc::channel();
        let event_sender = sender.clone();

        std::thread::spawn(move || {
            loop {
                if event::poll(config.tick_rate).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        let key = Key::from(key);
                        event_sender.send(Event::Input(key)).unwrap();
                    }
                }

                event_sender.send(Event::Tick).unwrap();
            }
        });

        EventHandler {
            receiver,
            _sender: sender,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.receiver.recv()
    }
}

impl From<KeyEvent> for Key {
    fn from(value: KeyEvent) -> Self {
        match value {
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => Self::Enter,
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => Self::Backspace,
            KeyEvent {
                code: KeyCode::Esc, ..
            } => Self::Esc,

            KeyEvent {
                code: KeyCode::Left,
                ..
            } => Self::Left,
            KeyEvent {
                code: KeyCode::Right,
                ..
            } => Self::Right,
            KeyEvent {
                code: KeyCode::Up, ..
            } => Self::Up,
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => Self::Down,

            KeyEvent {
                code: KeyCode::Char(c),
                ..
            } => Key::Char(c),

            _ => Self::Unmapped,
        }
    }
}
