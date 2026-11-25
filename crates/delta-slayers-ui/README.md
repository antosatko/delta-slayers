# delta-slayers-ui

Game-specific Bevy UI components for Delta Slayers. The crate implements the
Carbon visual language used by the HTML prototype and provides a small set of
BSN scenes with shared presentation behaviour.

## Visual direction

Carbon uses a neutral graphite surface hierarchy with one restrained azure
accent. Ordinary navigation, metadata, icons, and item artwork remain neutral.
Azure is reserved for primary actions, selected states, links, and focus.
Positive, warning, and danger colors appear only where they communicate status,
using subtle tinted surfaces rather than bright fills.

Panels rely on spacing, tonal surfaces, and soft elevation instead of decorative
outlines. Item tiles are square and artwork-first, with the item name and an
optional quantity overlaid at the bottom. Screen headers contain a title and a
hoverable help affordance; screens with multiple views can place a tab row
directly underneath.

## Setup

Add the plugin once:

```rust
app.add_plugins(CarbonPlugin);
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
        @item_tile(CarbonIcon::Sword, "Iron longsword", None, false)
        --
        @action_button("Equip", CarbonIcon::Confirm, CarbonButtonTone::Primary)
        on(|_activate: On<Activate>| {
            // Send a game request or update navigation state here.
        })
    ]
}
```

## Components

- CarbonTheme and CarbonPlugin provide the shared palette and interaction
  behaviour.
- root and panel provide full-window and raised-surface layout.
- page_header, screen_tabs, and screen_tab provide title, contextual help,
  and optional second-row screen navigation.
- section_header, action_button, and badge provide common hierarchy and
  action/status treatments.
- stat_bar displays a value and semantic progress fill.
- item_tile is a square, artwork-first item control. Pass None for items
  without a stack quantity.
- reference_term opens a contextual information card after a 600 ms hover
  dwell.

All hierarchies are BSN scenes. Plugin systems only project interaction state
into existing components; they do not spawn or rebuild UI trees.

The old Slate type names remain as compatibility aliases, but new code should
use the Carbon names.

## Showcase

Run the component catalogue from the workspace root:

```sh
cargo run -p delta-slayers-ui --example carbon_showcase
```

The example interactions are demonstration-only and do not mutate game state.
