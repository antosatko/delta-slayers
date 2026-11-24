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
| `delta_slayers::gameplay::spawning::SpawnPoint` | Spawn location. Choose `kind`; for Player points, a higher `priority` wins. |
| `delta_slayers::gameplay::billboard::Billboard` | Makes a flat object face the player camera horizontally. |

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
and fast-travel buttons.

The current UI and visited-zone model treats each kind as one destination. The
component API can be extended with stable zone IDs before adding multiple
shops or guilds.

### Spawn points

Place a SpawnPoint on an Empty at the desired player start location. Its
world-space position is used, with the camera raised 1.7 units along the
point's local up axis. Its world rotation sets the initial camera orientation.
If several Player points exist, the one with the highest priority is selected;
avoid tied priorities.

`Enemy` and `Npc` are reserved spawn-point kinds for future systems. They are
available as authoring metadata but do not spawn entities yet.

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
