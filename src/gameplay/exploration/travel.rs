use bevy::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    app::state::AppState,
    gameplay::{
        environment::EnvironmentAssets,
        npc::Npc,
        player::{self, PLAYER_VISUAL_HEIGHT, Player},
    },
};

use super::generation::{ExplorationMapRoot, spawn_map_geometry};
use super::hex::{TILE_RADIUS_METERS, hex_center};
use super::terrain::terrain_surface_height;
use super::{
    ActiveExplorationMap, ActiveMapData, ExplorationPortal, ExplorationReturnPortal,
    ExplorationSession, ExplorationTravelRequest, NearbyExplorationPortal, PortalAction,
    TravelAnchor,
};

const MAP_WORLD_OFFSET: f32 = 512.0;

pub(in crate::gameplay) fn detect_portal(
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

pub(in crate::gameplay) fn process_travel_request(
    mut commands: Commands,
    environment: Res<EnvironmentAssets>,
    mut request: ResMut<ExplorationTravelRequest>,
    mut active: ResMut<ActiveExplorationMap>,
    mut session: ResMut<ExplorationSession>,
    mut movement_target: ResMut<player::MovementTarget>,
    mut player: Query<(&mut Transform, &mut Player), Without<Npc>>,
    mut npcs: Query<&mut Transform, (With<Npc>, Without<Player>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(action) = request.0 else {
        return;
    };
    if matches!(action, PortalAction::Enter(_)) && !environment.is_ready() {
        return;
    }
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

            spawn_map_geometry(
                &mut commands,
                root,
                config,
                seed,
                &environment,
                &mut meshes,
                &mut materials,
            );

            let spawn_local = hex_center(0, 2, TILE_RADIUS_METERS);
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
                config.radius, TILE_RADIUS_METERS
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

pub(crate) fn generate_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0xD37A_51A7_0B5E_ED01)
}
