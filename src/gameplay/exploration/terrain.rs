use bevy::prelude::*;

use crate::gameplay::{
    npc::Npc,
    player::{PLAYER_VISUAL_HEIGHT, Player},
};

use super::hex::{TILE_RADIUS_METERS, hex_center};
use super::{ActiveExplorationMap, Biome, ExplorationPortal};

const TERRAIN_OCTAVES: usize = 4;

/// Keeps the player and Godo on the smoothstep-interpolated terrain surface.
/// The player transform remains the ground-position authority.
pub(in crate::gameplay) fn sync_ground_height(
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

pub(super) fn terrain_surface_height(config: ExplorationPortal, seed: u64, point: Vec2) -> f32 {
    let size = TILE_RADIUS_METERS;
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

pub(super) fn terrain_height(config: ExplorationPortal, seed: u64, point: Vec2) -> f32 {
    fractal_noise(
        point.x * config.terrain_frequency,
        point.y * config.terrain_frequency,
        seed,
    ) * config.height_multiplier
}

pub(super) fn biome_at(config: ExplorationPortal, seed: u64, point: Vec2) -> Biome {
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

pub(super) fn fractal_noise(x: f32, z: f32, seed: u64) -> f32 {
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

pub(super) fn cell_random(seed: u64, q: i32, r: i32, salt: u64) -> f32 {
    (lattice_noise(seed ^ salt, q as i64, r as i64) + 1.0) * 0.5
}

fn smoothstep(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
