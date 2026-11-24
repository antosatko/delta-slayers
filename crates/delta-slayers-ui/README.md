# delta-slayers-ui

Game-specific Bevy UI primitives that implement the Slate visual language used
by Delta Slayers.

## Use

Add the plugin once:

```rust
app.add_plugins(SlatePlugin);
```

Compose the components from BSN scenes:

```rust
bsn! {
    @panel(percent(100))
    Children [
        @section_header(SlateIcon::Inventory, "Carried items")
        --
        @item_tile(SlateIcon::Sword, "Iron longsword", "1 · EQUIPPED", true)
        --
        @action_button("Equip", SlateIcon::Confirm, SlateButtonTone::Primary)
        on(|_activate: On<Activate>| {
            // Send a game request or update navigation state here.
        })
    ]
}
```

The public surface includes:

- `SlateTheme` and `SlatePlugin` for palette and shared interaction behaviour.
- `root`, `panel`, `page_header`, and `section_header` for layout.
- `action_button`, `badge`, `stat_bar`, and `item_tile` for common controls.
- `reference_term` for delayed, hoverable reference cards.

All hierarchies are BSN scenes. The crate's systems only update the visual
state of existing controls; they do not spawn or rebuild UI entities.

## Showcase

Run the standalone component catalogue from the workspace root:

```sh
cargo run -p delta-slayers-ui --example slate_showcase
```

The example actions are demonstration-only and do not mutate game state.
