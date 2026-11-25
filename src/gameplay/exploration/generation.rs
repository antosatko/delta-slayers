use bevy::{
    prelude::*,
    world_serialization::{WorldAsset, WorldAssetRoot},
};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::gameplay::player::MovementBlocker;

use super::hex::{HEX_CORNER_COUNT, HEX_NEIGHBORS, hex_center, hex_corner, hex_layout_radius};
use super::terrain::{biome_at, cell_random, terrain_height, terrain_surface_height};
use super::{Biome, ExplorationPortal, ExplorationReturnPortal, ExplorationTile, UnreachableTile};

const TILE_MODEL_SOURCE_RADIUS: f32 = 5.0;

#[derive(Clone, Copy)]
pub(super) struct GeneratedTile {
    pub(super) center: Vec2,
    pub(super) height: f32,
    pub(super) biome: Biome,
    pub(super) has_feature: bool,
}

#[derive(Component)]
pub(super) struct ExplorationMapRoot;

pub(super) fn spawn_map_geometry(
    commands: &mut Commands,
    root: Entity,
    config: ExplorationPortal,
    seed: u64,
    tile_assets: &[Handle<WorldAsset>; 2],
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    tuning: &super::VegetationTuning,
) {
    let tiles = generate_tiles(config, seed);
    let reachable = reachable_tiles(&tiles, (0, 2), config.max_jump_height);
    spawn_terrain_tiles(
        commands,
        root,
        config,
        seed,
        tile_assets,
        materials,
        &tiles,
        &reachable,
    );
    spawn_boundary_walls(commands, root, config, &tiles);
    super::vegetation::spawn_vegetation(
        commands,
        root,
        config,
        seed,
        &tiles,
        asset_server,
        images,
        tuning,
    );

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
            let biome = biome_at(config, seed, center);
            let variant_roll = cell_random(seed, q, r, 0xA24B_5F31_7C91_02D7);
            let protected_area = center.length() < config.tile_size * 2.5 || (q == 0 && r == 2);
            let feature_chance = if biome == Biome::Highlands { 0.55 } else { 0.08 };
            let has_feature = !protected_area && variant_roll < feature_chance;

            tiles.insert(
                (q, r),
                GeneratedTile {
                    center,
                    height: terrain_height(config, seed, center),
                    biome,
                    has_feature,
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
    tile_assets: &[Handle<WorldAsset>; 2],
    materials: &mut Assets<StandardMaterial>,
    tiles: &HashMap<(i32, i32), GeneratedTile>,
    reachable: &HashSet<(i32, i32)>,
) {
    let model_scale = config.tile_size / TILE_MODEL_SOURCE_RADIUS;
    let base_surface_offset = 0.016_821_861 * model_scale;

    for (&(q, r), tile) in tiles {
        let center = tile.center;
        let tile_index = usize::from(tile.has_feature);

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
            WorldAssetRoot(tile_assets[tile_index].clone()),
            Transform::from_xyz(center.x, tile.height + base_surface_offset, center.y)
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
    }
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
