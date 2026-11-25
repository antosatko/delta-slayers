# Environment tile and object collections

These assets are exported from Blender collections with the glTF Collection
Exporter. Configure each collection to export a GLB and use Skein's **Trigger
all Collection Exporters** action when refreshing the asset set.

The loader expects one tile collection export per supported biome:

| Blender collection | Export path | Contents |
| --- | --- | --- |
| `tiles` | `assets/enviroment/biomes/grassland.glb` | All grassland tile variants |
| `tiles` | `assets/enviroment/biomes/woodland.glb` | All woodland tile variants |
| `tiles` | `assets/enviroment/biomes/dryland.glb` | All dryland tile variants |
| `tiles` | `assets/enviroment/biomes/highlands.glb` | All highland tile variants |
| `objects` | `assets/enviroment/general.glb` | Shared spawnable object prototypes |

The path spelling `enviroment` is intentional; it matches the requested asset
layout. The filename follows the biome name, for example
`assets/enviroment/biomes/lava.glb` for a future Lava biome export. The current
generator supports Grassland, Woodland, Dryland, and Highlands; adding another
biome also requires adding its gameplay classification and loader entry.

Each exported GLB is a standalone collection scene. All root objects in the
collection scene are loaded, then split into individual reusable world assets.
Keep each tile or spawnable model as a root object in its collection. The
object's children remain part of that model, so nested mesh objects are
supported. The collection itself is not a runtime object container.

## Tile variants

Put every interchangeable tile model in the matching biome's `tiles`
collection. Keep the source tile radius at 5 m; the generator scales it to
the configured hex radius. One root model is chosen deterministically for
each generated tile from the available variants in that biome's GLB.

Attach `delta_slayers::gameplay::environment::ObjectSpawner` to a tile root
that should be eligible for object placement.

| Field | Default | Meaning |
| --- | --- | --- |
| `capacity` | `Large` | Total size budget available on this tile |
| `spawn_chance` | `0.65` | Probability for each successive placement attempt, clamped to 0–1 |

Size ranks use these capacity units:

| Rank | Units |
| --- | ---: |
| `Small` (S) | 1 |
| `Medium` (M) | 2 |
| `Large` (L) | 3 |

The budget decreases by the size of each placed object. For example, an L
capacity tile can hold one L object, one M plus one S, or three S objects.
A tile without `ObjectSpawner` does not receive generated objects.

## Spawnable object prototypes

Place reusable object models in the shared Blender `objects` collection and
export it to `assets/enviroment/general.glb`. Attach
`delta_slayers::gameplay::environment::SpawnableObject` to each eligible root
object.

| Field | Default | Meaning |
| --- | --- | --- |
| `size` | `Small` | Capacity consumed when this object is placed |
| `rarity` | `Common` | Relative selection weight among objects that fit the remaining budget |

Rarity weights are relative, not guaranteed percentages: Common 100, Uncommon
35, Rare 12, Epic 4, and Legendary 1. Objects too large for the remaining
tile budget are excluded from that placement attempt.

Placement is seeded from the exploration map seed and tile coordinates, so
the same map seed produces the same tile variants and object placements. Each
placement receives a small deterministic offset around the tile center and
uses the terrain's interpolated surface height.

These objects are visual world content only at this stage. They do not expose
interaction prompts, collision behavior, or gameplay actions.

## Export checklist

- Configure each collection with a glTF Collection Exporter and the exact path
  shown above.
- Export as GLB (`.glb`) with mesh data and materials included.
- Attach Skein components to the root Blender objects, not only to mesh
  descendants.
- Keep the source object transforms and child hierarchy intact.
- Re-export the affected collection whenever its component values or models
  change.
