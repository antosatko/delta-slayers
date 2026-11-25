use bevy::prelude::*;

use super::{
    player::{self, MovementBlocker, PLAYER_COLLIDER_RADIUS, PLAYER_VISUAL_HEIGHT, Player},
    spawning::WorldStartup,
};

const GODO_CHASE_SPEED: f32 = 2.4;
const GODO_STOP_DISTANCE: f32 = PLAYER_COLLIDER_RADIUS * 2.0 + 0.45;

/// The Godo NPC. Its transform is positioned at the center of its billboard;
/// movement and collision use the corresponding ground position.
#[derive(Component)]
pub(super) struct Npc;

/// Spawns Godo beside the player's final initial position. This runs after
/// authored spawn-point placement, so new games and loaded saves both start
/// with the NPC nearby.
pub(super) fn spawn_godo(
    mut commands: Commands,
    startup: Res<WorldStartup>,
    player: Query<&Transform, With<Player>>,
    existing: Query<(), With<Npc>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !startup.spawn_point_applied || !existing.is_empty() {
        return;
    }
    let Ok(player) = player.single() else {
        return;
    };

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("npc/godo.png")),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(0.95, 1.68))),
        MeshMaterial3d(material),
        Transform::from_translation(player.translation + Vec3::new(3.0, PLAYER_VISUAL_HEIGHT, 3.0)),
        Npc,
        super::billboard::Billboard,
        super::billboard::SquashOnMove,
        DespawnOnExit(crate::app::state::AppState::InGame),
    ));
}

/// Moves Godo toward the player while respecting the same authored blockers
/// and collision resolution used by player movement.
pub(super) fn chase_player(
    time: Res<Time>,
    player: Query<&Transform, (With<Player>, Without<Npc>)>,
    blockers: Query<&GlobalTransform, With<MovementBlocker>>,
    mut npcs: Query<&mut Transform, (With<Npc>, Without<Player>)>,
) {
    let Ok(player) = player.single() else {
        return;
    };

    let delta_secs = time.delta_secs();
    if !delta_secs.is_finite() || delta_secs <= 0.0 {
        return;
    }

    for mut npc in &mut npcs {
        let ground_position = npc.translation - Vec3::Y * PLAYER_VISUAL_HEIGHT;
        let movement = chase_step(ground_position, player.translation, delta_secs);
        if movement == Vec3::ZERO {
            continue;
        }
        let next_position = player::move_with_collisions(
            ground_position,
            movement,
            |candidate, intended_movement| {
                blockers.iter().find_map(|blocker| {
                    let contact = player::movement_collision_contact(candidate, blocker)?;
                    (intended_movement.dot(contact.normal) < -1e-5).then_some(contact)
                })
            },
        );

        npc.translation.x = next_position.x;
        npc.translation.z = next_position.z;
    }
}

fn chase_step(position: Vec3, target: Vec3, delta_secs: f32) -> Vec3 {
    if !delta_secs.is_finite() || delta_secs <= 0.0 {
        return Vec3::ZERO;
    }

    let mut offset = target - position;
    offset.y = 0.0;
    let distance = offset.length();
    let remaining_distance = distance - GODO_STOP_DISTANCE;
    if remaining_distance <= 0.0 || distance <= f32::EPSILON {
        return Vec3::ZERO;
    }

    offset / distance * (GODO_CHASE_SPEED * delta_secs).min(remaining_distance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npc_stops_at_its_personal_space_distance() {
        let movement = chase_step(Vec3::ZERO, Vec3::X * (GODO_STOP_DISTANCE - 0.1), 1.0);
        assert_eq!(movement, Vec3::ZERO);
    }

    #[test]
    fn npc_chase_step_is_speed_limited_and_horizontal() {
        let movement = chase_step(Vec3::ZERO, Vec3::new(100.0, 50.0, 0.0), 1.0);
        assert!((movement.length() - GODO_CHASE_SPEED).abs() < 1e-5);
        assert_eq!(movement.y, 0.0);
        assert!(movement.x > 0.0);
    }

    #[test]
    fn npc_reaches_but_does_not_cross_its_stop_distance() {
        let target = Vec3::X * (GODO_STOP_DISTANCE + 0.2);
        let movement = chase_step(Vec3::ZERO, target, 1.0);
        assert!((movement.length() - 0.2).abs() < 1e-5);
    }
}
