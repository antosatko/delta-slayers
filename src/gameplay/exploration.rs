use bevy::{
    gltf::GltfAssetLabel,
    prelude::*,
    world_serialization::{WorldAsset, WorldAssetRoot},
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    app::state::AppState,
    gameplay::{
        npc::Npc,
        player::{self, MovementBlocker, PLAYER_VISUAL_HEIGHT, Player},
    },
};

const MAP_WORLD_OFFSET: f32 = 512.0;
const HEX_CORNER_COUNT: usize = 6;
const TILE_MODEL_SOURCE_RADIUS: f32 = 5.0;
const TERRAIN_OCTAVES: usize = 4;
const MIN_MAP_RADIUS: u32 = 8;
const MAX_MAP_RADIUS: u32 = 32;

#[derive(Component, Reflect, Clone, Copy, Debug)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::exploration"]
pub struct ExplorationPortal {
    /// A fixed seed makes this portal generate the same map every time.
    /// Leave empty to generate a new seed on each visit.
    pub seed: Option<u64>,
    /// Radius measured in hex tiles.
    pub radius: u32,
    /// Rendered radius of one hex tile, in meters.
    pub tile_size: f32,
    /// Maximum vertical displacement of the terrain from its base level.
    pub height_multiplier: f32,
    /// Controls the scale of elevation features.
    pub terrain_frequency: f32,
    /// Controls the scale of biome regions.
    pub biome_scale: f32,
    /// Legacy Skein field retained so older town-map.glb exports still instantiate.
    /// No procedural vegetation is generated from this value.
    pub vegetation_density: f32,
    /// Maximum elevation difference the player can jump between adjacent tiles, in meters.
    pub max_jump_height: f32,
}

impl Default for ExplorationPortal {
    fn default() -> Self {
        Self {
            seed: None,
            radius: 12,
            tile_size: 6.0,
            height_multiplier: 2.2,
            terrain_frequency: 0.045,
            biome_scale: 0.028,
            vegetation_density: 0.28,
            max_jump_height: 1.5,
        }
    }
}

impl ExplorationPortal {
    pub(crate) fn sanitized(self) -> Self {
        Self {
            radius: self.radius.clamp(MIN_MAP_RADIUS, MAX_MAP_RADIUS),
            tile_size: finite_clamp(self.tile_size, 6.0, 4.0, 8.0),
            height_multiplier: finite_clamp(self.height_multiplier, 2.2, 0.0, 5.0),
            terrain_frequency: finite_clamp(self.terrain_frequency, 0.045, 0.01, 0.15),
            biome_scale: finite_clamp(self.biome_scale, 0.028, 0.006, 0.12),
            vegetation_density: finite_clamp(self.vegetation_density, 0.28, 0.0, 0.8),
            max_jump_height: finite_clamp(self.max_jump_height, 1.5, 0.25, 3.5),
            ..self
        }
    }
}

#[derive(Component)]
pub(super) struct ExplorationReturnPortal;

#[derive(Resource, Default)]
pub(crate) struct NearbyExplorationPortal(pub(crate) Option<PortalAction>);

#[derive(Resource, Default)]
pub(crate) struct ExplorationTravelRequest(pub(crate) Option<PortalAction>);

#[derive(Resource, Default)]
pub(crate) struct ActiveExplorationMap(pub(crate) Option<ActiveMapData>);

#[derive(Clone, Copy)]
pub(crate) struct ActiveMapData {
    root: Entity,
    pub(crate) origin: Vec3,
    pub(crate) config: ExplorationPortal,
    pub(crate) seed: u64,
}

#[derive(Resource, Default)]
pub(super) struct ExplorationSession(pub(super) Option<TravelAnchor>);

#[derive(Clone, Copy)]
pub(super) struct TravelAnchor {
    pub(super) transform: Transform,
    pub(super) yaw: f32,
    pub(super) pitch: f32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum PortalAction {
    Enter(ExplorationPortal),
    Return,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Biome {
    Grassland,
    Woodland,
    Dryland,
    Highlands,
}

impl Biome {
    fn color(self) -> [f32; 4] {
        match self {
            Self::Grassland => [0.28, 0.46, 0.22, 1.0],
            Self::Woodland => [0.16, 0.34, 0.17, 1.0],
            Self::Dryland => [0.55, 0.48, 0.28, 1.0],
            Self::Highlands => [0.39, 0.42, 0.40, 1.0],
        }
    }

    pub(crate) fn minimap_color(self) -> Color {
        let color = self.color();
        Color::srgb(color[0], color[1], color[2])
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct ExplorationTile {
    pub(crate) q: i32,
    pub(crate) r: i32,
    pub(crate) biome: Biome,
}

/// Added to generated tiles that cannot be reached from the map's spawn tile
/// without crossing a transition higher than the configured jump limit.
#[derive(Component, Default, Clone, Copy, Debug)]
pub(crate) struct UnreachableTile;

#[derive(Clone, Copy)]
struct GeneratedTile {
    center: Vec2,
    height: f32,
    biome: Biome,
}

const HEX_NEIGHBORS: [(i32, i32); 6] = [
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, 0),
    (-1, 1),
    (0, 1),
];

/// Updates the nearby portal target used by the existing E interaction prompt.
/// Authored portals use the same transformed Blender-cube bounds as TriggerBox.
pub(super) fn detect_portal(
    player: Query<&Transform, With<Player>>,
    portals: Query<(&GlobalTransform, &ExplorationPortal)>,
    return_portals: Query<&GlobalTransform, With<ExplorationReturnPortal>>,
    mut nearby: ResMut<NearbyExplorationPortal>,
) {
    let Ok(player) = player.single() else {
        nearby.0 = None;
        return;
    };

    let interaction_point = player.translation + Vec3::Y * player::INTERACTION_HEIGHT;
    nearby.0 = portals
        .iter()
        .find(|(transform, _)| contains_point(transform, interaction_point))
        .map(|(_, portal)| PortalAction::Enter(*portal))
        .or_else(|| {
            return_portals
                .iter()
                .any(|transform| contains_point(transform, interaction_point))
                .then_some(PortalAction::Return)
        });
}

fn contains_point(transform: &GlobalTransform, point: Vec3) -> bool {
    let local = transform.affine().inverse().transform_point3(point);
    local.is_finite() && local.abs().cmple(Vec3::ONE).all()
}

/// Consumes an interaction request and either creates an exploration instance
/// or restores the town-side transform captured when the player entered it.
pub(super) fn process_travel_request(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut request: ResMut<ExplorationTravelRequest>,
    mut active: ResMut<ActiveExplorationMap>,
    mut session: ResMut<ExplorationSession>,
    mut movement_target: ResMut<player::MovementTarget>,
    mut player: Query<(&mut Transform, &mut Player), Without<Npc>>,
    mut npcs: Query<&mut Transform, (With<Npc>, Without<Player>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(action) = request.0.take() else {
        return;
    };
    let Ok((mut player_transform, mut player_state)) = player.single_mut() else {
        return;
    };

    match action {
        PortalAction::Enter(authored_config) => {
            if active.0.is_some() {
                return;
            }

            let original_transform = *player_transform;
            let mut return_transform = original_transform;
            let mut exit_direction = original_transform.rotation * Vec3::NEG_Z;
            exit_direction.y = 0.0;
            return_transform.translation += exit_direction.normalize_or_zero() * 3.0;
            session.0 = Some(TravelAnchor {
                transform: return_transform,
                yaw: player_state.yaw,
                pitch: player_state.pitch,
            });

            let config = authored_config.sanitized();
            let seed = config.seed.unwrap_or_else(generate_seed);
            let tile_assets: [Handle<WorldAsset>; 2] = [
                asset_server.load(GltfAssetLabel::Scene(0).from_asset("tiles/1.glb")),
                asset_server.load(GltfAssetLabel::Scene(0).from_asset("tiles/2.glb")),
            ];
            let origin = Vec3::new(
                original_transform.translation.x + MAP_WORLD_OFFSET,
                0.0,
                original_transform.translation.z,
            );
            let root = commands
                .spawn((
                    Transform::from_translation(origin),
                    Visibility::Inherited,
                    ExplorationMapRoot,
                    DespawnOnExit::<AppState>(AppState::InGame),
                ))
                .id();

            spawn_map_geometry(
                &mut commands,
                root,
                config,
                seed,
                &tile_assets,
                &mut meshes,
                &mut materials,
            );

            let spawn_local = hex_center(0, 2, hex_layout_radius(config));
            let spawn_height = terrain_surface_height(config, seed, spawn_local);
            player_transform.translation =
                origin + Vec3::new(spawn_local.x, spawn_height, spawn_local.y);

            for mut npc in &mut npcs {
                npc.translation =
                    player_transform.translation + Vec3::new(3.0, PLAYER_VISUAL_HEIGHT, 3.0);
            }

            active.0 = Some(ActiveMapData {
                root,
                origin,
                config,
                seed,
            });
            movement_target.clear();
            info!(
                "Entered exploration map (seed {seed}, radius {}, tile radius {:.2} m)",
                config.radius, config.tile_size
            );
        }
        PortalAction::Return => {
            let Some(anchor) = session.0.take() else {
                return;
            };
            if let Some(map) = active.0.take() {
                commands.entity(map.root).despawn();
            }

            *player_transform = anchor.transform;
            player_state.yaw = anchor.yaw;
            player_state.pitch = anchor.pitch;
            for mut npc in &mut npcs {
                npc.translation =
                    player_transform.translation + Vec3::new(3.0, PLAYER_VISUAL_HEIGHT, 3.0);
            }
            movement_target.clear();
            info!("Returned to the town");
        }
    }
}

#[derive(Component)]
struct ExplorationMapRoot;

pub(crate) fn generate_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0xD37A_51A7_0B5E_ED01)
}

fn spawn_map_geometry(
    commands: &mut Commands,
    root: Entity,
    config: ExplorationPortal,
    seed: u64,
    tile_assets: &[Handle<WorldAsset>; 2],
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
        tile_assets,
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
fn reachable_tiles(
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
    let base_surface_offset = 0.016821861267089844 * model_scale;

    for (&(q, r), tile) in tiles {
        let center = tile.center;
        let variant_roll = cell_random(seed, q, r, 0xA24B_5F31_7C91_02D7);
        let protected_area = center.length() < config.tile_size * 2.5 || (q == 0 && r == 2);
        let feature_chance = if tile.biome == Biome::Highlands {
            0.55
        } else {
            0.08
        };
        let use_feature_tile = !protected_area && variant_roll < feature_chance;
        let tile_index = if use_feature_tile { 1 } else { 0 };

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
            let b = hex_corner(
                tile.center,
                config.tile_size,
                (side + 1) % HEX_CORNER_COUNT,
            );
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

/// Keeps the player and Godo on the smoothstep-interpolated terrain surface.
/// The player transform remains the ground-position authority.
pub(super) fn sync_ground_height(
    active: Res<ActiveExplorationMap>,
    mut player: Query<&mut Transform, With<Player>>,
    mut npcs: Query<&mut Transform, (With<Npc>, Without<Player>)>,
) {
    let Some(map) = active.0 else {
        return;
    };

    if let Ok(mut transform) = player.single_mut() {
        let local = transform.translation - map.origin;
        transform.translation.y = map.origin.y
            + terrain_surface_height(map.config, map.seed, Vec2::new(local.x, local.z));
    }

    for mut transform in &mut npcs {
        let local = transform.translation - map.origin;
        transform.translation.y = map.origin.y
            + terrain_surface_height(map.config, map.seed, Vec2::new(local.x, local.z))
            + PLAYER_VISUAL_HEIGHT;
    }
}

fn terrain_surface_height(config: ExplorationPortal, seed: u64, point: Vec2) -> f32 {
    let size = hex_layout_radius(config);
    let axial_q = (3.0_f32.sqrt() / 3.0 * point.x - point.y / 3.0) / size;
    let axial_r = (2.0 / 3.0 * point.y) / size;
    let q0 = axial_q.floor() as i32;
    let r0 = axial_r.floor() as i32;
    let tq = axial_q - q0 as f32;
    let tr = axial_r - r0 as f32;

    // Split each axial parallelogram along its short diagonal. Smoothstep
    // barycentric weights agree on shared edges, so the field stays continuous.
    let (coordinates, weights) = if tq + tr <= 1.0 {
        (
            [(q0, r0), (q0 + 1, r0), (q0, r0 + 1)],
            [1.0 - tq - tr, tq, tr],
        )
    } else {
        (
            [(q0 + 1, r0 + 1), (q0, r0 + 1), (q0 + 1, r0)],
            [tq + tr - 1.0, 1.0 - tq, 1.0 - tr],
        )
    };

    let smooth_weights = weights.map(|weight| smoothstep(weight.clamp(0.0, 1.0)));
    let weight_total = smooth_weights.iter().sum::<f32>();
    coordinates
        .into_iter()
        .zip(smooth_weights)
        .map(|((q, r), weight)| {
            let center = hex_center(q, r, size);
            terrain_height(config, seed, center) * weight
        })
        .sum::<f32>()
        / weight_total.max(f32::EPSILON)
}

fn terrain_height(config: ExplorationPortal, seed: u64, point: Vec2) -> f32 {
    fractal_noise(
        point.x * config.terrain_frequency,
        point.y * config.terrain_frequency,
        seed,
    ) * config.height_multiplier
}

fn biome_at(config: ExplorationPortal, seed: u64, point: Vec2) -> Biome {
    let elevation = fractal_noise(
        point.x * config.terrain_frequency,
        point.y * config.terrain_frequency,
        seed,
    );
    let moisture = fractal_noise(
        point.x * config.biome_scale,
        point.y * config.biome_scale,
        seed ^ 0x6A09_E667_F3BC_C909,
    );
    if elevation > 0.48 {
        Biome::Highlands
    } else if moisture > 0.28 {
        Biome::Woodland
    } else if moisture < -0.3 {
        Biome::Dryland
    } else {
        Biome::Grassland
    }
}

fn fractal_noise(x: f32, z: f32, seed: u64) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 0.55;
    let mut frequency = 1.0;
    let mut normalization = 0.0;

    for octave in 0..TERRAIN_OCTAVES {
        total += value_noise(
            x * frequency,
            z * frequency,
            seed.wrapping_add((octave as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)),
        ) * amplitude;
        normalization += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    (total / normalization).clamp(-1.0, 1.0)
}

fn value_noise(x: f32, z: f32, seed: u64) -> f32 {
    let x0 = x.floor() as i64;
    let z0 = z.floor() as i64;
    let tx = smoothstep(x - x.floor());
    let tz = smoothstep(z - z.floor());

    let a = lattice_noise(seed, x0, z0);
    let b = lattice_noise(seed, x0 + 1, z0);
    let c = lattice_noise(seed, x0, z0 + 1);
    let d = lattice_noise(seed, x0 + 1, z0 + 1);

    lerp(lerp(a, b, tx), lerp(c, d, tx), tz)
}

fn lattice_noise(seed: u64, x: i64, z: i64) -> f32 {
    let mut value = seed
        ^ (x as u64).wrapping_mul(0x9E37_79B1_85EB_CA87)
        ^ (z as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;
    let unit = (value >> 40) as f32 / ((1_u32 << 24) as f32);
    unit * 2.0 - 1.0
}

fn cell_random(seed: u64, q: i32, r: i32, salt: u64) -> f32 {
    (lattice_noise(seed ^ salt, q as i64, r as i64) + 1.0) * 0.5
}

fn smoothstep(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn hex_layout_radius(config: ExplorationPortal) -> f32 {
    config.tile_size
}

pub(crate) fn hex_center(q: i32, r: i32, size: f32) -> Vec2 {
    Vec2::new(
        3.0_f32.sqrt() * size * (q as f32 + r as f32 * 0.5),
        1.5 * size * r as f32,
    )
}

fn hex_corner(center: Vec2, size: f32, index: usize) -> Vec2 {
    let angle = std::f32::consts::FRAC_PI_6 + index as f32 * std::f32::consts::PI / 3.0;
    center + Vec2::new(angle.cos(), angle.sin()) * size
}

pub(crate) fn world_to_axial(point: Vec2, size: f32) -> (i32, i32) {
    let q = (3.0_f32.sqrt() / 3.0 * point.x - point.y / 3.0) / size;
    let r = (2.0 / 3.0 * point.y) / size;
    let x = q;
    let z = r;
    let y = -x - z;

    let mut rx = x.round();
    let ry = y.round();
    let mut rz = z.round();
    let dx = (rx - x).abs();
    let dy = (ry - y).abs();
    let dz = (rz - z).abs();
    if dx > dy && dx > dz {
        rx = -ry - rz;
    } else if dy > dz {
        rz = -rx - ry;
    }
    (rx as i32, rz as i32)
}

fn hex_distance(q: i32, r: i32) -> i32 {
    q.abs().max(r.abs()).max((-q - r).abs())
}

fn finite_clamp(value: f32, fallback: f32, min: f32, max: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_map_tile_count_matches_radius() {
        for radius in 1..=16 {
            let count = (-(radius as i32)..=radius as i32)
                .flat_map(|q| {
                    let min_r = (-(radius as i32)).max(-q - radius as i32);
                    let max_r = (radius as i32).min(-q + radius as i32);
                    min_r..=max_r
                })
                .count();
            assert_eq!(count, 1 + 3 * radius as usize * (radius as usize + 1));
        }
    }

    #[test]
    fn seeded_height_generation_is_deterministic_and_seed_sensitive() {
        let config = ExplorationPortal::default();
        let point = Vec2::new(17.5, -9.25);
        let first = terrain_height(config, 1234, point);
        let repeated = terrain_height(config, 1234, point);
        let different = terrain_height(config, 9876, point);

        assert_eq!(first, repeated);
        assert_ne!(first, different);
    }

    #[test]
    fn octave_seed_offsets_use_wrapping_arithmetic() {
        let sample = fractal_noise(0.2, -0.4, u64::MAX);

        assert!(sample.is_finite());
        assert!((-1.0..=1.0).contains(&sample));
    }

    #[test]
    fn portal_parameters_are_clamped_to_safe_ranges() {
        let config = ExplorationPortal {
            seed: Some(42),
            radius: 1000,
            tile_size: f32::NAN,
            height_multiplier: -4.0,
            terrain_frequency: f32::INFINITY,
            biome_scale: 0.0,
            vegetation_density: 3.0,
            max_jump_height: 9.0,
        }
        .sanitized();

        assert_eq!(config.radius, MAX_MAP_RADIUS);
        assert_eq!(config.tile_size, 6.0);
        assert_eq!(config.height_multiplier, 0.0);
        assert_eq!(config.terrain_frequency, 0.045);
        assert_eq!(config.biome_scale, 0.006);
        assert_eq!(config.vegetation_density, 0.8);
        assert_eq!(config.max_jump_height, 3.5);
        assert_eq!(config.seed, Some(42));
    }

    #[test]
    fn tile_layout_uses_the_full_imported_model_radius() {
        let config = ExplorationPortal::default();

        assert_eq!(config.tile_size, 6.0);
        assert_eq!(hex_layout_radius(config), 6.0);

        let too_small = ExplorationPortal {
            tile_size: 2.0,
            ..config
        }
        .sanitized();
        let too_large = ExplorationPortal {
            tile_size: 9.0,
            ..config
        }
        .sanitized();

        assert_eq!(too_small.tile_size, 4.0);
        assert_eq!(too_large.tile_size, 8.0);
    }

    #[test]
    fn terrain_height_uses_smoothstep_between_adjacent_tiles() {
        let config = ExplorationPortal::default();
        let first = hex_center(0, 0, hex_layout_radius(config));
        let second = hex_center(1, 0, hex_layout_radius(config));
        let first_height = terrain_height(config, 1234, first);
        let second_height = terrain_height(config, 1234, second);
        let midpoint = (first + second) * 0.5;

        assert_ne!(first_height, second_height);
        assert!((terrain_surface_height(config, 1234, first) - first_height).abs() < 1e-5);
        assert!(
            (terrain_surface_height(config, 1234, midpoint)
                - (first_height + second_height) * 0.5)
                .abs()
                < 1e-4
        );
    }

    #[test]
    fn reachability_stops_at_transitions_above_the_jump_limit() {
        let tiles = HashMap::from([
            ((0, 0), GeneratedTile {
                center: Vec2::ZERO,
                height: 0.0,
                biome: Biome::Grassland,
            }),
            ((1, 0), GeneratedTile {
                center: Vec2::X,
                height: 0.75,
                biome: Biome::Grassland,
            }),
            ((2, 0), GeneratedTile {
                center: Vec2::X * 2.0,
                height: 2.0,
                biome: Biome::Highlands,
            }),
        ]);

        let reachable = reachable_tiles(&tiles, (0, 0), 1.0);

        assert!(reachable.contains(&(0, 0)));
        assert!(reachable.contains(&(1, 0)));
        assert!(!reachable.contains(&(2, 0)));
    }

    #[test]
    fn axial_conversion_round_trips_hex_centers() {
        for q in -5..=5 {
            for r in -5..=5 {
                let center = hex_center(q, r, 1.5);
                assert_eq!(world_to_axial(center, 1.5), (q, r));
            }
        }
    }

    #[test]
    fn perimeter_wall_edges_face_outside_their_hex_neighbors() {
        let boundary_edges = [
            ((1, 0), 5),
            ((1, -1), 4),
            ((0, -1), 3),
            ((-1, 0), 2),
            ((-1, 1), 1),
            ((0, 1), 0),
        ];
        let center = hex_center(0, 0, 1.0);

        for ((dq, dr), side) in boundary_edges {
            let neighbor_direction = hex_center(dq, dr, 1.0) - center;
            let a = hex_corner(center, 1.0, side);
            let b = hex_corner(center, 1.0, (side + 1) % HEX_CORNER_COUNT);
            let edge_midpoint = (a + b) * 0.5;
            assert!(edge_midpoint.dot(neighbor_direction) > 0.0);
        }
    }
}
