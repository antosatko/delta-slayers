use bevy::{prelude::*, world_serialization::WorldAssetRoot};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::gameplay::{
    environment::{
        EnvironmentAssets, ObjectSizeRank, ObjectSpawner, RemainingObjectCapacity,
    },
    player::MovementBlocker,
};

use super::hex::{HEX_CORNER_COUNT, HEX_NEIGHBORS, hex_center, hex_corner, hex_layout_radius};
use super::terrain::{biome_at, cell_random, terrain_height, terrain_surface_height};
use super::{Biome, ExplorationPortal, ExplorationReturnPortal, ExplorationTile, UnreachableTile};

const TILE_MODEL_SOURCE_RADIUS: f32 = 5.0;
const TILE_VARIANT_SALT: u64 = 0xA24B_5F31_7C91_02D7;
const OBJECT_SPAWN_SALT: u64 = 0x83D2_EA19_6C45_B70F;

/// Approximate half-extents matched to the current exported object meshes:
/// the small cone is about 0.86 m wide; the medium Suzanne mesh is 2.74 m wide.
fn object_collision_dimensions(size: ObjectSizeRank) -> (f32, f32) {
    match size {
        ObjectSizeRank::Small => (0.5, 0.5),
        ObjectSizeRank::Medium => (1.4, 0.85),
        ObjectSizeRank::Large => (2.0, 1.2),
    }
}

#[derive(Clone, Copy)]
pub(super) struct GeneratedTile {
    pub(super) center: Vec2,
    pub(super) height: f32,
    pub(super) biome: Biome,
}

#[derive(Component)]
pub(super) struct ExplorationMapRoot;

pub(super) fn spawn_map_geometry(
    commands: &mut Commands,
    root: Entity,
    config: ExplorationPortal,
    seed: u64,
    environment: &EnvironmentAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let tiles = generate_tiles(config, seed);
    let reachable = reachable_tiles(&tiles, (0, 2), config.max_jump_height);
    spawn_terrain_tiles(
        commands,
        root,
        config,
        seed,
        environment,
        materials,
        &tiles,
        &reachable,
    );
    spawn_boundary_walls(commands, root, config, &tiles);

    let return_height = terrain_surface_height(config, seed, Vec2::ZERO);
    // Keep the interaction volume independent from the beacon's visual scale.
    // Its vertical bounds cover the entire configured terrain height range.
    commands.spawn((
        ExplorationReturnPortal,
        Transform::from_scale(Vec3::new(1.6, config.height_multiplier + 2.0, 1.6)),
        ChildOf(root),
    ));

    let portal_base = meshes.add(Cylinder::new(0.9, 0.1));
    let portal_beam = meshes.add(Cylinder::new(0.075, 1.8));
    let portal_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.72, 0.96),
        emissive: LinearRgba::new(0.08, 0.3, 0.5, 1.0),
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(portal_base),
        MeshMaterial3d(portal_material.clone()),
        Transform::from_xyz(0.0, return_height + 0.08, 0.0),
        ChildOf(root),
    ));
    commands.spawn((
        Mesh3d(portal_beam),
        MeshMaterial3d(portal_material),
        Transform::from_xyz(0.0, return_height + 0.95, 0.0),
        ChildOf(root),
    ));
}

#[derive(Component)]
pub struct ExplorationTileTint(Handle<StandardMaterial>);

#[derive(Component)]
pub struct ExplorationTileTintApplied;

/// Applies biome materials to imported meshes, including nested GLB children.
/// Instancing the complete world preserves Skein-authored components on tile 2.
pub fn apply_tile_materials(
    mut commands: Commands,
    tiles: Query<(Entity, &ExplorationTileTint, &Children), Without<ExplorationTileTintApplied>>,
    renderables: Query<(Option<&Children>, Has<Mesh3d>)>,
) {
    for (tile, tint, children) in &tiles {
        let mut pending = children.iter().collect::<Vec<_>>();
        let mut applied = false;

        while let Some(entity) = pending.pop() {
            let Ok((nested_children, has_mesh)) = renderables.get(entity) else {
                continue;
            };

            if has_mesh {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(tint.0.clone()));
                applied = true;
            }

            if let Some(nested_children) = nested_children {
                pending.extend(nested_children.iter());
            }
        }

        if applied {
            commands.entity(tile).insert(ExplorationTileTintApplied);
        }
    }
}

fn generate_tiles(config: ExplorationPortal, seed: u64) -> HashMap<(i32, i32), GeneratedTile> {
    let mut tiles = HashMap::new();
    let layout_radius = hex_layout_radius(config);

    for q in -(config.radius as i32)..=config.radius as i32 {
        let min_r = (-(config.radius as i32)).max(-q - config.radius as i32);
        let max_r = (config.radius as i32).min(-q + config.radius as i32);
        for r in min_r..=max_r {
            let center = hex_center(q, r, layout_radius);
            tiles.insert(
                (q, r),
                GeneratedTile {
                    center,
                    height: terrain_height(config, seed, center),
                    biome: biome_at(config, seed, center),
                },
            );
        }
    }

    tiles
}

/// Finds all tiles connected to the spawn tile through jumps within the
/// configured height limit. The same height limit is used for cliff blockers.
pub(super) fn reachable_tiles(
    tiles: &HashMap<(i32, i32), GeneratedTile>,
    start: (i32, i32),
    max_jump_height: f32,
) -> HashSet<(i32, i32)> {
    let mut reachable = HashSet::new();
    let mut pending = VecDeque::new();

    if tiles.contains_key(&start) {
        reachable.insert(start);
        pending.push_back(start);
    }

    while let Some(current) = pending.pop_front() {
        let Some(current_tile) = tiles.get(&current) else {
            continue;
        };

        for (dq, dr) in HEX_NEIGHBORS {
            let neighbor = (current.0 + dq, current.1 + dr);
            if reachable.contains(&neighbor) {
                continue;
            }

            let Some(neighbor_tile) = tiles.get(&neighbor) else {
                continue;
            };

            if (neighbor_tile.height - current_tile.height).abs() <= max_jump_height {
                reachable.insert(neighbor);
                pending.push_back(neighbor);
            }
        }
    }

    reachable
}

fn spawn_terrain_tiles(
    commands: &mut Commands,
    root: Entity,
    config: ExplorationPortal,
    seed: u64,
    environment: &EnvironmentAssets,
    materials: &mut Assets<StandardMaterial>,
    tiles: &HashMap<(i32, i32), GeneratedTile>,
    reachable: &HashSet<(i32, i32)>,
) {
    let model_scale = config.tile_size / TILE_MODEL_SOURCE_RADIUS;
    let base_surface_offset = 0.016_821_861 * model_scale;
    let mut tiles_with_spawners = 0;
    let mut spawned_objects = 0;

    for (&(q, r), tile) in tiles {
        let variants = environment.tiles_for(tile.biome);
        if variants.is_empty() {
            continue;
        }

        let variant_roll = cell_random(seed, q, r, TILE_VARIANT_SALT);
        let variant_index = ((variant_roll * variants.len() as f32) as usize)
            .min(variants.len() - 1);
        let prototype = &variants[variant_index];

        let base_color = tile.biome.color();
        let shade = 0.92 + cell_random(seed, q, r, 0x51ED_270B_91A4_6C3D) * 0.14;
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(
                base_color[0] * shade,
                base_color[1] * shade,
                base_color[2] * shade,
                base_color[3],
            ),
            perceptual_roughness: 1.0,
            ..default()
        });

        let mut tile_entity = commands.spawn((
            WorldAssetRoot(prototype.handle.clone()),
            Transform::from_xyz(tile.center.x, tile.height + base_surface_offset, tile.center.y)
                .with_scale(Vec3::splat(model_scale)),
            ExplorationTile {
                q,
                r,
                biome: tile.biome,
            },
            ExplorationTileTint(material),
            ChildOf(root),
        ));

        if !reachable.contains(&(q, r)) {
            tile_entity.insert(UnreachableTile);
        }

        if let Some(spawner) = prototype.spawner {
            let tile_entity = tile_entity.id();
            commands.entity(tile_entity).insert((
                spawner,
                RemainingObjectCapacity(Some(spawner.capacity)),
            ));
            tiles_with_spawners += 1;
            spawned_objects += spawn_objects_on_tile(
                commands,
                root,
                tile_entity,
                config,
                seed,
                (q, r),
                tile,
                spawner,
                environment,
            );
        }
    }

    info!(
        "Generated {spawned_objects} objects across {tiles_with_spawners} object-spawning tiles"
    );
}

/// Performs deterministic, size-budgeted object placement for a tile prototype
/// that has an ObjectSpawner component.
fn spawn_objects_on_tile(
    commands: &mut Commands,
    map_root: Entity,
    tile_entity: Entity,
    config: ExplorationPortal,
    seed: u64,
    coordinates: (i32, i32),
    tile: &GeneratedTile,
    spawner: ObjectSpawner,
    environment: &EnvironmentAssets,
) -> usize {
    let mut remaining = spawner.capacity.units();
    let mut spawned_count = 0;
    let spawn_chance = spawner.effective_spawn_chance();

    for attempt in 0..spawner.capacity.units() {
        if spawn_chance <= 0.0 {
            break;
        }

        let attempt_salt = OBJECT_SPAWN_SALT
            .wrapping_add(u64::from(attempt).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let attempt_roll = cell_random(seed, coordinates.0, coordinates.1, attempt_salt);
        if attempt_roll > spawn_chance {
            break;
        }

        let selection_roll = cell_random(
            seed,
            coordinates.0,
            coordinates.1,
            attempt_salt ^ 0xD1B5_4A32_D192_ED03,
        );
        let Some(object) = environment.choose_spawnable(tile.biome, remaining, selection_roll) else {
            break;
        };

        let placement_roll = cell_random(
            seed,
            coordinates.0,
            coordinates.1,
            attempt_salt ^ 0x94D0_49BB_1331_11EB,
        );
        let angle = placement_roll * std::f32::consts::TAU;
        let radius_roll = cell_random(
            seed,
            coordinates.0,
            coordinates.1,
            attempt_salt ^ 0xBF58_476D_1CE4_E5B9,
        );
        let (collision_radius, collision_half_height) =
            object_collision_dimensions(object.definition.size);
        let minimum_offset = collision_radius + 0.32 + 0.1;
        let offset_radius =
            (config.tile_size * (0.12 + radius_roll * 0.32)).max(minimum_offset);
        let local_position = tile.center
            + Vec2::new(angle.cos(), angle.sin()) * offset_radius;
        let height = super::terrain::terrain_surface_height(config, seed, local_position);

        commands.spawn((
            WorldAssetRoot(object.handle.clone()),
            Transform::from_xyz(local_position.x, height, local_position.y)
                .with_rotation(Quat::from_rotation_y(angle)),
            ChildOf(map_root),
        ));

        // Spawn a separate invisible collider so the prototype's authored
        // render transform and hierarchy remain untouched.
        commands.spawn((
            MovementBlocker,
            Transform::from_xyz(
                local_position.x,
                height + collision_half_height,
                local_position.y,
            )
            .with_scale(Vec3::new(
                collision_radius,
                collision_half_height,
                collision_radius,
            )),
            ChildOf(map_root),
        ));
        spawned_count += 1;

        remaining = remaining.saturating_sub(object.definition.size.units());
        commands.entity(tile_entity).insert(RemainingObjectCapacity(
            ObjectSizeRank::from_units(remaining),
        ));
        if remaining == 0 {
            break;
        }
    }

    spawned_count
}

fn spawn_boundary_walls(
    commands: &mut Commands,
    root: Entity,
    config: ExplorationPortal,
    tiles: &HashMap<(i32, i32), GeneratedTile>,
) {
    const BOUNDARY_EDGES: [((i32, i32), usize); 6] = [
        ((1, 0), 5),
        ((1, -1), 4),
        ((0, -1), 3),
        ((-1, 0), 2),
        ((-1, 1), 1),
        ((0, 1), 0),
    ];

    for (&(q, r), tile) in tiles {
        for ((dq, dr), side) in BOUNDARY_EDGES {
            let neighbor_coordinates = (q + dq, r + dr);
            let neighbor = tiles.get(&neighbor_coordinates);

            if neighbor.is_some() && (q, r) > neighbor_coordinates {
                continue;
            }

            let blocks_jump = neighbor.is_none_or(|neighbor| {
                (neighbor.height - tile.height).abs() > config.max_jump_height
            });
            if !blocks_jump {
                continue;
            }

            let a = hex_corner(tile.center, config.tile_size, side);
            let b = hex_corner(tile.center, config.tile_size, (side + 1) % HEX_CORNER_COUNT);
            let midpoint = (a + b) * 0.5;
            let edge = b - a;
            let yaw = edge.x.atan2(edge.y);

            commands.spawn((
                MovementBlocker,
                Transform::from_xyz(midpoint.x, 0.0, midpoint.y)
                    .with_rotation(Quat::from_rotation_y(yaw))
                    .with_scale(Vec3::new(
                        0.18,
                        config.height_multiplier + 2.0,
                        config.tile_size * 0.62,
                    )),
                ChildOf(root),
            ));
        }
    }
}
