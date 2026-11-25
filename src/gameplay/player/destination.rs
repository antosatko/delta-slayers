use bevy::prelude::*;

use super::{MovementTarget, Player};

const TRAJECTORY_HEIGHT: f32 = 0.12;

pub(super) const TRAJECTORY_ARROW_SPACING: f32 = 1.2;
pub(super) const TRAJECTORY_ARROW_LENGTH: f32 = 0.72;
pub(super) const TRAJECTORY_ARROW_SPEED: f32 = 0.85;
pub(super) const DESTINATION_RING_RADIUS: f32 = 0.48;
pub(super) const DESTINATION_BEACON_HEIGHT: f32 = 0.72;

pub(super) fn draw_move_trajectory(
    mut gizmos: Gizmos,
    player: Query<&Transform, With<Player>>,
    target: Res<MovementTarget>,
    time: Res<Time>,
) {
    let Some(destination) = target.destination else {
        return;
    };
    let Ok(player_transform) = player.single() else {
        return;
    };

    let start = player_transform.translation + Vec3::Y * TRAJECTORY_HEIGHT;
    let end = destination + Vec3::Y * TRAJECTORY_HEIGHT;
    let mut offset = end - start;
    offset.y = 0.0;
    let distance = offset.length();
    if distance <= 0.03 {
        return;
    }

    let direction = offset / distance;
    let arrow_color = Color::srgb(0.38, 0.78, 1.0);
    let phase = trajectory_arrow_phase(time.elapsed_secs());
    let mut arrow_start_distance = phase;

    // Arrows advance from the player toward the destination, then re-enter
    // from behind to create a continuous, slowly flowing trajectory.
    while arrow_start_distance < distance {
        let arrow_end_distance = (arrow_start_distance + TRAJECTORY_ARROW_LENGTH).min(distance);
        if arrow_end_distance - arrow_start_distance >= 0.12 {
            let arrow_start = start + direction * arrow_start_distance;
            let arrow_end = start + direction * arrow_end_distance;
            gizmos
                .arrow(arrow_start, arrow_end, arrow_color)
                .with_tip_length(0.18);
        }
        arrow_start_distance += TRAJECTORY_ARROW_SPACING;
    }

    // The ground ring and vertical beacon distinguish the selected endpoint.
    let destination_color = Color::srgb(1.0, 0.78, 0.28);
    let ring_center = end + Vec3::Y * 0.025;
    gizmos
        .circle(
            Isometry3d::new(
                ring_center,
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
            ),
            DESTINATION_RING_RADIUS,
            destination_color,
        )
        .resolution(48);
    gizmos.line(
        end + Vec3::Y * 0.08,
        end + Vec3::Y * DESTINATION_BEACON_HEIGHT,
        destination_color,
    );
}

fn trajectory_arrow_phase(elapsed_secs: f32) -> f32 {
    (elapsed_secs * TRAJECTORY_ARROW_SPEED).rem_euclid(TRAJECTORY_ARROW_SPACING)
}

/// Converts a pitch stored by the previous camera setup to the new, slightly
/// shallower default while preserving the saved zoom adjustment.

