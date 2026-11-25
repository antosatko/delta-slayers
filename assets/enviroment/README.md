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

Attach `delta_slayers::gameplay::environment::ObjectSpawner` to a tile root
or to a marker Empty anywhere below that root. The first spawner found in
root-first hierarchy order supplies the tile's capacity and spawn chance.

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
| `size` | `Small` | Capacity consumed when this object is placed |
| `rarity` | `Common` | Relative selection weight among objects that fit the remaining budget |

Rarity weights are relative, not guaranteed percentages: Common 100, Uncommon
35, Rare 12, Epic 4, and Legendary 1. Objects too large for the remaining
tile budget are excluded from that placement attempt. Biome-specific and
general objects participate in the same weighted selection.

Placement is seeded from the exploration map seed and tile coordinates, so
the same map seed produces the same tile variants and object placements. Each
placement receives a small deterministic offset around the tile center and
uses the terrain's interpolated surface height.

Generated objects have invisible movement blockers. The blocker uses a
conservative upright box sized by the object's rank: Small 0.5 m, Medium
1.4 m, and Large 2.0 m horizontal half-extents. These values cover the
current small cone and medium Suzanne prototype bounds. The player and NPC
cannot walk through these blockers, but the shape is approximate rather than
mesh-accurate. Objects do not yet expose interaction prompts or gameplay
actions.

## Export checklist

- Export one GLB per biome with both `tiles` and `objects` collections in the
  same scene.
- Export `general.glb` with its shared `objects` collection.
- Enable **Full Collection Hierarchy** so the collection names survive as
  named nodes in the exported GLB.
- Export as GLB (`.glb`) with mesh data, materials, and Skein custom
  properties included.
- Attach `ObjectSpawner` and `SpawnableObject` to the relevant root objects.
- Keep source object transforms and child hierarchy intact.
- Re-export the affected GLB whenever its models or component values change.
