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
| UI | `ui/` | Overlay navigation, BSN scenes, UI input, and presentation projection |

Within those areas:

| Module | Responsibility |
| --- | --- |
| `app/mod.rs` | Application startup and plugin composition |
| `app/state.rs` | Top-level application states |
| `gameplay/player.rs` | Player component, movement, and look |
| `gameplay/spawning.rs` | Reflected spawn-point authoring types and initial player spawn selection |
| `gameplay/zones.rs` | Zone types, proximity detection, visited-zone data, and travel requests |
| `gameplay/inventory.rs` | Authoritative inventory data |
| `gameplay/world.rs` | In-game world and player setup |
| `gameplay/systems.rs` | Gameplay actions such as fast travel and returning to the menu |
| `gameplay/persistence.rs` | Save-file serialization and loading |
| `gameplay/billboard.rs` | Camera-facing billboard transform |
| `ui/stack.rs` | UI navigation stack and cursor-capture preference |
| `ui/components.rs` | UI marker components and UI-only requests |
| `ui/inventory.rs` | Inventory ViewModel, UI labels, and model-to-view projection |
| `ui/screens/` | BSN scene definitions for gameplay, main menu, and placeholder screens |
| `ui/systems/input.rs` | UI input and navigation actions |
| `ui/systems/projection.rs` | Visibility, cursor, and minimap projection |
| `ui/mod.rs` | UI plugin composition and system ordering |
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

The main world export is `assets/worlds/overworld.glb`. If it exists, gameplay
loads scene 0 through `WorldAssetRoot`; otherwise, the procedural test world
is retained. The world-ready event gates initial spawn-point selection so the
player does not query incomplete glTF hierarchies.

Current authored components:

- `TriggerZone`: interaction category and spherical radius. Detection uses the
  component-bearing entity's world-space origin.
- `SpawnPoint`: spawn role and priority. The highest-priority Player point
  supplies the initial camera position and orientation; saved player positions
  take precedence.
- `Billboard`: camera-facing behavior for flat world objects.

The current `ZoneKind`, visited-zone resource, and UI buttons assume one
destination per kind. Add stable zone IDs and data-driven destination UI before
authoring multiple destinations of the same kind. Enemy and NPC spawn point
kinds are metadata only until their runtime spawn systems are implemented.

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
- InventoryMenu (nested above inventory)

Cursor capture preference is stored alongside navigation state. The effective
cursor mode is derived from whether the stack is empty and whether the player
requested capture. UI observers must not directly change CursorOptions.

System ordering is explicit:

1. UiSet::Input processes input and changes state.
2. Gameplay input runs after the UI input set and checks whether gameplay is
   allowed.
3. UiSet::Projection derives visibility and cursor settings from state.

This prevents stale visibility and multiple systems fighting over cursor state.

## Inventory menu layout

The inventory menu overlay is a sibling of the inventory top bar and content,
inside the inventory root. It must not be parented under the 32-pixel top-bar
node: a 600-pixel modal needs the inventory root as its containing block.
This keeps layout, z-order, and hit-testing predictable.

The menu open state is represented by UiLayer::InventoryMenu. The menu button,
Escape, and right click all use the same navigation stack. Button observers
only change state; projection systems control Visibility.

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

- `AppState` owns top-level application navigation.
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

Do not introduce a generic UI framework, message bus abstraction, service
locator, or large plugin hierarchy without concrete pressure for one. Use Bevy
resources, observers, system sets, and BSN scenes directly.
