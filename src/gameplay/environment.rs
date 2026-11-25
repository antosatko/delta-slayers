use std::collections::HashMap;

use bevy::{
    asset::LoadState,
    camera::primitives::MeshAabb,
    ecs::reflect::AppTypeRegistry,
    gltf::GltfAssetLabel,
    prelude::*,
    world_serialization::{DynamicWorld, DynamicWorldBuilder, WorldAsset},
};

use super::{exploration::Biome, player::MovementBlocker};

const BIOME_FILES: [(Biome, &str); 4] = [
    (Biome::Grassland, "grassland"),
    (Biome::Woodland, "woodland"),
    (Biome::Dryland, "dryland"),
    (Biome::Highlands, "highlands"),
];

const GENERAL_OBJECTS_FILE: &str = "enviroment/general.glb";

/// Relative object size used by authored spawn points.
///
/// S = 1, M = 2, and L = 3. A spawn point's capacity is the largest size
/// rank that it can host.
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
}

/// Weighted spawn frequency. Rarer objects remain possible at points whose
/// maximum size allows them, but common objects are selected more often.
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
/// for placement. Its size rank is compared with the maximum size allowed by
/// each tile's authored spawn point. It does not add interaction behavior.
#[derive(Component, Reflect, Default, Clone, Copy, Debug)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::environment"]
pub struct SpawnableObject {
    pub size: ObjectSizeRank,
    pub rarity: ObjectRarity,
}

/// Marks one authored object spawn point on a tile prototype.
///
/// Attach it to the tile root for a center spawn point, or to an Empty child
/// to place a point elsewhere on the tile. Each marked entity is an
/// independent point; capacity is the largest object size that may use it.
#[derive(Component, Reflect, Clone, Copy, Debug)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::environment"]
pub struct ObjectSpawner {
    /// Largest object size that can use this point.
    pub capacity: ObjectSizeRank,
    /// Probability that an object is placed at this point, from 0 to 1.
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

#[derive(Clone, Copy, Debug)]
pub(crate) struct ObjectSpawnPoint {
    pub(crate) position: Vec3,
    pub(crate) rotation: Quat,
    pub(crate) spawner: ObjectSpawner,
}

pub(crate) struct TilePrototype {
    pub(crate) handle: Handle<WorldAsset>,
    pub(crate) spawn_points: Vec<ObjectSpawnPoint>,
}

pub(crate) struct SpawnablePrototype {
    pub(crate) handle: Handle<WorldAsset>,
    pub(crate) definition: SpawnableObject,
    pub(crate) mesh_bottom: f32,
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
        max_size_units: u8,
        roll: f32,
    ) -> Option<&SpawnablePrototype> {
        let biome_objects = self
            .biome_objects
            .get(&biome)
            .map_or(&[][..], Vec::as_slice);

        choose_spawnable(
            biome_objects,
            &self.general_objects,
            max_size_units,
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
    meshes: Res<Assets<Mesh>>,
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
        let mut tile_spawn_point_count = 0;
        let mut removed_authoring_nodes = 0;
        for root in tile_roots {
            let Ok(mut asset) = WorldAsset::from_dynamic_world(&root.world, &type_registry) else {
                warn!("Could not create a tile prototype from {path}");
                continue;
            };
            let Some(root_entity) =
                normalize_prototype_root(&mut asset, root.root_name.as_ref(), &path)
            else {
                continue;
            };
            let spawn_points = collect_object_spawn_points(&asset.world, root_entity);
            let mesh_count = count_meshes(&asset.world);
            if mesh_count == 0 {
                warn!(
                    "Tile prototype from {path} lost all Mesh3d components during extraction"
                );
                continue;
            }

            removed_authoring_nodes += strip_prototype_markers(&mut asset.world, root_entity);
            let handle = world_assets.add(asset);
            if let AssetSourceKind::Biome(biome) = kind {
                environment
                    .tiles
                    .entry(biome)
                    .or_default()
                    .push(TilePrototype {
                        handle,
                        spawn_points: spawn_points.clone(),
                    });
                tile_count += 1;
                tile_mesh_count += mesh_count;
                tile_spawn_point_count += spawn_points.len();
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
            let Some(root_entity) =
                normalize_prototype_root(&mut asset, root.root_name.as_ref(), &path)
            else {
                continue;
            };
            let Some(mesh_bottom) = prototype_mesh_bottom(&asset.world, &meshes) else {
                warn!("Could not calculate the mesh bottom for spawnable object in {path}");
                continue;
            };
            let mesh_count = count_meshes(&asset.world);
            if mesh_count == 0 {
                warn!(
                    "Spawnable object prototype from {path} lost all Mesh3d components during extraction"
                );
                continue;
            }

            removed_authoring_nodes += strip_prototype_markers(&mut asset.world, root_entity);
            let handle = world_assets.add(asset);
            let prototype = SpawnablePrototype {
                handle,
                definition,
                mesh_bottom,
            };

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
            "Loaded {tile_count} tile variants ({tile_mesh_count} meshes, {tile_spawn_point_count} authored object spawn points) and {object_count} object prototypes ({object_mesh_count} meshes); removed {removed_authoring_nodes} authoring-only nodes from {path}"
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
) -> Option<Entity> {
    let Some(root_name) = root_name else {
        warn!("Cannot normalize an unnamed prototype root from {path}");
        return None;
    };
    let Some(root_entity) = asset
        .world
        .iter_entities()
        .find(|entity| entity.get::<Name>() == Some(root_name))
        .map(|entity| entity.id())
    else {
        warn!("Cannot find prototype root {root_name} in extracted asset {path}");
        return None;
    };

    if let Some(mut transform) = asset.world.get_mut::<Transform>(root_entity) {
        transform.translation = Vec3::ZERO;
    }

    let mut ancestors = Vec::new();
    let mut ancestor = asset.world.get::<ChildOf>(root_entity).map(ChildOf::parent);
    while let Some(entity) = ancestor {
        ancestors.push(entity);
        ancestor = asset.world.get::<ChildOf>(entity).map(ChildOf::parent);
    }
    asset.world.entity_mut(root_entity).remove::<ChildOf>();
    for entity in ancestors {
        let _ = asset.world.despawn(entity);
    }

    Some(root_entity)
}

fn asset_world_transform(world: &World, entity: Entity) -> GlobalTransform {
    let mut ancestry = Vec::new();
    let mut current = Some(entity);

    while let Some(entity) = current {
        ancestry.push(entity);
        current = world.get::<ChildOf>(entity).map(ChildOf::parent);
    }

    let mut transform = GlobalTransform::IDENTITY;
    for ancestor in ancestry.into_iter().rev() {
        let local = world.get::<Transform>(ancestor).copied().unwrap_or_default();
        transform = transform.mul_transform(local);
    }

    transform
}

/// Reads spawn markers only from the selected prototype subtree. Collection
/// and scene metadata must never become additional object spawn points.
fn collect_object_spawn_points(
    world: &World,
    prototype_root: Entity,
) -> Vec<ObjectSpawnPoint> {
    let mut pending = vec![(prototype_root, GlobalTransform::IDENTITY)];
    let mut points = Vec::new();

    while let Some((entity, parent_transform)) = pending.pop() {
        let local = world.get::<Transform>(entity).copied().unwrap_or_default();
        let transform = parent_transform.mul_transform(local);

        if let Some(spawner) = world.get::<ObjectSpawner>(entity).copied() {
            let world_transform = transform.compute_transform();
            points.push(ObjectSpawnPoint {
                position: world_transform.translation,
                rotation: world_transform.rotation,
                spawner,
            });
        }

        if let Some(children) = world.get::<Children>(entity) {
            pending.extend(children.iter().map(|child| (child, transform)));
        }
    }

    // Stable ordering keeps seed-based placement independent of entity IDs.
    points.sort_by(|a, b| {
        a.position
            .x
            .total_cmp(&b.position.x)
            .then(a.position.y.total_cmp(&b.position.y))
            .then(a.position.z.total_cmp(&b.position.z))
            .then(a.rotation.x.total_cmp(&b.rotation.x))
            .then(a.rotation.y.total_cmp(&b.rotation.y))
            .then(a.rotation.z.total_cmp(&b.rotation.z))
            .then(a.rotation.w.total_cmp(&b.rotation.w))
            .then(a.spawner.capacity.units().cmp(&b.spawner.capacity.units()))
            .then(a.spawner.spawn_chance.total_cmp(&b.spawner.spawn_chance))
    });

    points
}

/// Finds the lowest mesh point after applying the full descendant transform
/// hierarchy. This is used only to place spawned object geometry on terrain.
fn prototype_mesh_bottom(world: &World, meshes: &Assets<Mesh>) -> Option<f32> {
    let mut minimum_y = f32::INFINITY;
    let mut found_mesh = false;

    for entity in world.iter_entities() {
        let Some(mesh_3d) = entity.get::<Mesh3d>() else {
            continue;
        };
        let Some(mesh) = meshes.get(&mesh_3d.0) else {
            continue;
        };
        let Some(aabb) = mesh.get_aabb() else {
            continue;
        };

        let center = Vec3::new(aabb.center.x, aabb.center.y, aabb.center.z);
        let half_extents = Vec3::new(
            aabb.half_extents.x,
            aabb.half_extents.y,
            aabb.half_extents.z,
        );
        let transform = asset_world_transform(world, entity.id());

        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [-1.0, 1.0] {
                    let corner = center + half_extents * Vec3::new(x, y, z);
                    minimum_y = minimum_y.min(transform.transform_point(corner).y);
                }
            }
        }

        found_mesh = true;
    }

    (found_mesh && minimum_y.is_finite()).then_some(minimum_y)
}

/// Removes spawn metadata and unrelated Empty-only branches while retaining
/// rendered meshes and explicitly authored movement blockers.
fn strip_prototype_markers(world: &mut World, root: Entity) -> usize {
    let entities = world
        .iter_entities()
        .map(|entity| entity.id())
        .collect::<Vec<_>>();

    for entity in entities {
        world.entity_mut(entity).remove::<ObjectSpawner>();
        world.entity_mut(entity).remove::<SpawnableObject>();
    }

    prune_non_rendering_descendants(world, root)
}

fn prune_non_rendering_descendants(world: &mut World, parent: Entity) -> usize {
    let children = world
        .get::<Children>(parent)
        .map(|children| children.iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let mut removed = 0;

    for child in children {
        if hierarchy_contains_runtime_content(world, child) {
            removed += prune_non_rendering_descendants(world, child);
        } else {
            removed += despawn_subtree(world, child);
        }
    }

    removed
}

fn despawn_subtree(world: &mut World, entity: Entity) -> usize {
    let children = world
        .get::<Children>(entity)
        .map(|children| children.iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let mut removed = 1;

    for child in children {
        removed += despawn_subtree(world, child);
    }

    let _ = world.despawn(entity);
    removed
}

fn hierarchy_contains_runtime_content(world: &World, entity: Entity) -> bool {
    world.get::<Mesh3d>(entity).is_some()
        || world.get::<MovementBlocker>(entity).is_some()
        || world.get::<Children>(entity).is_some_and(|children| {
            children
                .iter()
                .any(|child| hierarchy_contains_runtime_content(world, child))
        })
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

fn collect_hierarchy_entities(world: &World, entity: Entity, output: &mut Vec<Entity>) {
    output.push(entity);
    if let Some(children) = world.get::<Children>(entity) {
        for child in children.iter() {
            collect_hierarchy_entities(world, child, output);
        }
    }
}

/// Selects from the current biome's object prototypes and the shared general
/// prototypes. Objects larger than a spawn point's maximum size are excluded.
fn choose_spawnable<'a>(
    biome_objects: &'a [SpawnablePrototype],
    general_objects: &'a [SpawnablePrototype],
    max_size_units: u8,
    roll: f32,
) -> Option<&'a SpawnablePrototype> {
    let total_weight = biome_objects
        .iter()
        .chain(general_objects.iter())
        .filter(|object| object.definition.size.units() <= max_size_units)
        .map(|object| object.definition.rarity.selection_weight())
        .sum::<f32>();

    if total_weight <= 0.0 || !total_weight.is_finite() {
        return None;
    }

    let mut target = roll.clamp(0.0, 1.0 - f32::EPSILON) * total_weight;
    for object in biome_objects
        .iter()
        .chain(general_objects.iter())
        .filter(|object| object.definition.size.units() <= max_size_units)
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
    fn size_ranks_have_increasing_order() {
        assert_eq!(ObjectSizeRank::Small.units(), 1);
        assert_eq!(ObjectSizeRank::Medium.units(), 2);
        assert_eq!(ObjectSizeRank::Large.units(), 3);
    }

    #[test]
    fn all_authored_spawn_points_keep_nested_transforms_and_ignore_ancestors() {
        let mut world = World::new();
        let scene_root = world
            .spawn((Transform::IDENTITY, ObjectSpawner::default()))
            .id();
        let root = world
            .spawn((
                Transform::from_xyz(10.0, 1.0, -4.0)
                    .with_scale(Vec3::new(2.0, 1.0, 2.0)),
                ChildOf(scene_root),
            ))
            .id();

        world.spawn((
            Transform::from_xyz(1.0, 0.0, 2.0),
            ObjectSpawner {
                capacity: ObjectSizeRank::Small,
                spawn_chance: 0.5,
            },
            ChildOf(root),
        ));
        world.spawn((
            Transform::from_xyz(-2.0, 0.0, 0.0),
            ObjectSpawner {
                capacity: ObjectSizeRank::Large,
                spawn_chance: 1.0,
            },
            ChildOf(root),
        ));

        let points = collect_object_spawn_points(&world, root);

        assert_eq!(points.len(), 2);
        assert!(points.iter().any(|point| {
            point
                .position
                .abs_diff_eq(Vec3::new(12.0, 1.0, 0.0), 1e-5)
        }));
        assert!(points.iter().any(|point| {
            point
                .position
                .abs_diff_eq(Vec3::new(6.0, 1.0, -4.0), 1e-5)
        }));
    }

    #[test]
    fn marker_only_empty_nodes_are_removed_but_mesh_hierarchy_is_kept() {
        let mut world = World::new();
        let root = world
            .spawn((
                Name::new("Tile"),
                Transform::IDENTITY,
                Mesh3d(Handle::<Mesh>::default()),
                ObjectSpawner::default(),
            ))
            .id();
        let marker = world
            .spawn((
                Name::new("Spawn marker"),
                Transform::from_xyz(2.0, 0.0, 1.0),
                ObjectSpawner::default(),
                ChildOf(root),
            ))
            .id();
        let group = world
            .spawn((Name::new("Mesh group"), Transform::IDENTITY, ChildOf(root)))
            .id();
        let mesh_child = world
            .spawn((
                Transform::IDENTITY,
                Mesh3d(Handle::<Mesh>::default()),
                ChildOf(group),
            ))
            .id();
        let empty_branch = world
            .spawn((
                Name::new("Unsupported empty"),
                Transform::IDENTITY,
                ChildOf(root),
            ))
            .id();
        let blocker_parent = world
            .spawn((
                Name::new("Authored blocker parent"),
                Transform::from_xyz(0.0, 0.0, 2.0),
                ChildOf(root),
            ))
            .id();
        let blocker = world
            .spawn((
                Name::new("Authored movement blocker"),
                Transform::from_xyz(1.0, 0.0, 0.0),
                MovementBlocker,
                ChildOf(blocker_parent),
            ))
            .id();

        let removed = strip_prototype_markers(&mut world, root);

        assert_eq!(removed, 2);
        assert!(world.get::<ObjectSpawner>(root).is_none());
        assert!(world.get::<ObjectSpawner>(marker).is_none());
        assert!(world.get::<Transform>(marker).is_none());
        assert!(world.get::<Transform>(empty_branch).is_none());
        assert!(world.get::<Transform>(blocker_parent).is_some());
        assert!(world.get::<Transform>(blocker).is_some());
        assert!(world.get::<MovementBlocker>(blocker).is_some());
        assert!(world.get::<Transform>(group).is_some());
        assert!(world.get::<Mesh3d>(mesh_child).is_some());
    }

    #[test]
    fn prototype_mesh_bottom_includes_mesh_and_ancestor_transforms() {
        let mut meshes = Assets::<Mesh>::default();
        let mesh_handle = meshes.add(Cuboid::new(2.0, 1.0, 4.0));

        let mut world = World::new();
        let root = world
            .spawn(
                Transform::from_xyz(10.0, 3.0, 4.0)
                    .with_scale(Vec3::new(2.0, 1.0, 1.0)),
            )
            .id();
        world.spawn((
            Mesh3d(mesh_handle),
            Transform::from_xyz(1.0, 0.0, 0.0),
            ChildOf(root),
        ));

        let mesh_bottom = prototype_mesh_bottom(&world, &meshes).unwrap();

        assert!((mesh_bottom - 2.5).abs() < 1e-5);
    }

    #[test]
    fn normalize_prototype_root_removes_collection_ancestors() {
        let mut world = World::new();
        let scene = world.spawn(Transform::IDENTITY).id();
        let collection = world
            .spawn((Transform::IDENTITY, ChildOf(scene)))
            .id();
        let root = world
            .spawn((
                Name::new("Prototype"),
                Transform::from_xyz(12.0, 3.0, -11.0),
                ChildOf(collection),
            ))
            .id();
        let mut asset = WorldAsset::new(world);

        assert_eq!(
            normalize_prototype_root(
                &mut asset,
                Some(&Name::new("Prototype")),
                "test.glb"
            ),
            Some(root)
        );

        assert!(asset.world.get::<ChildOf>(root).is_none());
        assert!(asset.world.get::<Transform>(scene).is_none());
        assert!(asset.world.get::<Transform>(collection).is_none());
        assert_eq!(
            asset.world.get::<Transform>(root).unwrap().translation,
            Vec3::ZERO
        );
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

        assert_eq!(
            normalize_prototype_root(
                &mut asset,
                Some(&Name::new("Prototype")),
                "test.glb"
            ),
            Some(root)
        );

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
