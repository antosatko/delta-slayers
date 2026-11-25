use bevy::prelude::*;

use super::player::{
    self, MovementBlocker, Player, PLAYER_COLLIDER_RADIUS, PLAYER_VISUAL_HEIGHT,
};

const GODO_CHASE_SPEED: f32 = 2.4;
const GODO_STOP_DISTANCE: f32 = PLAYER_COLLIDER_RADIUS * 2.0 + 0.45;

/// The Godo NPC. Its transform is positioned at the center of its billboard;
/// movement and collision use the corresponding ground position.
#[derive(Component)]
pub(super) struct Npc;

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
        let mut offset = player.translation - ground_position;
        offset.y = 0.0;
        let distance = offset.length();
        let remaining_distance = distance - GODO_STOP_DISTANCE;
        if remaining_distance <= 0.0 || distance <= f32::EPSILON {
            continue;
        }

        let movement_distance = (GODO_CHASE_SPEED * delta_secs).min(remaining_distance);
        let movement = offset / distance * movement_distance;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npc_stops_chasing_inside_its_personal_space() {
        let distance = GODO_STOP_DISTANCE - 0.1;
        assert!(distance - GODO_STOP_DISTANCE <= 0.0);
    }

    #[test]
    fn npc_chase_speed_is_slower_than_player_walk_speed() {
        assert!(GODO_CHASE_SPEED < 5.0);
    }
}
