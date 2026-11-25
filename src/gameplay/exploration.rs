use bevy::{
    asset::RenderAssetUsages,
    mesh::Indices,
    prelude::*,
    render::render_resource::PrimitiveTopology,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    app::state::AppState,
    gameplay::{
        npc::Npc,
        player::{self, MovementBlocker, Player, PLAYER_VISUAL_HEIGHT},
    },
};

const MAP_WORLD_OFFSET: f32 = 512.0;
const HEX_CORNER_COUNT: usize = 6;
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
    /// Side length of one hex tile, in world units.
    pub tile_size: f32,
    /// Maximum vertical displacement of the terrain from its base level.
    pub height_multiplier: f32,
    /// Controls the scale of elevation features.
    pub terrain_frequency: f32,
    /// Controls the scale of biome regions.
    pub biome_scale: f32,
    /// Fraction of forest tiles populated with trees.
    pub vegetation_density: f32,
}

impl Default for ExplorationPortal {
    fn default() -> Self {
        Self {
            seed: None,
            radius: 12,
            tile_size: 1.5,
            height_multiplier: 2.2,
            terrain_frequency: 0.045,
            biome_scale: 0.028,
            vegetation_density: 0.28,
        }
    }
}

impl ExplorationPortal {
    pub(crate) fn sanitized(self) -> Self {
        Self {
            radius: self.radius.clamp(MIN_MAP_RADIUS, MAX_MAP_RADIUS),
            tile_size: finite_clamp(self.tile_size, 1.5, 1.0, 3.0),
            height_multiplier: finite_clamp(self.height_multiplier, 2.2, 0.0, 5.0),
            terrain_frequency: finite_clamp(self.terrain_frequency, 0.045, 0.01, 0.15),
            biome_scale: finite_clamp(self.biome_scale, 0.028, 0.006, 0.12),
            vegetation_density: finite_clamp(self.vegetation_density, 0.28, 0.0, 0.8),
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
    origin: Vec3,
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
enum Biome {
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
}

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

            spawn_map_geometry(&mut commands, root, config, seed, &mut meshes, &mut materials);

            let spawn_local = Vec2::new(0.0, config.tile_size * 3.0);
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
                "Entered exploration map (seed {seed}, radius {}, tile size {:.2})",
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

fn generate_seed() -> u64 {
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
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let terrain = meshes.add(build_terrain_mesh(config, seed));
    let terrain_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 1.0,
        ..default()
    });
    commands.spawn((
        Mesh3d(terrain),
        MeshMaterial3d(terrain_material),
        Transform::IDENTITY,
        ChildOf(root),
    ));

    spawn_vegetation(commands, root, config, seed, meshes, materials);
    spawn_boundary_walls(commands, root, config, seed);

    let return_height = terrain_surface_height(config, seed, Vec2::ZERO);
    // Keep the interaction volume independent from the beacon's visual scale.
    // Its vertical bounds cover the entire configured terrain height range.
    commands.spawn((
        ExplorationReturnPortal,
        Transform::from_scale(Vec3::new(
            1.6,
            config.height_multiplier + 2.0,
            1.6,
        )),
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

fn build_terrain_mesh(config: ExplorationPortal, seed: u64) -> Mesh {
    let mut positions = Vec::<[f32; 3]>::new();
    let mut normals = Vec::<[f32; 3]>::new();
    let mut colors = Vec::<[f32; 4]>::new();
    let mut indices = Vec::<u32>::new();

    for q in -(config.radius as i32)..=config.radius as i32 {
        let min_r = (-(config.radius as i32)).max(-q - config.radius as i32);
        let max_r = (config.radius as i32).min(-q + config.radius as i32);
        for r in min_r..=max_r {
            let center = hex_center(q, r, config.tile_size);
            let center_height = terrain_height(config, seed, center);
            let base_color = biome_at(config, seed, center).color();
            // Slight deterministic per-tile variation keeps the hex layout
            // readable without adding gaps or breaking the shared height field.
            let shade = 0.94
                + cell_random(seed, q, r, 0x51ED_270B_91A4_6C3D) * 0.12;
            let color = [
                base_color[0] * shade,
                base_color[1] * shade,
                base_color[2] * shade,
                base_color[3],
            ];

            for side in 0..HEX_CORNER_COUNT {
                let current = hex_corner(center, config.tile_size, side);
                let next = hex_corner(center, config.tile_size, (side + 1) % HEX_CORNER_COUNT);
                let current_height = terrain_height(config, seed, current);
                let next_height = terrain_height(config, seed, next);

                let first_index = positions.len() as u32;
                // Reverse the perimeter order so the top faces point upward.
                // Derive a flat normal per triangle so height variation affects
                // both the silhouette and the lighting.
                let center_vertex = Vec3::new(center.x, center_height, center.y);
                let next_vertex = Vec3::new(next.x, next_height, next.y);
                let current_vertex = Vec3::new(current.x, current_height, current.y);
                let normal = (next_vertex - center_vertex)
                    .cross(current_vertex - center_vertex)
                    .normalize_or_zero();
                positions.extend([
                    center_vertex.to_array(),
                    next_vertex.to_array(),
                    current_vertex.to_array(),
                ]);
                normals.extend([normal.to_array(); 3]);
                colors.extend([color; 3]);
                indices.extend([first_index, first_index + 1, first_index + 2]);
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn spawn_vegetation(
    commands: &mut Commands,
    root: Entity,
    config: ExplorationPortal,
    seed: u64,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let trunk = meshes.add(Cylinder::new(0.12, 0.7));
    let canopy = meshes.add(Cone::new(0.62, 1.35));
    let rock = meshes.add(Sphere::new(0.42));
    let wood_material = materials.add(Color::srgb(0.31, 0.19, 0.11));
    let leaf_material = materials.add(Color::srgb(0.17, 0.38, 0.18));
    let rock_material = materials.add(Color::srgb(0.43, 0.45, 0.43));

    for q in -(config.radius as i32)..=config.radius as i32 {
        let min_r = (-(config.radius as i32)).max(-q - config.radius as i32);
        let max_r = (config.radius as i32).min(-q + config.radius as i32);
        for r in min_r..=max_r {
            let center = hex_center(q, r, config.tile_size);
            let distance_from_return = center.length();
            if distance_from_return < config.tile_size * 2.5 {
                continue;
            }

            let biome = biome_at(config, seed, center);
            let chance = cell_random(seed, q, r, 0xA3C5_9AC3_7D21_4F01);
            let height = terrain_surface_height(config, seed, center);
            match biome {
                Biome::Woodland if chance < config.vegetation_density => {
                    commands.spawn((
                        Mesh3d(trunk.clone()),
                        MeshMaterial3d(wood_material.clone()),
                        Transform::from_xyz(center.x, height + 0.35, center.y),
                        ChildOf(root),
                    ));
                    commands.spawn((
                        Mesh3d(canopy.clone()),
                        MeshMaterial3d(leaf_material.clone()),
                        Transform::from_xyz(center.x, height + 1.05, center.y),
                        ChildOf(root),
                    ));
                }
                Biome::Highlands if chance < 0.32 => {
                    commands.spawn((
                        Mesh3d(rock.clone()),
                        MeshMaterial3d(rock_material.clone()),
                        Transform::from_xyz(center.x, height + 0.24, center.y)
                            .with_scale(Vec3::new(1.3, 0.75, 1.0)),
                        ChildOf(root),
                    ));
                }
                _ => {}
            }
        }
    }
}

fn spawn_boundary_walls(commands: &mut Commands, root: Entity, config: ExplorationPortal, seed: u64) {
    // Pointy-top axial directions mapped to the matching perimeter edge.
    const BOUNDARY_EDGES: [((i32, i32), usize); 6] = [
        ((1, 0), 5),
        ((1, -1), 4),
        ((0, -1), 3),
        ((-1, 0), 2),
        ((-1, 1), 1),
        ((0, 1), 0),
    ];

    for q in -(config.radius as i32)..=config.radius as i32 {
        let min_r = (-(config.radius as i32)).max(-q - config.radius as i32);
        let max_r = (config.radius as i32).min(-q + config.radius as i32);
        for r in min_r..=max_r {
            if hex_distance(q, r) != config.radius as i32 {
                continue;
            }

            let center = hex_center(q, r, config.tile_size);
            for ((dq, dr), side) in BOUNDARY_EDGES {
                if hex_distance(q + dq, r + dr) <= config.radius as i32 {
                    continue;
                }

                let a = hex_corner(center, config.tile_size, side);
                let b = hex_corner(center, config.tile_size, (side + 1) % HEX_CORNER_COUNT);
                let midpoint = (a + b) * 0.5;
                let edge = b - a;
                let yaw = edge.x.atan2(edge.y);
                // Cover the entire possible terrain height range, even when
                // the portal uses a high elevation multiplier.
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
}

/// Keeps the player and Godo on the same triangulated height field used by the
/// generated mesh. The player transform remains the ground-position authority.
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
        transform.translation.y =
            map.origin.y + terrain_surface_height(map.config, map.seed, Vec2::new(local.x, local.z));
    }

    for mut transform in &mut npcs {
        let local = transform.translation - map.origin;
        transform.translation.y = map.origin.y
            + terrain_surface_height(map.config, map.seed, Vec2::new(local.x, local.z))
            + PLAYER_VISUAL_HEIGHT;
    }
}

fn terrain_surface_height(config: ExplorationPortal, seed: u64, point: Vec2) -> f32 {
    let (q, r) = world_to_axial(point, config.tile_size);
    let center = hex_center(q, r, config.tile_size);
    let offset = point - center;
    let center_height = terrain_height(config, seed, center);
    if offset.length_squared() <= 1e-8 {
        return center_height;
    }

    let angle = offset.y.atan2(offset.x);
    let start_angle = std::f32::consts::FRAC_PI_6;
    let sector = ((angle - start_angle).rem_euclid(std::f32::consts::TAU)
        / (std::f32::consts::PI / 3.0))
        .floor() as usize
        % HEX_CORNER_COUNT;
    let a = hex_corner(center, config.tile_size, sector) - center;
    let b = hex_corner(center, config.tile_size, (sector + 1) % HEX_CORNER_COUNT) - center;
    let determinant = a.perp_dot(b);
    if determinant.abs() <= 1e-6 {
        return center_height;
    }

    let u = offset.perp_dot(b) / determinant;
    let v = a.perp_dot(offset) / determinant;
    let corner_a_height = terrain_height(config, seed, center + a);
    let corner_b_height = terrain_height(config, seed, center + b);
    center_height * (1.0 - u - v) + corner_a_height * u + corner_b_height * v
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
            seed.wrapping_add(
                (octave as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
            ),
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

fn hex_center(q: i32, r: i32, size: f32) -> Vec2 {
    Vec2::new(
        3.0_f32.sqrt() * size * (q as f32 + r as f32 * 0.5),
        1.5 * size * r as f32,
    )
}

fn hex_corner(center: Vec2, size: f32, index: usize) -> Vec2 {
    let angle = std::f32::consts::FRAC_PI_6
        + index as f32 * std::f32::consts::PI / 3.0;
    center + Vec2::new(angle.cos(), angle.sin()) * size
}

fn world_to_axial(point: Vec2, size: f32) -> (i32, i32) {
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
        }
        .sanitized();

        assert_eq!(config.radius, MAX_MAP_RADIUS);
        assert_eq!(config.tile_size, 1.5);
        assert_eq!(config.height_multiplier, 0.0);
        assert_eq!(config.terrain_frequency, 0.045);
        assert_eq!(config.biome_scale, 0.006);
        assert_eq!(config.vegetation_density, 0.8);
        assert_eq!(config.seed, Some(42));
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
