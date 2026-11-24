# Delta Slayers — Architecture and UI Design

## Goals

Delta Slayers is a small Bevy 0.20 prototype. Keep the architecture simple
enough for rapid iteration while making gameplay, presentation, and navigation
independently understandable.

All game UI is authored with Bevy Scene Notation (BSN). Rust systems may update
state and project it into components, but they must not construct UI trees
imperatively.

## Module boundaries

| Module | Responsibility |
| --- | --- |
| main.rs | Module declarations and plugin composition |
| state.rs | Top-level application states |
| game.rs | In-game world setup, save data, fast-travel execution |
| player.rs | Player movement and look |
| zone.rs | Zone model, proximity detection, visited-zone data |
| billboard.rs | Camera-facing billboard transform |
| ui_stack.rs | UI navigation stack and cursor-capture preference |
| ui.rs | BSN views, UI input, presentation projection |
| inventory.rs | Inventory domain model and inventory view model |
| menu.rs | Main-menu BSN view |
| transitions.rs | Placeholder screen BSN views |

Gameplay setup must not spawn UI. UI setup must not own gameplay-world entities.

## MVVM in an ECS

MVVM is a separation of responsibilities, not a framework layered on top of
Bevy:

- Model: gameplay/domain resources and components. Examples include
  InventoryModel, VisitedZones, NearbyZone, and FastTravelRequest.
- ViewModel: presentation-ready state derived from the model. InventoryViewModel
  holds display labels for currency values. Add item summaries, equipment
  labels, or sorting state here when a real feature needs them.
- View: BSN scene functions in ui.rs and menu.rs. A view describes hierarchy,
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

- UiStack owns overlay navigation and cursor preference.
- InventoryModel owns currency, item, and equipment data.
- InventoryViewModel owns derived display values.
- NearbyZone and VisitedZones are gameplay facts.
- FastTravelRequest is the command boundary from UI to gameplay. UI requests a
  destination; game.rs performs the teleport.
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
