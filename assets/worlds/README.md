# Blender-authored world

Export the main Blender scene to:

```text
assets/worlds/town-map.glb
```

The game loads scene 0 from this file while `AppState::Loading` is active. The
loading screen remains visible until Bevy reports that the world instance is
ready, then the app transitions to `AppState::InGame`. The local GLB is
required; there is no procedural fallback.

## Authoring components

Start the game in a debug build, open Blender's Skein panel, and run **Fetch a
Remote Type Registry**. The Bevy app must be running while fetching the registry.
If the component API changes, fetch the registry again.

Add these components to Blender Objects (or Empties where appropriate):

| Component type path | Use |
| --- | --- |
| `delta_slayers::gameplay::zones::TriggerBox` | Oriented interaction / fast-travel volume. Choose `kind`; the object's transform defines its bounds. |
| `delta_slayers::gameplay::exploration::ExplorationPortal` | Opens a seeded procedural hex wilderness. Its fields configure the generated map; the object's transform defines the interaction volume. |
| `delta_slayers::gameplay::player::MovementBlocker` | Invisible oriented box that blocks player movement; the object's transform defines its bounds. |
| `delta_slayers::gameplay::spawning::SpawnPoint` | Spawn location. Choose `kind`; for Player points, a higher `priority` wins. |
| `delta_slayers::gameplay::billboard::Billboard` | Makes a flat object face the player camera horizontally. |
| `delta_slayers::gameplay::billboard::SquashOnMove` | Adds subtle movement-driven squashing to an entity's visual scale. |

### Trigger boxes

Create a default Blender cube and attach a TriggerBox component to the cube
object. Resize it in Object Mode to define the volume; its location, rotation,
and scale are all respected, including when nested under other transformed
objects. Keep the cube's scale unapplied so it is exported as a node transform.
The default cube's local bounds are -1 to 1 on each axis, so a scale of 1
produces a 2-by-2-by-2 world-unit trigger.

Give the cube a transparent material so it does not obstruct the level view.
The trigger test uses the transformed cube bounds, not the rendered material.
Available kinds are `Shop` and `Guild`, which map to the existing UI panels
and fast-travel buttons. Fast travel uses the trigger's world-space X/Z center,
but keeps the player's current Y elevation regardless of the trigger's height.

The current UI and visited-zone model treats each kind as one destination. The
component API can be extended with stable zone IDs before adding multiple
shops or guilds.

### Exploration portals

Add `ExplorationPortal` to a default Blender cube. Its transformed bounds
define the interaction volume, using the same local -1 to 1 bounds as
`TriggerBox`. Place the cube around the player's upper body; for a ground-level
portal, a cube centered about 1 unit above the ground with its default 1-unit
half-height will contain the 1.7-unit interaction probe. Enter the volume and
press **E** to generate and enter a procedural exploration map. The generated
map is placed in a separate world area while the town remains loaded. A return beacon is created near the map's
center; enter its volume and press **E** to return to the town.

The portal exposes these settings:

| Field | Default | Runtime range | Effect |
| --- | ---: | ---: | --- |
| `seed` | empty | any `u64` | A fixed value reproduces the same terrain and decoration. Empty chooses a new seed on each visit. |
| `radius` | 12 | 8–32 | Hex radius; the map contains `1 + 3r(r + 1)` tiles. |
| `tile_size` | 1.5 | 1–3 | Side length of each hex tile in world units. |
| `height_multiplier` | 2.2 | 0–5 | Vertical scale of the terrain. Set to zero for flat terrain. |
| `terrain_frequency` | 0.045 | 0.01–0.15 | Higher values create smaller, more frequent elevation features. |
| `biome_scale` | 0.028 | 0.006–0.12 | Controls the size of biome regions. |
| `vegetation_density` | 0.28 | 0–0.8 | Chance of placing a tree on eligible woodland tiles. |

Values outside the runtime ranges are clamped. For a reproducible test map,
set a seed explicitly; leave it empty when each visit should be different.
In debug builds, entering the map opens a dismissible on-screen readout of the
actual seed and effective generation parameters. The generated terrain has grassland, woodland, dryland, and highland regions,
with procedural height variation, subtle per-hex color variation, sparse trees
and rocks, and a movement-blocked outer boundary. The map is a first
exploration prototype: it does not yet persist its discovered state or add
resource gathering, ecology simulation, or NPC populations.

### Invisible movement walls

Add `MovementBlocker` to a Blender Empty. The Empty's world transform defines
an invisible box with local bounds from -1 to 1 on each axis; use its location,
rotation, and scale to place and size the wall. For example, make the Empty
thin on X (or Z) and long along the wall. Rotate it around the vertical Y axis
to align it with angled walls. Keep the wall upright; pitch and roll are not
supported.

The player has a circular horizontal footprint with a radius of 0.32 world
units and a height of 1.6 units. Extend the blocker vertically far enough to
overlap that body. Walls only block horizontal movement; the player can slide
along their edges. Both click-to-move and WASD / arrow movement respect them.
Spawn points and fast-travel teleports intentionally bypass wall collision.

Use an Empty rather than a visible mesh, so the blocker is not rendered. For
editing, Blender's Empty display shape and size can be adjusted independently
of its transform scale.

### Spawn points

Place a SpawnPoint on an Empty at the desired player start location, at ground
level. Its world-space position becomes the player's ground position; its
horizontal rotation sets the initial camera heading. The gameplay camera follows
above and behind the player, and its pitch is controlled independently.
If several Player points exist, the one with the highest priority is selected;
avoid tied priorities.

`Enemy` and `Npc` remain reserved spawn-point kinds. Godo is currently spawned
by gameplay setup at a fixed offset from the player's initial position; NPC
spawn points are not yet consumed.

### Moving sprite animation

Add `SquashOnMove` alongside `Billboard` to a moving sprite object that
should squash slightly as its transform changes. The animation affects only
the visual X/Y scale, returns to the authored scale when movement stops, and
ignores large one-frame displacements such as teleports.

### Export notes

- Export the scene as glTF Binary (`.glb`) and keep it at the exact path above.
- Export Skein component extras with the scene.
- Keep gameplay tags on the Blender Object / Empty whose transform represents
  the zone or spawn point.
- Prefer object-level components for these gameplay markers. Skein also
  supports mesh and material components, but those are not needed here.
- Keep the root scene hierarchy and transforms intact; gameplay queries use
  world-space transforms so nested objects are supported.

### Migrating existing files

`TriggerZone` was replaced by `TriggerBox`. Fetch the updated remote type
registry in Skein, remove old TriggerZone components from the Blender objects,
and add TriggerBox components to the intended cube objects. Re-export the GLB.
