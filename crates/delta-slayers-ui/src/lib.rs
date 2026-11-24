//! Reusable Slate UI components for Delta Slayers.
//!
//! All UI hierarchy is authored with Bevy Scene Notation. The crate provides
//! presentation primitives and small presentation-only behaviours; it does not
//! own game state or game navigation.

mod theme;
mod widgets;

pub use theme::{SlateButtonTone, SlatePlugin, SlateStatus, SlateTheme};
pub use widgets::{
    action_button, badge, item_tile, page_header, panel, reference_term, root, section_header,
    stat_bar, SlateIcon,
};
