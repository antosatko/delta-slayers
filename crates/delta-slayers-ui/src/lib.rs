//! Carbon UI components for Delta Slayers.
//!
//! All UI hierarchy is authored with Bevy Scene Notation. The crate provides
//! presentation primitives and small presentation-only behaviours; it does not
//! own game state or game navigation.

mod theme;
mod widgets;

pub use theme::{CarbonButtonTone, CarbonPlugin, CarbonSelection, CarbonStatus, CarbonTab, CarbonTheme};
pub use widgets::{
    CarbonIcon, action_button, badge, eyebrow_text, equipment_slot, item_art_tile, item_tile,
    item_tile_optional, key_hint, muted_text, page_header, page_title, panel, reference_term, root, screen_tab,
    screen_tabs, section_header, stat_bar,
};

// Compatibility names for call sites that have not migrated from Slate yet.
pub use theme::CarbonButtonTone as SlateButtonTone;
pub use theme::CarbonPlugin as SlatePlugin;
pub use theme::CarbonStatus as SlateStatus;
pub use theme::CarbonTheme as SlateTheme;
pub use widgets::CarbonIcon as SlateIcon;
