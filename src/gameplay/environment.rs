use std::collections::HashMap;

use bevy::{
    asset::LoadState,
    ecs::reflect::AppTypeRegistry,
    gltf::GltfAssetLabel,
    prelude::*,
    world_serialization::{DynamicWorld, DynamicWorldBuilder, WorldAsset},
};

use super::exploration::Biome;

const BIOME_TILE_FILES: [(Biome, &str); 4] = [
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

/// Metadata for an object prototype in enviroment/general.glb.
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
enum CollectionKind {
    BiomeTiles(Biome),
    GeneralObjects,
}

struct CollectionSource {
    kind: CollectionKind,
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

/// Loaded, split collection exports used by procedural world generation.
///
/// Each input GLB is a Blender Collection Exporter output. Its root objects
/// are extracted into individual reusable WorldAssets while their nested
/// hierarchy, mesh/material handles, and Skein components are preserved.
#[derive(Resource, Default)]
pub(crate) struct EnvironmentAssets {
    sources: Vec<CollectionSource>,
    tiles: HashMap<Biome, Vec<TilePrototype>>,
    objects: Vec<SpawnablePrototype>,
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
        remaining_capacity: u8,
        roll: f32,
    ) -> Option<&SpawnablePrototype> {
        choose_spawnable(&self.objects, remaining_capacity, roll)
    }
}

/// Starts loading one GLB per biome tile collection and the shared object
/// collection. These are Blender collection exports, not whole-level scenes.
pub(super) fn load_assets(
    mut environment: ResMut<EnvironmentAssets>,
    asset_server: Res<AssetServer>,
) {
    if !environment.sources.is_empty() {
        return;
    }

    for (biome, file_stem) in BIOME_TILE_FILES {
        let path = format!("enviroment/biomes/{file_stem}.glb");
        let handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset(path.clone()));
        environment.sources.push(CollectionSource {
            kind: CollectionKind::BiomeTiles(biome),
            path,
            handle,
            processed: false,
        });
    }

    environment.sources.push(CollectionSource {
        kind: CollectionKind::GeneralObjects,
        path: GENERAL_OBJECTS_FILE.to_owned(),
        handle: asset_server
            .load(GltfAssetLabel::Scene(0).from_asset(GENERAL_OBJECTS_FILE)),
        processed: false,
    });
}

/// Splits each loaded collection scene into individual root-object assets.
/// This lets generation choose one tile model without instantiating every
/// sibling object in the collection at every tile position.
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
                    warn!("Failed to load environment collection {path}: {error}");
                    environment.sources[index].processed = true;
                }
                continue;
            };
            extract_root_objects(collection, &type_registry, kind)
        };

        let root_count = extracted.len();
        for root in extracted {
            let Ok(asset) = WorldAsset::from_dynamic_world(&root.world, &type_registry) else {
                warn!("Could not split a root object from environment collection {path}");
                continue;
            };
            let handle = world_assets.add(asset);

            match kind {
                CollectionKind::BiomeTiles(biome) => {
                    environment
                        .tiles
                        .entry(biome)
                        .or_default()
                        .push(TilePrototype {
                            handle,
                            spawner: root.spawner,
                        });
                }
                CollectionKind::GeneralObjects => {
                    if let Some(definition) = root.spawnable {
                        environment.objects.push(SpawnablePrototype {
                            handle,
                            definition,
                        });
                    }
                }
            }
        }

        if root_count == 0 {
            warn!("Environment collection {path} contains no usable root objects");
        } else {
            info!("Loaded {root_count} root objects from {path}");
        }

        environment.sources[index].processed = true;
    }

    if environment.is_ready() {
        for (biome, _) in BIOME_TILE_FILES {
            if environment.tiles_for(biome).is_empty() {
                warn!("No tile root objects were loaded for biome {biome:?}");
            }
        }
        if environment.objects.is_empty() {
            warn!("No SpawnableObject prototypes were found in {GENERAL_OBJECTS_FILE}");
        }
    }
}

struct ExtractedRoot {
    world: DynamicWorld,
    spawner: Option<ObjectSpawner>,
    spawnable: Option<SpawnableObject>,
}

fn extract_root_objects(
    collection: &WorldAsset,
    type_registry: &bevy::reflect::TypeRegistry,
    kind: CollectionKind,
) -> Vec<ExtractedRoot> {
    let roots = collection
        .world
        .iter_entities()
        .filter(|entity| entity.get::<ChildOf>().is_none())
        .map(|entity| entity.id())
        .collect::<Vec<_>>();

    roots
        .into_iter()
        .filter_map(|root| {
            let spawner = collection.world.get::<ObjectSpawner>(root).copied();
            let spawnable = collection.world.get::<SpawnableObject>(root).copied();

            if matches!(kind, CollectionKind::GeneralObjects) && spawnable.is_none() {
                return None;
            }

            let mut hierarchy = Vec::new();
            collect_hierarchy_entities(&collection.world, root, &mut hierarchy);

            let world = DynamicWorldBuilder::from_world(&collection.world, type_registry)
                .extract_entities(hierarchy.into_iter())
                .remove_empty_entities()
                .build();

            Some(ExtractedRoot {
                world,
                spawner,
                spawnable,
            })
        })
        .collect()
}

fn collect_hierarchy_entities(world: &World, entity: Entity, output: &mut Vec<Entity>) {
    output.push(entity);
    if let Some(children) = world.get::<Children>(entity) {
        for child in children.iter() {
            collect_hierarchy_entities(world, child, output);
        }
    }
}

fn choose_spawnable<'a>(
    objects: &'a [SpawnablePrototype],
    remaining_capacity: u8,
    roll: f32,
) -> Option<&'a SpawnablePrototype> {
    let total_weight = objects
        .iter()
        .filter(|object| object.definition.size.units() <= remaining_capacity)
        .map(|object| object.definition.rarity.selection_weight())
        .sum::<f32>();

    if total_weight <= 0.0 || !total_weight.is_finite() {
        return None;
    }

    let mut target = roll.clamp(0.0, 1.0 - f32::EPSILON) * total_weight;
    for object in objects
        .iter()
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
