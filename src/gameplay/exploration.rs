use bevy::prelude::*;

mod generation;
mod hex;
mod terrain;
mod travel;
mod vegetation;

pub use generation::apply_tile_materials;
pub(crate) use vegetation::{VegetationTuning, apply_tuning as apply_vegetation_tuning, ensure_player_interactor};
pub(crate) use hex::{hex_center, world_to_axial};
pub(super) use terrain::sync_ground_height;
pub(crate) use travel::generate_seed;
pub(super) use travel::{detect_portal, process_travel_request};

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
    /// The runtime's F4 vegetation lab controls grass density instead.
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

fn finite_clamp(value: f32, fallback: f32, min: f32, max: f32) -> f32 {
    if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests;
