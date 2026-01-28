//! Key handlers for each view of the [state](crate::state::State).
//!
//! That's used to modify the state based on user interation.

#![allow(unused)]

use crate::{io::key::Key, state::State};

/// Represents the full state of the current view.
///
/// To render a screen-based view, like a page in web, we need two main informations:
///     - what's the current view?
///     - what's active in it?
///
/// For the latter, we divide more or less as a browser.
/// We have an [active state](self::Active) for both the active and hovered items.
pub(crate) struct View {
    pub id: ViewId,

    pub active: Active,
    pub hovered: Active,
}

/// An identifier of a full screen.
pub(crate) enum ViewId {
    Home,
    Search,
}

/// This represents an UI active block.
///
/// An actie block can be wether a selected or hovered block.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Active {
    Album,
    Search,
    Playlists,
    Playing,
    Home,
    Library,
    None,
}

pub const DEFAULT_VIEW: View = View {
    id: ViewId::Home,
    active: Active::None,
    hovered: Active::Library,
};

pub fn handle(key: Key, state: &mut State) {
    match key {
        Key::Esc => handle_esc(state),
        _ => handle_view(key, state),
    }
}

fn handle_view(key: Key, state: &mut State) {
    let current = state.current_view();

    match current.active {
        Active::None => handler(key, state),
        _ => {}
    }
}

/// Default event handler for `None` active state.
fn handler(key: Key, state: &mut State) {
    match key {
        Key::Enter => {
            let hovered = state.current_view().hovered;
            state.set_current_view(Some(hovered), None);
        }

        Key::Up => match state.current_view().hovered {
            Active::Playlists => state.set_current_view(None, Some(Active::Library)),
            Active::Playing => state.set_current_view(None, Some(Active::Playlists)),
            _ => {}
        },

        Key::Down => match state.current_view().hovered {
            Active::Library => state.set_current_view(None, Some(Active::Playlists)),
            Active::Home | Active::Album | Active::Playlists => {
                state.set_current_view(None, Some(Active::Playing))
            }
            _ => {}
        },

        Key::Right => match state.current_view().hovered {
            Active::Playlists | Active::Library => match state.current_view().id {
                ViewId::Home => state.set_current_view(None, Some(Active::Home)),
                _ => {}
            },
            _ => {}
        },

        Key::Left => match state.current_view().hovered {
            Active::Album | Active::Home => state.set_current_view(None, Some(Active::Library)),
            _ => {}
        },
        _ => {}
    }
}

fn handle_esc(state: &mut State) {
    match state.current_view().active {
        _ => state.set_current_view(Some(Active::None), None),
    }
}
