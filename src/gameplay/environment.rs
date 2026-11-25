use std::collections::HashMap;

use bevy::{
    asset::LoadState,
    ecs::reflect::AppTypeRegistry,
    gltf::GltfAssetLabel,
    prelude::*,
    world_serialization::{DynamicWorld, DynamicWorldBuilder, WorldAsset},
};

use super::exploration::Biome;

const BIOME_FILES: [(Biome, &str); 4] = [
    (Biome::Grassland, "grassland"),
    (Biome::Woodland, "woodland"),
    (Biome::Dryland, "dryland"),
    (Biome::Highlands, "highlands"),
];

const GENERAL_OBJECTS_FILE: &str = "enviroment/general.glb";

/// Relative size and capacity rank used by environment object placement.
///
/// The numeric budget is intentionally small and discrete: S = 1, M = 2,
/// L = 3. A tile with L capacity can hold one L, one M plus one S, or three S.
#[derive(Reflect, Default, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(Default)]
#[type_path = "delta_slayers::gameplay::environment"]
pub enum ObjectSizeRank {
    #[default]
    Small,
    Medium,
    Large,
}

impl ObjectSizeRank {
    pub(crate) const fn units(self) -> u8 {
        match self {
            Self::Small => 1,
            Self::Medium => 2,
            Self::Large => 3,
        }
    }

    pub(crate) const fn from_units(units: u8) -> Option<Self> {
        match units {
            1 => Some(Self::Small),
            2 => Some(Self::Medium),
            3 => Some(Self::Large),
            _ => None,
        }
    }
}

/// Weighted spawn frequency. Rarer objects remain possible when a tile has
/// enough remaining capacity, but common objects are selected more often.
#[derive(Reflect, Default, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(Default)]
#[type_path = "delta_slayers::gameplay::environment"]
pub enum ObjectRarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl ObjectRarity {
    fn selection_weight(self) -> f32 {
        match self {
            Self::Common => 100.0,
            Self::Uncommon => 35.0,
            Self::Rare => 12.0,
            Self::Epic => 4.0,
            Self::Legendary => 1.0,
        }
    }
}

/// Metadata for an object prototype in a biome's objects collection or in
/// the shared general objects collection.
///
/// Attach this component to the root Blender object that should be eligible
/// for procedural placement. It does not add interaction behavior.
#[derive(Component, Reflect, Default, Clone, Copy, Debug)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::environment"]
pub struct SpawnableObject {
    pub size: ObjectSizeRank,
    pub rarity: ObjectRarity,
}

/// Marks a tile prototype as eligible to receive generated objects.
///
/// capacity is the complete size budget for the tile. Each spawned object
/// consumes its own size rank from that budget.
#[derive(Component, Reflect, Clone, Copy, Debug)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::environment"]
pub struct ObjectSpawner {
    pub capacity: ObjectSizeRank,
    /// Chance for each successive placement attempt, from 0 to 1.
    pub spawn_chance: f32,
}

impl Default for ObjectSpawner {
    fn default() -> Self {
        Self {
            capacity: ObjectSizeRank::Large,
            spawn_chance: 0.65,
        }
    }
}

impl ObjectSpawner {
    pub(crate) fn effective_spawn_chance(self) -> f32 {
        if self.spawn_chance.is_finite() {
            self.spawn_chance.clamp(0.0, 1.0)
        } else {
            Self::default().spawn_chance
        }
    }
}

/// Runtime remainder of a tile's authored object-spawn capacity.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct RemainingObjectCapacity(pub(crate) Option<ObjectSizeRank>);

#[derive(Clone, Copy)]
enum AssetSourceKind {
    Biome(Biome),
    General,
}

struct AssetSource {
    kind: AssetSourceKind,
    path: String,
    handle: Handle<WorldAsset>,
    processed: bool,
}

pub(crate) struct TilePrototype {
    pub(crate) handle: Handle<WorldAsset>,
    pub(crate) spawner: Option<ObjectSpawner>,
}

pub(crate) struct SpawnablePrototype {
    pub(crate) handle: Handle<WorldAsset>,
    pub(crate) definition: SpawnableObject,
}

/// Loaded prototype pools extracted from named Blender collection nodes.
///
/// Each biome GLB provides both its tile variants and its biome-specific
/// spawnable objects. The shared general GLB contributes a second, global
/// object pool that is eligible in every biome.
#[derive(Resource, Default)]
pub(crate) struct EnvironmentAssets {
    sources: Vec<AssetSource>,
    tiles: HashMap<Biome, Vec<TilePrototype>>,
    biome_objects: HashMap<Biome, Vec<SpawnablePrototype>>,
    general_objects: Vec<SpawnablePrototype>,
}

impl EnvironmentAssets {
    pub(crate) fn is_ready(&self) -> bool {
        !self.sources.is_empty() && self.sources.iter().all(|source| source.processed)
    }

    pub(crate) fn tiles_for(&self, biome: Biome) -> &[TilePrototype] {
        self.tiles.get(&biome).map_or(&[], Vec::as_slice)
    }

    pub(crate) fn choose_spawnable(
        &self,
        biome: Biome,
        remaining_capacity: u8,
        roll: f32,
    ) -> Option<&SpawnablePrototype> {
        let biome_objects = self
            .biome_objects
            .get(&biome)
            .map_or(&[][..], Vec::as_slice);

        choose_spawnable(
            biome_objects,
            &self.general_objects,
            remaining_capacity,
            roll,
        )
    }
}

/// Starts loading one GLB per biome and the shared general-object GLB.
/// Each biome GLB contains named tiles and objects collections.
pub(super) fn load_assets(
    mut environment: ResMut<EnvironmentAssets>,
    asset_server: Res<AssetServer>,
) {
    if !environment.sources.is_empty() {
        return;
    }

    for (biome, file_stem) in BIOME_FILES {
        let path = format!("enviroment/biomes/{file_stem}.glb");
        let handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset(path.clone()));
        environment.sources.push(AssetSource {
            kind: AssetSourceKind::Biome(biome),
            path,
            handle,
            processed: false,
        });
    }

    environment.sources.push(AssetSource {
        kind: AssetSourceKind::General,
        path: GENERAL_OBJECTS_FILE.to_owned(),
        handle: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(GENERAL_OBJECTS_FILE)),
        processed: false,
    });
}

/// Extracts tile and object prototypes from the named collection subtrees.
///
/// Blender's glTF exporter must preserve the collection hierarchy so the
/// tiles and objects collection names are present as empty root nodes.
pub(super) fn collect_assets(
    mut environment: ResMut<EnvironmentAssets>,
    mut world_assets: ResMut<Assets<WorldAsset>>,
    app_type_registry: Res<AppTypeRegistry>,
    asset_server: Res<AssetServer>,
) {
    let type_registry = app_type_registry.read();

    for index in 0..environment.sources.len() {
        if environment.sources[index].processed {
            continue;
        }

        let handle = environment.sources[index].handle.clone();
        let kind = environment.sources[index].kind;
        let path = environment.sources[index].path.clone();

        let extracted = {
            let Some(collection) = world_assets.get(&handle) else {
                if let Some(LoadState::Failed(error)) = asset_server.get_load_state(&handle) {
                    warn!("Failed to load environment asset {path}: {error}");
                    environment.sources[index].processed = true;
                }
                continue;
            };

            match kind {
                AssetSourceKind::Biome(_) => (
                    extract_collection_roots(
                        collection,
                        &type_registry,
                        "tiles",
                        PrototypeKind::Tile,
                    ),
                    extract_collection_roots(
                        collection,
                        &type_registry,
                        "objects",
                        PrototypeKind::Spawnable,
                    ),
                ),
                AssetSourceKind::General => (
                    Vec::new(),
                    extract_collection_roots(
                        collection,
                        &type_registry,
                        "objects",
                        PrototypeKind::Spawnable,
                    ),
                ),
            }
        };

        let tile_roots = extracted.0;
        let object_roots = extracted.1;
        let mut tile_count = 0;
        let mut tile_mesh_count = 0;
        for root in tile_roots {
            let Ok(mut asset) = WorldAsset::from_dynamic_world(&root.world, &type_registry) else {
                warn!("Could not create a tile prototype from {path}");
                continue;
            };
            if !normalize_prototype_root(&mut asset, root.root_name.as_ref(), &path) {
                continue;
            }
            let mesh_count = count_meshes(&asset.world);
            if mesh_count == 0 {
                warn!(
                    "Tile prototype from {path} lost all Mesh3d components during extraction"
                );
                continue;
            }

            let handle = world_assets.add(asset);
            if let AssetSourceKind::Biome(biome) = kind {
                environment
                    .tiles
                    .entry(biome)
                    .or_default()
                    .push(TilePrototype {
                        handle,
                        spawner: root.spawner,
                    });
                tile_count += 1;
                tile_mesh_count += mesh_count;
            }
        }

        let mut object_count = 0;
        let mut object_mesh_count = 0;
        for root in object_roots {
            let Some(definition) = root.spawnable else {
                continue;
            };
            let Ok(mut asset) = WorldAsset::from_dynamic_world(&root.world, &type_registry) else {
                warn!("Could not create a spawnable object prototype from {path}");
                continue;
            };
            if !normalize_prototype_root(&mut asset, root.root_name.as_ref(), &path) {
                continue;
            }
            let mesh_count = count_meshes(&asset.world);
            if mesh_count == 0 {
                warn!(
                    "Spawnable object prototype from {path} lost all Mesh3d components during extraction"
                );
                continue;
            }

            let handle = world_assets.add(asset);
            let prototype = SpawnablePrototype { handle, definition };

            match kind {
                AssetSourceKind::Biome(biome) => {
                    environment
                        .biome_objects
                        .entry(biome)
                        .or_default()
                        .push(prototype);
                }
                AssetSourceKind::General => environment.general_objects.push(prototype),
            }
            object_count += 1;
            object_mesh_count += mesh_count;
        }

        if tile_count == 0 && matches!(kind, AssetSourceKind::Biome(_)) {
            warn!("No tile prototypes were found in the tiles collection of {path}");
        }
        if object_count == 0 {
            warn!("No SpawnableObject prototypes were found in the objects collection of {path}");
        }

        info!(
            "Loaded {tile_count} tile variants ({tile_mesh_count} meshes) and {object_count} object prototypes ({object_mesh_count} meshes) from {path}"
        );
        environment.sources[index].processed = true;
    }

    if environment.is_ready() {
        for (biome, _) in BIOME_FILES {
            if environment.tiles_for(biome).is_empty() {
                warn!("No tile variants were loaded for biome {biome:?}");
            }
        }
    }
}

fn normalize_prototype_root(
    asset: &mut WorldAsset,
    root_name: Option<&Name>,
    path: &str,
) -> bool {
    let Some(root_name) = root_name else {
        warn!("Cannot normalize an unnamed prototype root from {path}");
        return false;
    };
    let Some(root_entity) = asset
        .world
        .iter_entities()
        .find(|entity| entity.get::<Name>() == Some(root_name))
        .map(|entity| entity.id())
    else {
        warn!("Cannot find prototype root {root_name} in extracted asset {path}");
        return false;
    };

    if let Some(mut transform) = asset.world.get_mut::<Transform>(root_entity) {
        transform.translation = Vec3::ZERO;
    }

    true
}

fn count_meshes(world: &World) -> usize {
    world
        .iter_entities()
        .filter(|entity| entity.get::<Mesh3d>().is_some())
        .count()
}

#[derive(Clone, Copy)]
enum PrototypeKind {
    Tile,
    Spawnable,
}

struct ExtractedRoot {
    world: DynamicWorld,
    root_name: Option<Name>,
    spawner: Option<ObjectSpawner>,
    spawnable: Option<SpawnableObject>,
}

/// Extracts direct object roots from a named Blender collection. The
/// collection container is included in each extracted world to preserve its
/// transform while other sibling objects remain outside that prototype.
fn extract_collection_roots(
    source: &WorldAsset,
    type_registry: &bevy::reflect::TypeRegistry,
    collection_name: &str,
    prototype_kind: PrototypeKind,
) -> Vec<ExtractedRoot> {
    let collection_root = source
        .world
        .iter_entities()
        .find(|entity| {
            entity
                .get::<Name>()
                .is_some_and(|name| name.as_str() == collection_name)
                && entity.get::<Children>().is_some()
        })
        .map(|entity| entity.id());

    let Some(collection_root) = collection_root else {
        warn!("GLB is missing the named {collection_name} collection node");
        return Vec::new();
    };

    let roots = source
        .world
        .get::<Children>(collection_root)
        .map(|children| children.iter().collect::<Vec<_>>())
        .unwrap_or_default();

    roots
        .into_iter()
        .filter_map(|root| {
            if matches!(prototype_kind, PrototypeKind::Tile)
                && !hierarchy_contains_mesh(&source.world, root)
            {
                warn!(
                    "Ignoring a tile entry in {collection_name} because its hierarchy contains no mesh"
                );
                return None;
            }

            let root_name = source.world.get::<Name>(root).cloned();
            let spawner = find_object_spawner(&source.world, root);
            let spawnable = source.world.get::<SpawnableObject>(root).copied();

            if matches!(prototype_kind, PrototypeKind::Spawnable) && spawnable.is_none() {
                return None;
            }

            // Preserve the collection root's ancestors as well. Its ChildOf
            // reference must not point outside the extracted WorldAsset.
            let mut hierarchy = Vec::new();
            let mut ancestor = Some(collection_root);
            while let Some(entity) = ancestor {
                hierarchy.push(entity);
                ancestor = source.world.get::<ChildOf>(entity).map(ChildOf::parent);
            }
            hierarchy.reverse();
            collect_hierarchy_entities(&source.world, root, &mut hierarchy);

            let world = DynamicWorldBuilder::from_world(&source.world, type_registry)
                .extract_entities(hierarchy.into_iter())
                .remove_empty_entities()
                .build();

            Some(ExtractedRoot {
                world,
                root_name,
                spawner,
                spawnable,
            })
        })
        .collect()
}

fn hierarchy_contains_mesh(world: &World, entity: Entity) -> bool {
    world.get::<Mesh3d>(entity).is_some()
        || world
            .get::<Children>(entity)
            .is_some_and(|children| {
                children
                    .iter()
                    .any(|child| hierarchy_contains_mesh(world, child))
            })
}

fn find_object_spawner(world: &World, entity: Entity) -> Option<ObjectSpawner> {
    world.get::<ObjectSpawner>(entity).copied().or_else(|| {
        world
            .get::<Children>(entity)?
            .iter()
            .find_map(|child| find_object_spawner(world, child))
    })
}

fn collect_hierarchy_entities(world: &World, entity: Entity, output: &mut Vec<Entity>) {
    output.push(entity);
    if let Some(children) = world.get::<Children>(entity) {
        for child in children.iter() {
            collect_hierarchy_entities(world, child, output);
        }
    }
}

/// Selects from the current biome's object prototypes and the shared general
/// prototypes. Objects that exceed the remaining tile capacity are excluded.
fn choose_spawnable<'a>(
    biome_objects: &'a [SpawnablePrototype],
    general_objects: &'a [SpawnablePrototype],
    remaining_capacity: u8,
    roll: f32,
) -> Option<&'a SpawnablePrototype> {
    let total_weight = biome_objects
        .iter()
        .chain(general_objects.iter())
        .filter(|object| object.definition.size.units() <= remaining_capacity)
        .map(|object| object.definition.rarity.selection_weight())
        .sum::<f32>();

    if total_weight <= 0.0 || !total_weight.is_finite() {
        return None;
    }

    let mut target = roll.clamp(0.0, 1.0 - f32::EPSILON) * total_weight;
    for object in biome_objects
        .iter()
        .chain(general_objects.iter())
        .filter(|object| object.definition.size.units() <= remaining_capacity)
    {
        let weight = object.definition.rarity.selection_weight();
        if target < weight {
            return Some(object);
        }
        target -= weight;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_rank_maps_to_a_decrementing_capacity_budget() {
        assert_eq!(ObjectSizeRank::Small.units(), 1);
        assert_eq!(ObjectSizeRank::Medium.units(), 2);
        assert_eq!(ObjectSizeRank::Large.units(), 3);

        assert_eq!(ObjectSizeRank::from_units(3), Some(ObjectSizeRank::Large));
        assert_eq!(ObjectSizeRank::from_units(2), Some(ObjectSizeRank::Medium));
        assert_eq!(ObjectSizeRank::from_units(1), Some(ObjectSizeRank::Small));
        assert_eq!(ObjectSizeRank::from_units(0), None);
    }

    #[test]
    fn default_spawner_has_large_capacity_and_a_bounded_spawn_chance() {
        let spawner = ObjectSpawner::default();

        assert_eq!(spawner.capacity, ObjectSizeRank::Large);
        assert!((0.0..=1.0).contains(&spawner.effective_spawn_chance()));
    }

    #[test]
    fn prototype_root_translation_is_reset_without_changing_rotation_or_scale() {
        let rotation = Quat::from_rotation_y(0.7);
        let scale = Vec3::new(1.2, 0.8, 1.4);
        let mut world = World::new();
        let root = world
            .spawn((
                Name::new("Prototype"),
                Transform {
                    translation: Vec3::new(12.0, 3.0, -11.0),
                    rotation,
                    scale,
                },
            ))
            .id();
        let mut asset = WorldAsset::new(world);

        assert!(normalize_prototype_root(
            &mut asset,
            Some(&Name::new("Prototype")),
            "test.glb"
        ));

        let transform = asset.world.get::<Transform>(root).unwrap();
        assert_eq!(transform.translation, Vec3::ZERO);
        assert_eq!(transform.rotation, rotation);
        assert_eq!(transform.scale, scale);
    }

    #[test]
    fn invalid_spawn_chance_uses_the_default() {
        let spawner = ObjectSpawner {
            spawn_chance: f32::NAN,
            ..default()
        };

        assert_eq!(
            spawner.effective_spawn_chance(),
            ObjectSpawner::default().spawn_chance
        );
    }
}
