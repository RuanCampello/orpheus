//! Key handlers for each view of the [state](crate::state::State).
//!
//! That's used to modify the state based on user interation.

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
#[derive(Debug, Clone, Copy)]
pub(crate) enum Active {
    Album,
    Search,
    Home,
    Library,
    None,
}

pub const DEFAULT_VIEW: View = View {
    id: ViewId::Home,
    active: Active::None,
    hovered: Active::Library,
};
