# Delta Slayers — Architecture and UI Design

## Goals

Delta Slayers is a small Bevy 0.20 prototype. Keep the architecture simple
enough for rapid iteration while making gameplay, presentation, and navigation
independently understandable.

All game UI is authored with Bevy Scene Notation (BSN). Rust systems may update
state and project it into components, but they must not construct UI trees
imperatively.

## Source layout and module boundaries

The source tree is organized around three ownership areas:

| Area | Modules | Responsibility |
| --- | --- | --- |
| Application | `app/` | Bevy app construction, startup configuration, and top-level `AppState` |
| Gameplay | `gameplay/` | Domain resources/components, world setup, simulation, and persistence |
| UI | `ui/` | Overlay navigation, BSN scenes, input, and presentation projection |
| UI library | `crates/delta-slayers-ui/` | Reusable Carbon theme, BSN widgets, and presentation-only behaviours |

Within those areas:

| Module | Responsibility |
| --- | --- |
| `app/mod.rs` | Application startup and plugin composition |
| `app/state.rs` | Top-level application states, including the Loading transition |
| `gameplay/player.rs` | Player component, movement, and look |
| `gameplay/spawning.rs` | Reflected spawn-point authoring types and initial player spawn selection |
| `gameplay/zones.rs` | Zone types, proximity detection, visited-zone data, and travel requests |
| `gameplay/inventory.rs` | Authoritative inventory data |
| `gameplay/world.rs` | In-game world setup and the world-ready transition |
| `gameplay/systems.rs` | Gameplay actions such as fast travel and returning to the menu |
| `gameplay/persistence.rs` | Save-file serialization and loading |
| `gameplay/billboard.rs` | Camera-facing billboard transform |
| `ui/stack.rs` | UI navigation stack and cursor-capture preference |
| `ui/components.rs` | UI marker components and UI-only requests |
| `ui/inventory.rs` | Inventory ViewModel, UI labels, and model-to-view projection |
| `ui/screens/` | BSN scene definitions for loading, gameplay, main menu, and placeholder screens |
| `ui/systems/input.rs` | UI input and navigation actions |
| `ui/systems/projection.rs` | Visibility, cursor, and minimap projection |
| `ui/mod.rs` | UI plugin composition and system ordering |
| `crates/delta-slayers-ui/` | Reusable Carbon UI crate; consumed by the game and its standalone showcase example |
| `gameplay/mod.rs` | Gameplay plugin composition and system ordering |

Keep the root `main.rs` limited to module declarations and starting the app.
Do not add unrelated root-level modules as the project grows; extend the
responsibility-focused area that owns the feature.

Gameplay setup must not spawn UI. UI setup must not own gameplay-world entities.

## Blender-authored world data

Bevy Skein is installed through `SkeinPlugin`. Gameplay components exposed to
Blender derive `Reflect`, reflect `Component` data, and are registered in the
app's type registry. Keep these types as stable authoring APIs: renaming a type,
field, or enum variant can invalidate data already stored in a `.blend` or
exported glTF file.

The main world export is `assets/worlds/town-map.glb`. Gameplay loads scene 0
through `WorldAssetRoot` during `AppState::Loading`. The loading screen is a
BSN scene owned by `ui/screens/loading.rs`; the `WorldInstanceReady` observer
advances the app to `AppState::InGame`. World entities are tagged to survive
the Loading-to-InGame transition and are cleaned up when leaving InGame. The
GLB is required; there is no procedural world fallback.

Current authored components:

- `TriggerBox`: interaction category and oriented box volume. The component
  belongs on a default Blender cube; its full world transform defines the
  bounds, with the default cube's local extents of -1 to 1 on each axis.
- `SpawnPoint`: spawn role and priority. The highest-priority Player point
  supplies the initial camera position and orientation; saved player positions
  take precedence.
- `Billboard`: camera-facing behavior for flat world objects.

The TriggerZone-to-TriggerBox rename is a reflected authoring API break.
Existing Blender files must replace the old component using the refreshed
Skein type registry and re-export their GLB. The current `ZoneKind`, visited-zone
resource, and UI buttons assume one destination per kind. Add stable zone IDs
and data-driven destination UI before authoring multiple destinations of the
same kind. Enemy and NPC spawn point kinds are metadata only until their runtime
spawn systems are implemented.

See `assets/worlds/README.md` for the Blender workflow and component schema.

## MVVM in an ECS

MVVM is a separation of responsibilities, not a framework layered on top of
Bevy:

- Model: gameplay/domain resources and components. Examples include
  InventoryModel, VisitedZones, NearbyZone, and FastTravelRequest.
- ViewModel: presentation-ready state derived from the model. InventoryViewModel
  holds display labels for currency values. Add item summaries, equipment
  labels, or sorting state here when a real feature needs them.
- View: BSN scene functions under ui/screens/. A view describes hierarchy,
  layout, styling, assets, and event bindings.

Data flow:

    User input / UI events
            |
            v
    UI actions mutate navigation or domain requests
            |
            v
    Model resources and components
            |
            v
    ViewModel projection systems
            |
            v
    BSN-created UI entities

Avoid storing the same fact in the model and view. Whether the inventory is
open belongs to UiStack; do not add a second InventoryOpen boolean.

## UI navigation and input

UiStack is the only authority for in-game overlay navigation. Layers are pushed
when opened and popped when closed. The newest layer is the topmost layer and
is therefore the first one closed by Escape or right click.

Current layers:

- Pause
- Panel(ZoneKind)
- Inventory
- GameMenu (independent overlay, also accessible while inventory is open)

Cursor capture preference is stored alongside navigation state. The effective
cursor mode is derived from whether the stack is empty and whether the player
requested capture. UI observers must not directly change CursorOptions.

System ordering is explicit:

1. UiSet::Input processes input and changes state.
2. Gameplay input runs after the UI input set and checks whether gameplay is
   allowed.
3. UiSet::Projection derives visibility and cursor settings from state.

This prevents stale visibility and multiple systems fighting over cursor state.

## In-game menu overlay

The in-game menu is a root-level BSN scene, independent of the inventory
hierarchy. Escape opens it directly when no other overlay is active; Escape or
right click closes the topmost layer. The inventory's Menu button opens the
same layer, preserving the inventory underneath it. Its open state is
represented by UiLayer::GameMenu, and a projection system controls Visibility.

## Visual hierarchy and Carbon UI principles

The interface should have a small number of deliberate attention anchors. Avoid giving every panel, control, label, and status the same visual weight. Use spacing and grouping before adding separators; reserve the strongest contrast and filled treatment for the current primary action or selected state. Keep secondary actions quieter, use semantic colours only where they communicate meaning, and avoid decorative borders around ordinary surfaces. Distinguish layered surfaces with restrained elevation and soft shadows rather than outlining every container. Use consistent styling for equivalent controls and keep related information close together. Hover references should be discoverable but should not compete with the main task until the user deliberately pauses over them. Screen headers show only the page title and a hoverable help affordance; explanatory copy and secondary status belong in the help card or page content. Put tabs for distinct views of the same screen directly beneath the title, and omit that row when no such views exist.

## Reusable Carbon UI library

The workspace crate `delta-slayers-ui` contains the reusable presentation
layer. It is deliberately game-specific rather than a generic widget framework.

- `CarbonPlugin` registers the `CarbonTheme` resource and presentation systems
  for hover states and delayed help/reference cards.
- `CarbonTheme` is the shared palette source. It uses graphite surface steps,
  neutral text and icon colors, a restrained azure accent, and semantic status
  colors with subtle tinted backgrounds.
- `panel`, `page_header`, `screen_tabs`, `screen_tab`, `section_header`,
  `action_button`, `badge`, `stat_bar`, and `item_tile` are composable BSN
  scenes. Item tiles are square, artwork-first controls with optional quantity.
  Each screen tab carries a `CarbonTab` selection component, and each item
  tile exposes `CarbonSelection`; screens can update either state without
  rebuilding the scene.
- `reference_term` is a hoverable text control whose information card opens
  after a 600 ms dwell. The page header uses the same delayed-card behaviour for
  its help affordance.
- All controls are declared with BSN. Library systems only project interaction
  state into existing components; they do not create or rebuild UI trees.

The crate example is a standalone catalogue of the current visual language:

```sh
cargo run -p delta-slayers-ui --example carbon_showcase
```

The game imports the crate through the workspace path dependency. The main
menu is the first integrated screen; game-specific overlay navigation and
ViewModel projection remain in `src/ui/`.

## BSN conventions

- Define every UI entity hierarchy with bsn! or bsn_list!.
- Use current Bevy 0.20 syntax: prefix scene inclusions with @ and separate
  sibling entities with --.
- Compose repeated elements using scene functions such as @gear_slot("Head").
- Keep event observers next to the control they act on using on(...).
- Observers should emit requests or mutate model/navigation resources. They
  should not mutate presentation components.
- Systems should not spawn, despawn, or rebuild UI trees as a substitute for
  state projection.
- Do not use deprecated children! or imperative child-spawning patterns for UI.

## State ownership

- `AppState` owns top-level application navigation, including Loading.
- UiStack owns overlay navigation and cursor preference.
- InventoryModel owns currency, item, and equipment data.
- InventoryViewModel owns derived display values.
- NearbyZone and VisitedZones are gameplay facts.
- FastTravelRequest is the command boundary from UI to gameplay. UI requests a
  destination; gameplay systems perform the teleport.
- BSN views own layout and styling, never business rules.

## Adding a feature

1. Identify the authoritative model/resource for the new fact.
2. Add an event, request, or state transition for user intent.
3. Add a ViewModel projection only when presentation needs derived values.
4. Add a BSN scene function for the visual structure.
5. Bind events with on(...); keep observers short and focused.
6. If it is an overlay, add a UiLayer and derive visibility from UiStack.
7. Add focused tests for state transitions and data projection.
8. Run cargo fmt --check, cargo check, and cargo test.

## Deliberately out of scope

Keep the shared UI crate intentionally small and game-specific. Do not add a
generic styling DSL, message bus abstraction, service locator, or a large
plugin hierarchy without concrete pressure for one. Use Bevy resources,
observers, system sets, and BSN scenes directly.
