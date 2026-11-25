# Biome environment assets

Each biome is exported as a single GLB containing two Blender collections:
`tiles` and `objects`. This keeps the terrain variants and that biome's
spawnable object pool together. A separate `general.glb` supplies objects
that may appear in any biome.

```text
assets/enviroment/
├── biomes/
│   ├── grassland.glb   # tiles + objects
│   ├── woodland.glb    # tiles + objects
│   ├── dryland.glb     # tiles + objects
│   └── highlands.glb   # tiles + objects
└── general.glb         # objects
```

The path spelling `enviroment` is intentional; it matches the runtime asset
paths. The same filename convention can be used for future biomes, for
example `assets/enviroment/biomes/lava.glb`. The current generator supports
Grassland, Woodland, Dryland, and Highlands; a new biome also needs a
gameplay biome type and a loader entry.

## Collection hierarchy and export

Use the normal glTF Binary scene export for each file, not one Collection
Exporter output per collection. Keep the Blender scene hierarchy intact and
enable the glTF export option **Data > Scene Graph > Full Collection
Hierarchy**. This exports collections as named empty nodes, allowing the
loader to find the `tiles` and `objects` collection roots in the same scene.

For each biome file:

- Put all tile model roots in the collection named `tiles`.
- Put all biome-specific spawnable object roots in the collection named
  `objects`.

For `general.glb`, put shared spawnable object roots in its `objects`
collection. Do not put a second `tiles` collection in that file.

The loader reads the direct object roots inside each named collection, then
splits them into reusable prototypes while keeping each object's descendants
and Skein components together. Keep the component on the root object of the
model, not just on one of its mesh children.

## Tile variants

Put every interchangeable tile model in the matching biome file's `tiles`
collection. Keep the source tile radius at 5 m; the generator scales it to
the fixed rendered hex radius of 6 m. The selected object's root location is
treated as collection staging placement and reset to the origin when creating
a prototype; preserve the model's geometry relative to that object origin.
Root rotation, scale, and descendant transforms are retained. One root model
is chosen deterministically for each generated tile from the variants in that
biome's GLB.

Attach `delta_slayers::gameplay::environment::ObjectSpawner` to the tile root
or to an Empty child in the tile model. Every marked entity is one independent
spawn point. Its full transform relative to the tile root determines the
object's position and orientation. A marker on the root itself places an
object at the tile origin.

| Field | Default | Meaning |
| --- | --- | --- |
| `capacity` | `Large` | Largest object size this point can host |
| `spawn_chance` | `0.65` | Probability this point receives one object, clamped to 0–1 |

A point accepts any object whose rank is no larger than its capacity. Each
point receives at most one object; capacity is not shared between points.

The size ranks are ordered `Small` (S), `Medium` (M), and `Large` (L).
A tile without any `ObjectSpawner` markers does not receive generated
objects. Marker-only Empty nodes are read as authoring data and removed from
the instantiated tile hierarchy; mesh-bearing transform parents remain.

## Spawnable object prototypes

Attach `delta_slayers::gameplay::environment::SpawnableObject` to each
eligible root object in a biome's `objects` collection or in the shared
`general.glb` `objects` collection.

Objects in a biome file are available only when generating tiles of that
biome. For example, woodland mushrooms belong in
`biomes/woodland.glb`'s `objects` collection. Objects in `general.glb` are
available in every biome; a golden sword or other biome-independent prop
belongs there.

| Field | Default | Meaning |
| --- | --- | --- |
| `size` | `Small` | Rank used when matching an object to a spawn point |
| `rarity` | `Common` | Relative selection weight among objects allowed by that point |

Rarity weights are relative, not guaranteed percentages: Common 100, Uncommon
35, Rare 12, Epic 4, and Legendary 1. At each point, objects larger than its
capacity are excluded. Biome-specific and general objects participate in the
same weighted selection.

Placement is reproducible from the exploration map seed, tile coordinates,
and sorted spawn-point position. The object's position and orientation come
from the marker rather than a generated random offset. The complete mesh
hierarchy is used to compute an aggregate AABB, including descendant mesh
transforms. An invisible upright movement blocker uses those bounds with a
small padding, so the player and NPC cannot walk through the visible object.
The blocker is conservative rather than mesh-accurate physics. Objects do not
yet expose interaction prompts or gameplay actions.

## Export checklist

- Export one GLB per biome with both `tiles` and `objects` collections in the
  same scene.
- Export `general.glb` with its shared `objects` collection.
- Enable **Full Collection Hierarchy** so the collection names survive as
  named nodes in the exported GLB.
- Export as GLB (`.glb`) with mesh data, materials, and Skein custom
  properties included.
- Attach `SpawnableObject` to each object root in the object collection.
- Attach `ObjectSpawner` to each intended point on the tile root or an Empty
  child; do not place it on the scene or collection container.
- Keep model transforms and mesh hierarchy intact. Marker-only Empty nodes
  are consumed as spawn metadata and removed from runtime tile instances.
- Re-export the affected GLB whenever its models or component values change.
