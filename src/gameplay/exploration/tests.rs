use super::*;
use std::collections::HashMap;

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
