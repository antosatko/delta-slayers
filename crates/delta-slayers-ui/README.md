# delta-slayers-ui

Game-specific Bevy UI composition helpers for Delta Slayers. The crate uses
Bevy Feathers controls and theme tokens where they fit, with a restrained
Carbon-inspired visual language for game-specific presentation.

## Visual direction

Carbon uses a neutral graphite surface hierarchy with one restrained azure
accent. Ordinary navigation, metadata, icons, and item artwork remain neutral.
Azure is reserved for primary actions, selected states, links, and focus.
Positive, warning, and danger colors appear only where they communicate status,
using subtle tinted surfaces rather than bright fills.

Panels rely on spacing, tonal surfaces, and soft elevation instead of decorative
outlines. Item tiles are square and artwork-first, with the item name and an
optional quantity overlaid at the bottom. Use a hoverable help affordance only
where contextual guidance adds useful information; simple screens use a plain
title. Screens with multiple views can place a tab row directly underneath.

## Setup

Install Bevy Feathers and the game's presentation plugin once:

```rust
use bevy::{
    feathers::{dark_theme::create_dark_theme, theme::UiTheme, FeathersPlugins},
    prelude::*,
};

app.insert_resource(UiTheme(create_dark_theme()));
app.add_plugins((FeathersPlugins, CarbonPlugin));
```

Compose components with BSN. Scenes do not own game state; attach observers
that update the relevant model or navigation resource at the call site.

```rust
bsn! {
    @page_header(
        "Inventory",
        "Carried items",
        "Items currently carried by the party."
    )
    --
    @screen_tabs()
    Children [
        @screen_tab("Items", true)
        --
        @screen_tab("Equipment", false)
    ]
    --
    @panel(percent(100))
    Children [
        @section_header(CarbonIcon::Inventory, "Pack")
        --
        @item_tile_optional(CarbonIcon::Sword, "Iron longsword", None, false)
        --
        @action_button("Equip", CarbonIcon::Confirm, CarbonButtonTone::Primary)
        on(|_activate: On<Activate>| {
            // Send a game request or update navigation state here.
        })
    ]
}
```

## Components

- FeathersPlugins and UiTheme provide the shared widget controls and theme.
  CarbonTheme and CarbonPlugin retain game-specific palette tokens, hover-card
  timing, and selection projection.
- root and panel provide full-window and raised-surface layout.
- page_title and page_header provide plain titles or titles with contextual help;
  screen_tabs and screen_tab provide optional second-row screen navigation. Each tab carries a CarbonTab
  component; update its selected field to project active-tab styling.
- section_header, action_button, badge, eyebrow_text, muted_text, and
  key_hint provide common hierarchy, helper copy, and keyboard affordances.
- stat_bar displays a value and semantic progress fill.
- item_tile and item_tile_optional are square glyph-first controls;
  item_art_tile is a passive image-first tile for inventory presentation.
  equipment_slot provides a caption-free placeholder, while gear_slot creates
  labelled square equipment positions for character gear grids.
  Each tile exposes CarbonSelection so a screen can change its selected
  appearance without rebuilding the scene.
- reference_term opens a contextual information card after a 600 ms hover
  dwell.

All hierarchies are BSN scenes. Plugin systems only project interaction state
into existing components; they do not spawn or rebuild UI trees.

The crate exposes the current Feathers-backed game UI helpers and Carbon presentation API.

## Showcase

Run the component catalogue from the workspace root:

```sh
cargo run -p delta-slayers-ui --example carbon_showcase
```

The example includes working tab navigation, selectable inventory items,
updated item details, and visible use/drop feedback. Its responsive content
columns wrap when the window becomes too narrow for a side-by-side layout.
The game uses the same widgets for its inventory, equipment placeholders,
fast-travel choices, zone panels, modal overlays, loading screen, and
application placeholder screens.
