//! State related module.
//!
//! The [state](self::State) holds the information we need
//! in order to correctly (re)generate the UI.
//!
//! It's read by the [io manager](crate::io::Io) that queues the events
//! inputted by the user so that we can build the UI retaining data as possible
//! and non-blocking.

pub(crate) mod handler;

use crate::{
    config::Config,
    io::Event,
    state::handler::{Active, DEFAULT_VIEW, View},
};
use rspotify::model::{CurrentPlaybackContext, Page, PlayableItem, SimplifiedPlaylist};
use std::{sync::mpsc::Sender, time::Instant};

/// All state that the application holds
/// in order to render the UI.
pub(crate) struct State {
    pub config: Config,

    sender: Option<Sender<Event>>,

    pub playlists: Option<Page<SimplifiedPlaylist>>,

    pub current_playback_context: Option<CurrentPlaybackContext>,
    last_playback_pool: Instant,
    is_fetching_playback: bool,

    navigation: Vec<View>,

    seek_ms: Option<u128>,
}

impl State {
    pub fn new(config: Config, sender: Sender<Event>) -> Self {
        Self {
            config,
            sender: Some(sender),
            playlists: None,
            last_playback_pool: Instant::now(),
            current_playback_context: None,
            seek_ms: None,
            is_fetching_playback: false,
            navigation: vec![DEFAULT_VIEW],
        }
    }

    pub fn dispatch(&mut self, event: Event) {
        if let Some(sender) = &self.sender {
            if let Err(err) = sender.send(event) {
                panic!("{err}")
            }
        }
    }

    #[inline(always)]
    pub fn currently_active(&self) -> (Active, Active) {
        let view = self.current_view();
        (view.active, view.hovered)
    }

    #[inline(always)]
    pub fn current_view(&self) -> &View {
        self.navigation.last().unwrap_or(&DEFAULT_VIEW)
    }

    pub fn mut_current_view(&mut self) -> &mut View {
        self.navigation.last_mut().unwrap()
    }

    pub fn set_current_view(&mut self, active: Option<Active>, hovered: Option<Active>) {
        let current = self.mut_current_view();

        if let Some(active) = active {
            current.active = active;
        }

        if let Some(hovered) = hovered {
            current.hovered = hovered;
        }
    }

    pub fn update_tick(&mut self) {
        todo!()
    }

    fn poll_playback(&mut self) {
        const POOL_INTERVAL: u128 = 5_000;

        let elapsed = self.last_playback_pool.elapsed().as_millis();
        if !self.is_fetching_playback && elapsed >= POOL_INTERVAL {
            match self.seek_ms {
                Some(seek) => todo!(),
                _ => self.dispatch(Event::GetCurrentPlayback),
            }
        }
    }

    fn seek(&mut self, ms: u32) {
        if let Some(CurrentPlaybackContext {
            item: Some(item), ..
        }) = &self.current_playback_context
        {
            let duration = match item {
                PlayableItem::Track(track) => track.duration,
                PlayableItem::Episode(episode) => episode.duration,
                _ => Default::default(),
            }
            .num_milliseconds() as u32;

            let event = match ms < duration {
                true => Event::Seek(ms),
                _ => Event::NextTrack,
            };

            self.dispatch(event);
        }
    }
}
