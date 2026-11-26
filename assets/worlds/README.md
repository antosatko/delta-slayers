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
| `delta_slayers::gameplay::pickups::Pickup` | Makes an object interactable; configure its inventory action, item, finite or infinite stock, and cooldown. |
| `delta_slayers::gameplay::spawning::SpawnPoint` | Spawn location. Choose `kind`; for Player points, a higher `priority` wins. |
| `delta_slayers::gameplay::environment::ObjectSpawner` | Makes a tile model eligible for generated objects. Set its capacity rank and spawn chance. |
| `delta_slayers::gameplay::environment::SpawnableObject` | Marks a root object in a biome-specific or shared general objects collection as a spawnable prototype. Set its size rank and rarity. |
| `delta_slayers::gameplay::billboard::Billboard` | Makes a flat object face the player camera horizontally. |
| `delta_slayers::gameplay::billboard::SquashOnMove` | Adds subtle movement-driven squashing to an entity's visual scale. |
| `delta_slayers::gameplay::occlusion::PlayerOccluder` | Makes an object's meshes translucent when the object blocks the camera's view of the player. |

### Pickups

Attach `Pickup` to the root Blender Object or Empty that represents the
interactable object. Keep the component on the root so the complete model
hierarchy is removed when a finite pickup runs out.

| Field | Default | Meaning |
| --- | --- | --- |
| `action` | `AddToInventory` | Action performed on interaction. This is the only action currently available. |
| `item` | `Gold` | Inventory stack to increase: `Gold`, `Wood`, or `Stone`. |
| `quantity` | `1` | Remaining stock. Each successful interaction grants one item. |
| `infinite` | `false` | Keeps the pickup available without consuming its finite quantity. |
| `cooldown` | `0.5` | Minimum seconds between successful interactions. |

Approach the object and press **E** or use the interaction prompt. For
mesh pickups, interaction distance is measured from the closest point on the
transformed mesh bounds, not the Blender object origin. Marker-only pickups
fall back to their object origin. The horizontal range is 1.75 m and the
vertical range is 2.25 m from the player's upper-body interaction point. A finite pickup is despawned after its final
item is collected. An infinite pickup remains in the world and can be used
again after its cooldown. A quantity of zero makes a finite pickup unavailable.

The current inventory item choices map to the Gold, Wood, and Stone counters
in the inventory's Resources panel. Inventory contents are currently session
state and are not included in the save file.

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
| `seed` | empty | any `u64` | A fixed value reproduces the same terrain and biome layout. Empty chooses a new seed on each visit. |
| `radius` | 12 | 8–32 | Hex radius; the map contains `1 + 3r(r + 1)` tiles. |
| `tile_size` | 6.0 | fixed 6.0 | Legacy serialized field; runtime always uses a 6 m rendered tile radius. The supplied models have a 5 m source radius and are scaled to 6 m. |
| `height_multiplier` | 2.2 | 0–5 | Vertical scale of the terrain. Set to zero for flat terrain. |
| `terrain_frequency` | 0.045 | 0.01–0.15 | Higher values create smaller, more frequent elevation features. |
| `biome_scale` | 0.028 | 0.006–0.12 | Controls the size of biome regions. |
| `max_jump_height` | 1.5 | 0.25–3.5 | Maximum height difference the player can jump between adjacent tiles. |

Tiles are hexagonal cylinders with a fixed rendered radius of 6 m. The
legacy `tile_size` field remains in the reflected component so existing
Blender exports still load, but its authored value is ignored. Tile models use
a 5 m source radius and are scaled to 6 m; tile centers use that same 6 m
radius for regular hex spacing. The player and NPC ground height
uses smoothstep-weighted interpolation over the triangular grid between tile
centers. Adjacent tiles can be crossed only when their height difference does
not exceed `max_jump_height`; steeper edges receive movement blockers. A
flood fill from the spawn tile marks tiles outside the reachable region with
the `UnreachableTile` gameplay component. The exploration minimap highlights
those tiles for later use by gameplay systems.

Values outside the runtime ranges are clamped. The reflected
`vegetation_density` field is retained only for compatibility with older
GLB exports; it is ignored and does not generate vegetation. The new
`max_jump_height` field defaults to 1.5 m when absent from an older export.
You can refresh Skein's registry and re-export the portal to configure the
jump limit explicitly, but this is not required just to keep an existing
portal working. For a reproducible test map,
set a seed explicitly; leave it empty when each visit should be different.
In the setup dialog, **Randomize seed** chooses a fresh explicit seed while
preserving the other settings; seed 0 continues to generate a new seed on each
map entry. In debug builds, entering the map opens a dismissible on-screen
readout of the actual seed and effective generation parameters. Biome tile variants and biome-specific spawnable objects are loaded from
the named collection hierarchy described in [the environment asset workflow]
(../enviroment/README.md). Each biome has one GLB containing two collections:
`tiles` supplies the available tile model roots and `objects` supplies that
biome's eligible spawnable objects. The loader splits those collection roots
into separate reusable prototypes while preserving Skein components and
nested mesh hierarchies. Tile models are scaled from a 5 m source radius to
the fixed 6 m tile radius and arranged at regular hex spacing. Grassland,
woodland, dryland, and highland regions control tile tinting.

The shared `objects` collection in `assets/enviroment/general.glb` supplies
prototypes available in every biome. For example, mushrooms can be stored in
woodland's object collection while a golden sword can live in the shared
general collection. Tiles carrying `ObjectSpawner` choose from the current
biome's object pool plus the general pool. The tile's S/M/L capacity is
consumed by each generated object (S = 1, M = 2, L = 3); rarity affects
weighted selection across both pools. Spawned objects do not receive pickup behavior automatically; attach
`Pickup` to an eligible root object to make its generated instances
interactable. The map has a movement-blocked outer boundary and cliff-edge
blockers where the jump limit is exceeded. This is a first exploration
prototype: it does not yet persist discovered state or add resource
gathering, ecology simulation, or NPC populations.

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

### Player occluders

Add `PlayerOccluder` to a Blender Object with visible geometry. The marker
also applies to mesh descendants, so it can be placed on a parent object that
contains several mesh parts. The game tests the transformed world-space bounds
of those meshes against the segment from the gameplay camera to the player's
visual center. Objects that intersect this segment fade toward the configured
opacity; they fade back to their original material appearance when the player
is no longer behind them.

The reflected `occluded_opacity` field defaults to `0.35` (35% opacity).
Values range from `0.0` (fully transparent) to `1.0` (fully opaque). The value
is an opacity multiplier for the original material's base-color alpha.
Materials are copied per mesh before fading, so other objects that share the
same imported material are not affected. The fade is visual only; it does not
change collision, visibility, or gameplay queries.

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
- Prefer object-level components for these gameplay markers. Use `PlayerOccluder`
  on visible mesh objects or their parent object; it traverses descendant meshes.
  Skein also supports mesh and material components, but those are not needed here.
- Keep the root scene hierarchy and transforms intact; gameplay queries use
  world-space transforms so nested objects are supported.

### Migrating existing files

`TriggerZone` was replaced by `TriggerBox`. Fetch the updated remote type
registry in Skein, remove old TriggerZone components from the Blender objects,
and add TriggerBox components to the intended cube objects. Re-export the GLB.
