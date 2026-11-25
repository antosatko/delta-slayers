use std::collections::HashMap;

use bevy::prelude::*;
use bevy_grass::prelude::*;

use crate::gameplay::player::Player;

use super::{
    ActiveExplorationMap, Biome, ExplorationPortal,
    generation::GeneratedTile,
    hex::{HEX_NEIGHBORS, hex_center, hex_corner, world_to_axial},
    terrain::fractal_noise,
};

// One-metre masks avoid visible square cells on the six-metre hex tiles.
const MASK_CELL_SIZE: f32 = 1.0;
const TILE_MODEL_SOURCE_RADIUS: f32 = 5.0;
const CLIFF_FADE_WIDTH: f32 = 1.35;
const NEIGHBOR_EDGE: [usize; 6] = [5, 4, 3, 2, 1, 0];
const BIOME_NOISE_SALT: u64 = 0x6A09_E667_F3BC_C909;

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub(crate) struct VegetationTuning {
    pub(crate) global_density: f32,
    pub(crate) view_distance: f32,
    pub(crate) blade_height: f32,
    pub(crate) grassland_density: f32,
    pub(crate) woodland_density: f32,
    pub(crate) dryland_density: f32,
    pub(crate) highland_density: f32,
    pub(crate) wind_strength: f32,
    pub(crate) wind_speed: f32,
    pub(crate) shadows: bool,
}

impl Default for VegetationTuning {
    fn default() -> Self {
        Self {
            global_density: 0.72,
            view_distance: 36.0,
            blade_height: 0.88,
            grassland_density: 0.9,
            woodland_density: 0.9,
            dryland_density: 0.62,
            highland_density: 0.58,
            wind_strength: 0.36,
            wind_speed: 4.0,
            shadows: false,
        }
    }
}

#[derive(Component, Clone, Copy)]
struct BiomeVegetationField(Biome);

/// Builds four overlapping, biome-weighted fields. Each field spans the whole
/// map, while its density mask selects a smoothly blended biome contribution.
pub(super) fn spawn_vegetation(
    commands: &mut Commands,
    root: Entity,
    config: ExplorationPortal,
    seed: u64,
    tiles: &HashMap<(i32, i32), GeneratedTile>,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    tuning: &VegetationTuning,
) {
    let half_extent = (3.0_f32.sqrt() * (config.radius as f32 + 0.5) + 1.0) * config.tile_size;
    let field_size = Vec2::splat(half_extent * 2.0);
    let ground = GrassField {
        size: field_size,
        style: asset_server.load("grass_styles/wildflower_meadow.grass.ron"),
        ..default()
    }
    .with_ground_from_fn(images, MASK_CELL_SIZE, |point| {
        tile_ground_height(tiles, point, config.tile_size)
    });
    let shared_ground = ground.ground.clone();

    let grassland = ground.with_density_from_fn(images, MASK_CELL_SIZE, |point| {
        biome_density(
            tiles,
            config,
            seed,
            point,
            Biome::Grassland,
        )
    });
    spawn_biome_field(commands, root, grassland, Biome::Grassland, tuning);

    spawn_masked_field(
        commands,
        root,
        field_size,
        shared_ground.clone(),
        asset_server.load("grass_styles/forest_floor.grass.ron"),
        Biome::Woodland,
        config,
        seed,
        tiles,
        images,
        tuning,
    );
    spawn_masked_field(
        commands,
        root,
        field_size,
        shared_ground.clone(),
        asset_server.load("grass_styles/dune_marram.grass.ron"),
        Biome::Dryland,
        config,
        seed,
        tiles,
        images,
        tuning,
    );
    spawn_masked_field(
        commands,
        root,
        field_size,
        shared_ground,
        asset_server.load("grass_styles/frosted_meadow.grass.ron"),
        Biome::Highlands,
        config,
        seed,
        tiles,
        images,
        tuning,
    );
}

#[allow(clippy::too_many_arguments)]
fn spawn_masked_field(
    commands: &mut Commands,
    root: Entity,
    field_size: Vec2,
    ground: GrassGround,
    style: Handle<GrassStyle>,
    biome: Biome,
    config: ExplorationPortal,
    seed: u64,
    tiles: &HashMap<(i32, i32), GeneratedTile>,
    images: &mut Assets<Image>,
    tuning: &VegetationTuning,
) {
    let field = GrassField {
        size: field_size,
        style,
        ground,
        ..default()
    };
    let field = field.with_density_from_fn(images, MASK_CELL_SIZE, |point| {
        biome_density(tiles, config, seed, point, biome)
    });
    spawn_biome_field(commands, root, field, biome, tuning);
}

fn spawn_biome_field(
    commands: &mut Commands,
    root: Entity,
    field: GrassField,
    biome: Biome,
    tuning: &VegetationTuning,
) {
    commands.spawn((
        field,
        field_overrides(biome, tuning),
        BiomeVegetationField(biome),
        Transform::IDENTITY,
        ChildOf(root),
    ));
}

fn field_overrides(biome: Biome, tuning: &VegetationTuning) -> GrassFieldOverrides {
    let density_scale = match biome {
        Biome::Grassland => tuning.grassland_density,
        Biome::Woodland => tuning.woodland_density,
        Biome::Dryland => tuning.dryland_density,
        Biome::Highlands => tuning.highland_density,
    };
    GrassFieldOverrides {
        tint: None,
        density_scale,
        height_scale: tuning.blade_height,
        wind_scale: 1.0,
    }
}

/// The rendered tiles are individually flat. Grass must use those top planes,
/// not the smoothed movement surface used by the player and NPCs.
fn tile_ground_height(
    tiles: &HashMap<(i32, i32), GeneratedTile>,
    point: Vec2,
    tile_size: f32,
) -> f32 {
    let coordinates = world_to_axial(point, tile_size);
    let Some(tile) = tiles.get(&coordinates) else {
        return f32::NAN;
    };
    let center = hex_center(coordinates.0, coordinates.1, tile_size);
    if !inside_hex(point - center, tile_size) {
        return f32::NAN;
    }

    let model_scale = tile_size / TILE_MODEL_SOURCE_RADIUS;
    tile.height + 0.016_821_861 * model_scale
}

fn biome_density(
    tiles: &HashMap<(i32, i32), GeneratedTile>,
    config: ExplorationPortal,
    seed: u64,
    point: Vec2,
    biome: Biome,
) -> f32 {
    let coordinates = world_to_axial(point, config.tile_size);
    let Some(tile) = tiles.get(&coordinates) else {
        return 0.0;
    };
    let center = hex_center(coordinates.0, coordinates.1, config.tile_size);
    let local = point - center;
    if !inside_hex(local, config.tile_size)
        || tile.has_feature
        || point.length() < config.tile_size * 1.6
        || point.distance(hex_center(0, 2, config.tile_size)) < config.tile_size * 1.75
    {
        return 0.0;
    }

    let (elevation, moisture) = climate_values(config, seed, point);
    let weights = biome_weights(elevation, moisture);
    let biome_weight = match biome {
        Biome::Grassland => weights[0],
        Biome::Woodland => weights[1],
        Biome::Dryland => weights[2],
        Biome::Highlands => weights[3],
    };

    let cliff_fade = cliff_edge_fade(tiles, coordinates, center, point, config);

    (biome_weight * cliff_fade).clamp(0.0, 1.0)
}

fn climate_values(config: ExplorationPortal, seed: u64, point: Vec2) -> (f32, f32) {
    let elevation = fractal_noise(
        point.x * config.terrain_frequency,
        point.y * config.terrain_frequency,
        seed,
    );
    let moisture = fractal_noise(
        point.x * config.biome_scale,
        point.y * config.biome_scale,
        seed ^ BIOME_NOISE_SALT,
    );
    (elevation, moisture)
}

/// Smooth biome weights add to one everywhere, so neighbouring styles blend
/// instead of exposing the discrete biome boundaries between hex tiles.
fn biome_weights(elevation: f32, moisture: f32) -> [f32; 4] {
    let highlands = smoothstep((elevation - 0.34) / 0.28);
    let wet = smoothstep((moisture - 0.02) / 0.58);
    let dry = smoothstep((-moisture - 0.08) / 0.58);

    let grassland = (1.0 - wet) * (1.0 - dry);
    let woodland = wet;
    let dryland = dry;
    let lowland_total = (grassland + woodland + dryland).max(f32::EPSILON);
    let lowland = 1.0 - highlands;

    [
        grassland / lowland_total * lowland,
        woodland / lowland_total * lowland,
        dryland / lowland_total * lowland,
        highlands,
    ]
}

fn cliff_edge_fade(
    tiles: &HashMap<(i32, i32), GeneratedTile>,
    coordinates: (i32, i32),
    center: Vec2,
    point: Vec2,
    config: ExplorationPortal,
) -> f32 {
    let Some(tile) = tiles.get(&coordinates) else {
        return 0.0;
    };
    let mut fade: f32 = 1.0;

    for (index, (dq, dr)) in HEX_NEIGHBORS.into_iter().enumerate() {
        let neighbor = tiles.get(&(coordinates.0 + dq, coordinates.1 + dr));
        let steep = neighbor.is_none_or(|other| {
            (other.height - tile.height).abs() > config.max_jump_height
        });
        if !steep {
            continue;
        }

        let side = NEIGHBOR_EDGE[index];
        let a = hex_corner(center, config.tile_size, side);
        let b = hex_corner(
            center,
            config.tile_size,
            (side + 1) % 6,
        );
        let midpoint = (a + b) * 0.5;
        let inward = (center - midpoint).normalize_or_zero();
        let distance_from_edge = (point - midpoint).dot(inward).max(0.0);
        fade = fade.min(smoothstep(distance_from_edge / CLIFF_FADE_WIDTH));
    }

    fade
}

fn inside_hex(local: Vec2, size: f32) -> bool {
    let x = local.x.abs();
    let y = local.y.abs();
    x <= size * 3.0_f32.sqrt() * 0.5 + 0.02
        && y <= size - x / 3.0_f32.sqrt() + 0.02
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

/// Updates global render cost and per-biome field overrides from the tuning model.
pub(crate) fn apply_tuning(
    tuning: Res<VegetationTuning>,
    mut settings: ResMut<GrassSettings>,
    mut wind: ResMut<GrassWind>,
    mut fields: Query<(&BiomeVegetationField, &mut GrassFieldOverrides)>,
) {
    if !tuning.is_changed() {
        return;
    }

    settings.radius = tuning.view_distance.clamp(18.0, 52.0);
    settings.density_scale = tuning.global_density.clamp(0.0, 1.4);
    settings.shadows = if tuning.shadows {
        GrassShadows::Near
    } else {
        GrassShadows::Off
    };
    wind.strength = tuning.wind_strength.clamp(0.0, 1.0);
    wind.gust_speed = tuning.wind_speed.clamp(0.0, 12.0);

    for (field, mut overrides) in &mut fields {
        let next = field_overrides(field.0, &tuning);
        if *overrides != next {
            *overrides = next;
        }
    }
}

/// Only the player needs to stamp a trail initially; adding this marker is
/// conditional on an exploration map existing, so the authored town is untouched.
pub(crate) fn ensure_player_interactor(
    active_map: Res<ActiveExplorationMap>,
    players: Query<Entity, (With<Player>, Without<GrassInteractor>)>,
    mut commands: Commands,
) {
    if active_map.0.is_none() {
        return;
    }

    for player in &players {
        commands.entity(player).insert(GrassInteractor {
            radius: 0.72,
            strength: 0.82,
            max_height: Some(1.3),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biome_weights_are_normalized_and_non_negative() {
        for elevation in [-1.0, -0.4, 0.0, 0.4, 0.9] {
            for moisture in [-1.0, -0.2, 0.0, 0.3, 1.0] {
                let weights = biome_weights(elevation, moisture);
                assert!(weights.iter().all(|weight| *weight >= 0.0));
                assert!((weights.iter().sum::<f32>() - 1.0).abs() < 0.0001);
            }
        }
    }

    #[test]
    fn hex_mask_includes_center_and_excludes_points_beyond_edges() {
        assert!(inside_hex(Vec2::ZERO, 6.0));
        assert!(inside_hex(Vec2::new(5.0, 0.0), 6.0));
        assert!(!inside_hex(Vec2::new(6.0, 0.0), 6.0));
        assert!(!inside_hex(Vec2::new(0.0, 6.0), 6.0));
    }
}
