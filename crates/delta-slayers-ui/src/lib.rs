//! Carbon UI components for Delta Slayers.
//!
//! All UI hierarchy is authored with Bevy Scene Notation. The crate provides
//! presentation primitives and small presentation-only behaviours; it does not
//! own game state or game navigation.

mod theme;
mod widgets;

pub use theme::{
    CarbonButtonTone, CarbonPlugin, CarbonSelection, CarbonStatus, CarbonTab, CarbonTheme,
};
pub use widgets::{
    CarbonIcon, action_button, badge, equipment_slot, eyebrow_text, gear_slot, item_art_tile,
    item_tile, item_tile_optional, key_hint, muted_text, page_header, page_title, panel,
    reference_term, root, screen_tab, screen_tabs, section_header, stat_bar, CARBON_ICON_ATLAS_PATH,
};
