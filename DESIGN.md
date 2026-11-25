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
| `gameplay/player.rs` | Player model and tuning settings; stable façade for the player module API |
| `gameplay/player/camera.rs` | Follow camera, pan/zoom input, pitch, and camera settings behavior |
| `gameplay/player/movement.rs` | Cursor targeting, keyboard movement, and destination state |
| `gameplay/player/collision.rs` | Shared player/NPC collision resolution and authored movement blockers |
| `gameplay/player/destination.rs` | Movement-path and destination indicators |
| `gameplay/player/visual.rs` | Player billboard transform synchronization |
| `gameplay/spawning.rs` | Reflected spawn-point authoring types and initial player spawn selection |
| `gameplay/environment.rs` | Reflected object-spawn authoring types, Blender collection loading, and prototype catalog |
| `gameplay/zones.rs` | Town interaction zones, proximity detection, visited-zone data, and fast-travel requests |
| `gameplay/exploration.rs` | Exploration domain data and stable module façade |
| `gameplay/exploration/travel.rs` | Portal detection, entering/returning from maps, and travel anchors |
| `gameplay/exploration/generation.rs` | Seeded tile generation, reachability, biome tile selection, and capacity-limited object placement |
| `gameplay/exploration/terrain.rs` | Biome/noise sampling and smooth terrain elevation synchronization |
| `gameplay/exploration/hex.rs` | Axial coordinates, hex layout, and neighbor geometry |
| `gameplay/inventory.rs` | Authoritative inventory data |
| `gameplay/world.rs` | In-game world setup and the world-ready transition |
| `gameplay/npc.rs` | NPC movement that reuses the player collision resolver |
| `gameplay/systems.rs` | Gameplay actions such as fast travel and returning to the menu |
| `gameplay/persistence.rs` | Save-file serialization and loading |
| `gameplay/billboard.rs` | Camera-facing billboard transform |
| `gameplay/occlusion.rs` | Blender-authored player occluders and per-object material fading |
| `ui/stack.rs` | UI overlay navigation stack |
| `ui/components.rs` | UI marker components and UI-only requests |
| `ui/inventory.rs` | Inventory ViewModel, UI labels, and model-to-view projection |
| `ui/screens/` | BSN scene definitions for loading, gameplay, main menu, player settings, and placeholder screens |
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
- `MovementBlocker`: invisible oriented box that blocks the player's
  horizontal movement. Attach it to a Blender Empty; its transform defines a
  box with local extents of -1 to 1. Blockers are upright and may rotate around
  the vertical Y axis.
- `ExplorationPortal`: configured oriented interaction volume that generates
  a seeded, bounded hex wilderness. It exposes the map radius, rendered tile
  radius in meters, elevation scale, maximum jump height, terrain/biome
  frequency, and optional seed to Blender through Skein.
- `SpawnPoint`: spawn role and priority. The highest-priority Player point
  supplies the player's ground position and initial horizontal heading; saved
  player positions take precedence.
- `ObjectSpawner`: marks a tile prototype as eligible for procedural object
  placement. Its capacity rank supplies the tile's total size budget, and
  spawn chance controls successive placement attempts.
- `SpawnableObject`: marks a root object in a biome-specific or shared general
  objects collection as a spawnable prototype. Its size rank and rarity control
  capacity consumption and weighted selection; it has no interaction behavior.
- `Billboard`: camera-facing behavior for flat world objects. The player uses
  the existing full-body character image on an unlit, transparent 3D quad.
- `PlayerOccluder`: opt-in transparency for Blender-authored objects that cross the camera-to-player line of sight. The component applies to descendant meshes, each of which receives a private material copy while its opacity fades down and back up.
- `SquashOnMove`: opt-in movement animation for visual entities. It detects
  transform displacement and applies a subtle horizontal stretch / vertical
  squash to the sprite scale. It does not move the entity or alter gameplay
  position; large one-frame displacements are treated as teleports and ignored.

The player entity stores its position on the ground. Zone detection uses a
separate 1.7-unit interaction probe so existing Blender trigger volumes retain
their previous behavior. A separate perspective camera follows above and
behind the player from a fixed heading, initially offset from the player's
facing direction so the camera reveals the sides of world models. Holding the
middle mouse button and dragging pans the camera and its focus together across
the horizontal ground plane without changing camera height, rotating the view,
or changing the player's heading; dragging over UI does not pan the camera. The mouse wheel also does not zoom or tilt the camera while over UI.
By default, keyboard or click-to-move input clears the pan offset and recenters
the camera on the player. The auto-lock setting can be disabled to preserve an
independent free-camera view: player movement does not translate the camera.
The wheel still adjusts follow distance within fixed limits and vertical viewing
angle; tilt stays within the previous -0.97 to -0.67 radian range, and its scroll
speed is configurable. The mouse cursor remains visible
and ungrabbed during gameplay.
Left-clicking the ground sets a movement destination; the player walks toward
it continuously, and animated 3D arrows slowly travel along the remaining
trajectory; its dedicated gizmo depth bias keeps it visible through other
geometry. If collision-resolved click movement falls below 0.5 world units per
second, the destination and its trajectory are cancelled. Keyboard movement
remains available independently. Holding the left button for 300 ms starts periodic destination
updates from the cursor until release. A highlighted ground ring and vertical
beacon mark the endpoint. A new click replaces the destination, and holding
Shift increases movement speed. WASD and arrow keys also move relative to the
camera's horizontal heading; direct keyboard movement takes priority and clears
the clicked destination. Both movement modes share a circular player footprint
against Blender-authored `MovementBlocker` boxes. Movement advances in small
steps to prevent tunnelling, and collision correction keeps the player against
the wall while preserving tangential movement for wall sliding. Blockers affect
continuous movement, not spawn placement or fast-travel teleports. The inventory
does not pause gameplay: movement and world simulation continue behind it,
while camera scrolling is reserved for the inventory UI. Pause and the game menu still suspend gameplay. Right-click opens
the inventory or dismisses the active overlay.
Saved positions preserve their full world-space coordinates. Fast travel uses
the selected trigger box's world-space X/Z position and offsets the exit along
Z only. It preserves the player's current Y elevation; the trigger box's own
height never moves the player vertically. Billboard sprites track the camera's horizontal
bearing and partially follow its elevation while retaining an upright bias.
Visual entities tagged with `SquashOnMove` pulse with a small, smoothed
horizontal stretch and vertical compression while their transform is moving.
The animation restores the authored scale when idle and does not change the
entity's position or its gameplay state. The player image is
rendered on a vertical, camera-facing 3D quad. Godo is spawned from
`assets/npc/godo.png` as a billboard and continuously chases the player at a
slower speed, stopping at a small personal-space distance. NPC movement calls
the same collision resolver as player movement, so Blender-authored
`MovementBlocker` volumes block both. The town remains the existing
Blender-authored scene, while an `ExplorationPortal` can create a separate
procedural hex map at runtime. The generated tile heights position imported biome tile variants from
`assets/enviroment/biomes/{grassland,woodland,dryland,highlands}.glb`. Each
biome GLB contains named `tiles` and `objects` collections in the same scene.
The loader extracts tile roots only from `tiles` and biome-specific spawnable
object roots only from `objects`, keeping their hierarchies and Skein
components. Prototype extraction preserves the collection root's ancestor
chain and registers the render, transform, and hierarchy components required
by Bevy's reflected dynamic-world extraction (`Mesh3d`,
`MeshMaterial3d<StandardMaterial>`, `Transform`, `ChildOf`, and `Name`). Each
prototype's root translation is reset to the origin so Blender's staging
positions do not offset generated instances; root rotation, scale, and child
transforms remain intact. Tile meshes use a 5 m source radius and are scaled
to a fixed 6 m rendered radius. A
smoothstep-weighted triangular height field keeps player and NPC elevation
continuous between neighboring hex centers.
Neighboring tiles can be crossed only when their height difference is within
the configured maximum jump height; steeper edges receive movement blockers.
Reachability is flood-filled from the spawn tile, and tiles outside that
connected area carry the gameplay-only `UnreachableTile` marker for future
systems. The markers are also highlighted on the exploration minimap. Tiles
are tinted by biome, and Skein-authored components on tile variants are
retained.

The shared general object pool is loaded from
`assets/enviroment/general.glb`'s `objects` collection. Its root prototypes
are eligible in every biome, in addition to the current biome's object pool.
For example, woodland mushrooms belong in the woodland file's `objects`
collection, while a golden sword that can appear anywhere belongs in
`general.glb`. A tile variant with `ObjectSpawner` receives deterministic
placement attempts during map generation. Size ranks consume capacity units
(S = 1, M = 2, L = 3), so an L capacity tile supports one L object, one M
plus one S, or three S objects. Rarity weights are applied across both
eligible object pools. Generated objects currently have no interactions or separate gameplay behavior.
They receive invisible, size-rank-based movement blockers so the player and NPC
cannot walk through them. These are approximate upright box colliders, not
mesh-accurate physics. Procedural vegetation is not generated. A
collision-blocked perimeter and return beacon complete this exploration pass;
the beacon restores the town-side position captured on entry.

The TriggerZone-to-TriggerBox rename is a reflected authoring API break.
Existing Blender files must replace the old component using the refreshed
Skein type registry and re-export their GLB. The current `ZoneKind`, visited-zone
resource, and UI buttons assume one destination per kind. Add stable zone IDs
and data-driven destination UI before authoring multiple destinations of the
same kind. Enemy and NPC spawn point kinds are metadata only until their runtime
spawn systems are implemented.

The reflected `vegetation_density` field is retained as a legacy
compatibility field because existing Blender GLB exports may still contain it.
It is ignored by generation and has no setup control; procedural vegetation
remains disabled. The new `max_jump_height` field defaults when absent from
older exports, so existing exploration portals can continue to instantiate
without an immediate GLB re-export.

See `assets/worlds/README.md` for town-scene authoring and `assets/enviroment/README.md` for biome tile and object collections.


## Exploration maps

An exploration portal is an interaction volume, not an automatic touch trigger:
the player enters its bounds and presses E to open a setup dialog. The dialog
starts with the portal's authored values and exposes the seed, hex radius,
rendered tile radius, height multiplier, maximum jump height, terrain
frequency, and biome scale. A seed of 0 generates a new map per visit; **Randomize
seed** chooses a visible, reproducible seed without changing the other
parameters. Number inputs enforce the generator's safe runtime ranges, and a
live tile count reflects the selected radius. Reset restores the portal's
authored values; Generate submits the selected configuration to gameplay.
Tile radius defaults to 6 m. The supplied meshes have a 5 m source radius and
are scaled to the selected radius. Tile centers use the full rendered radius
for regular hex spacing. The player and NPC use a smoothstep-weighted
interpolation over the triangular grid between tile centers. Tile-to-tile
height differences up to the configured maximum jump height are traversable;
steeper edges are blocked. A flood fill from the spawn tile marks the other
tiles as unreachable with an `UnreachableTile` component, which is available
to future gameplay systems and highlighted on the minimap. The imported tile
models retain their vertical sides. No procedural vegetation is spawned.
While the dialog is open, UiStack owns its visibility and world simulation is
suspended. Debug builds also show a dismissible panel on map entry with the
resolved seed and effective generation parameters. The town scene stays loaded
while the generated map is placed in a distant world area. Its return beacon
restores the town-side transform. The current save format stores town state
only; using Backspace while exploring saves the return anchor so a subsequent
Quick Load starts back in town rather than at the generated map's world offset.

The generator is a first exploration pass, not the final ecosystem system. It
does not yet save map discovery, resource nodes, time progression, tribal
relations, or NPC populations.

## Persistence

The quick-save is stored in `save.txt` in the working directory. Version 2
stores the player's full world position, yaw, camera pitch, and visited zone
kinds. Versionless saves are treated as the previous format and have their
camera pitch migrated once when loaded. A valid save at world origin is still a
save; absence or invalid content is represented separately from its position.
Save read/write failures are logged rather than silently ignored.

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
- ExplorationSetup (player-facing portal configuration dialog; suspends gameplay)
- GameMenu (independent overlay, also accessible while inventory is open)

The gameplay cursor is always visible and ungrabbed. It is configured when
entering InGame rather than rewritten every frame. Opening an overlay does not
change cursor mode. Inventory is a non-pausing overlay: world
simulation and player movement continue behind it, but cursor-based movement
targeting and camera pan/zoom are blocked over the UI. Every interactive
gameplay overlay root exposes a `RelativeCursorPosition` hit region; this lets
the gameplay input systems ignore raw mouse input while the pointer is over a
visible UI surface. Clicks over UI do not set a destination, and a click that
begins over UI remains blocked until release. This prevents click-through when
an overlay closes during a held click.

System ordering is explicit:

1. UiSet::Input processes input and changes state.
2. Gameplay input runs after the UI input set and checks whether gameplay is
   allowed.
3. UiSet::Projection derives visibility and other presentation state from the
   current model.
4. The gameplay cursor is configured once on entering InGame.

This prevents stale visibility and avoids rewriting an unchanged cursor mode
every frame.

## In-game menu overlay

The in-game menu is a root-level BSN scene, independent of the inventory
hierarchy. Its Settings action opens live camera and movement tuning controls:
pan, zoom, and tilt scroll sensitivity; camera auto-lock while moving; walk
speed; and sprint speed. Camera distance limits are fixed rather than exposed
as settings. The same basic settings screen is available
from the main menu. Changes apply immediately and can be restored to
defaults. A collapsed Developer settings section exposes a toggle for the
exploration-generation diagnostics and a control to reset the free-camera pan
offset. These settings currently last for the application session. Escape opens it directly when no other overlay is active; Escape or
right click closes the topmost layer. The inventory no longer duplicates that
navigation with a separate Menu button. Its open state is represented by
UiLayer::GameMenu, and a projection system controls Visibility.

The main menu exposes Quick Load, New game, Settings, Load Save, Credits, and
Exit to Desktop. Its Settings screen shares the live camera and movement
controls with the in-game menu. Quick Load restores a valid save file when one exists; otherwise it
starts at the authored spawn point. New game always starts at the authored
spawn point. Load Save remains a placeholder until a save-selection interface
is implemented. All five main-menu actions use
the same neutral secondary-button style. Both start actions share the same
Loading transition and communicate their intent through GameStartMode.

## Visual hierarchy and Feathers-based controls

The interface should have a small number of deliberate attention anchors. Avoid giving every panel, control, label, and status the same visual weight. Use spacing and grouping before adding separators; reserve the strongest contrast and filled treatment for the current primary action or selected state. Keep secondary actions quieter, use semantic colours only where they communicate meaning, and avoid decorative borders around ordinary surfaces. Distinguish layered surfaces with restrained elevation and soft shadows rather than outlining every container. Use consistent styling for equivalent controls and keep related information close together. Hover references should be discoverable but should not compete with the main task until the user deliberately pauses over them. Use a hoverable help affordance only when the screen contains contextual information that is not already obvious from its visible content; otherwise use a plain page title. Explanatory copy and secondary status belong in the help card or page content. Put tabs for distinct views of the same screen directly beneath the title, and omit that row when no such views exist.

## Responsive layout rules

Game UI should adapt through Bevy UI flex layout rather than fixed screen assumptions:

- Full-window roots use zero minimum dimensions and allow wrapping where sibling
  panels may no longer fit side by side.
- Flexible columns use a small or zero minimum width and may shrink; avoid
  combining a large fixed basis with `flex_shrink: 0` unless the element is a
  deliberately fixed-size control.
- Inventory content may wrap into a vertical stack and its content region may
  scroll independently of the fixed header and actions.
- Modal and application panels use viewport-relative maximum dimensions,
  minimum-width guards, and scrolling when their content exceeds available
  height.
- Floating panels keep consistent viewport insets and must not assume a
  desktop-sized window.
- The minimap is player-oriented: the player's horizontal forward direction
  points toward its top edge, nearby markers rotate around the centered player
  marker; its orientation indicates the player's heading.
- Fixed-size artwork tiles and equipment slots remain fixed-size controls;
  their containing rows wrap instead of compressing the artwork.

## Reusable Feathers-based UI library

The workspace crate `delta-slayers-ui` contains game-specific BSN composition
helpers and presentation behaviour. The application enables Bevy 0.20.1's
`bevy_feathers` feature, installs `FeathersPlugins`, and initializes the
standard dark `UiTheme` once at startup.

- Shared action buttons are built with `FeathersButton`; the secondary screen
  tabs also use the Feathers button control and project their selected state
  through `ButtonVariant`.
- Shared panel surfaces use Feathers theme tokens and `ThemeContext`, while
  the Carbon palette remains available for game-specific art tiles, status
  treatments, minimap marks, and delayed hover cards.
- The exploration setup dialog uses `FeathersNumberInput` with hard limits,
  precision, and step sizes. Its values are synchronized with a UI ViewModel;
  only the Generate action sends a gameplay travel request.
- `CarbonPlugin` retains game-specific hover-card timing and selection
  projection. Its systems update existing BSN entities and do not spawn or
  rebuild UI trees.
- `page_header`, `section_header`, `badge`, `stat_bar`, `item_tile`,
  `item_art_tile`, `gear_slot`, `equipment_slot`, and the text helpers
  provide presentation patterns that Feathers does not attempt to define for
  this game's inventory and world screens.
- All hierarchies remain BSN scenes. UI observers mutate navigation or model
  request state; projection systems synchronize visibility and widget values.

Bevy documents Feathers as an experimental, editor-oriented collection rather
than a game UI framework. The project uses its ready-made controls and theme
system where they fit, with game-specific BSN composition retained around them.

The crate example is a standalone catalogue of the current visual language:

```sh
cargo run -p delta-slayers-ui --example carbon_showcase
```

The main menu, loading screen, application placeholders, inventory actions,
zone panels, interaction prompt, fast-travel panel, and modal actions use the
shared Feathers button control. Game-specific navigation and ViewModel
projection remain in `src/ui/`.

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
- UiStack owns overlay navigation.
- InventoryModel owns the current authoritative currency values. Item artwork
  and equipment positions in the current inventory screen are still static
  presentation placeholders, not persisted inventory data.
- InventoryViewModel owns the formatted currency labels projected into the
  inventory view.
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
