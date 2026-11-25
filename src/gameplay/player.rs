use bevy::{
    input::mouse::{AccumulatedMouseScroll, MouseScrollPixelsPerLine},
    prelude::*,
};

use crate::ui::stack::UiStack;

pub const CAMERA_DISTANCE: f32 = 18.0;
pub const CAMERA_HORIZONTAL_OFFSET: f32 = 0.45;
pub const CAMERA_DEFAULT_PITCH: f32 = -0.82;
pub const CAMERA_MIN_PITCH: f32 = -0.97;
pub const CAMERA_MAX_PITCH: f32 = -0.67;
const CAMERA_PITCH_SAVE_ADJUSTMENT: f32 = 0.08;
pub const INTERACTION_HEIGHT: f32 = 1.7;
const CAMERA_MIN_DISTANCE: f32 = 14.0;
const CAMERA_MAX_DISTANCE: f32 = 22.0;
const CAMERA_ZOOM_PER_LINE: f32 = 0.65;
const CAMERA_PITCH_PER_LINE: f32 = 0.02;
const CAMERA_TARGET_HEIGHT: f32 = 0.8;
const PLAYER_VISUAL_HEIGHT: f32 = 0.8;

#[derive(Component)]
pub struct Player {
    pub yaw: f32,
    /// Vertical angle of the follow camera. Negative values look down.
    pub pitch: f32,
}

#[derive(Component)]
pub(super) struct GameplayCamera {
    pub(super) distance: f32,
}

#[derive(Component)]
pub(super) struct PlayerVisual;

/// Scroll changes the follow camera's distance and vertical angle. Its
/// horizontal heading stays locked to the player's initial facing direction.
pub(super) fn adjust_camera(
    mut player: Query<&mut Player>,
    mut camera: Query<&mut GameplayCamera>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    scroll_conversion: Res<MouseScrollPixelsPerLine>,
    ui: Res<UiStack>,
) {
    if !ui.cursor_grabbed() {
        return;
    }

    let scroll = mouse_scroll.to_lines(&scroll_conversion).delta.y;
    if !scroll.is_finite() || scroll == 0.0 {
        return;
    }

    if let Ok(mut player) = player.single_mut() {
        player.pitch = camera_pitch_after_scroll(player.pitch, scroll);
    }
    if let Ok(mut camera) = camera.single_mut() {
        camera.distance = camera_distance_after_scroll(camera.distance, scroll);
    }
}

fn camera_pitch_after_scroll(pitch: f32, scroll: f32) -> f32 {
    (pitch + scroll * CAMERA_PITCH_PER_LINE).clamp(CAMERA_MIN_PITCH, CAMERA_MAX_PITCH)
}

fn camera_distance_after_scroll(distance: f32, scroll: f32) -> f32 {
    (distance - scroll * CAMERA_ZOOM_PER_LINE).clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE)
}

pub fn player_move(
    mut query: Query<(&Player, &mut Transform)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let sprinting = input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight);
    let speed = if sprinting { 8.0 } else { 5.0 };

    for (player, mut transform) in &mut query {
        let (forward, right) = camera_ground_basis(player.yaw);
        let mut dir = Vec3::ZERO;

        if input.pressed(KeyCode::KeyW) {
            dir += forward;
        }
        if input.pressed(KeyCode::KeyS) {
            dir -= forward;
        }
        if input.pressed(KeyCode::KeyA) {
            dir -= right;
        }
        if input.pressed(KeyCode::KeyD) {
            dir += right;
        }

        dir.y = 0.0;
        if dir != Vec3::ZERO {
            transform.translation += dir.normalize() * speed * dt;
        }
    }
}

/// Converts a pitch stored by the previous camera setup to the new, slightly
/// shallower default while preserving the saved zoom adjustment.
pub(super) fn migrate_saved_camera_pitch(pitch: f32) -> f32 {
    (pitch + CAMERA_PITCH_SAVE_ADJUSTMENT).clamp(CAMERA_MIN_PITCH, CAMERA_MAX_PITCH)
}

/// Horizontal movement axes projected from the camera view. The camera's
/// vertical pitch is intentionally ignored so movement stays on the ground.
fn camera_ground_basis(player_yaw: f32) -> (Vec3, Vec3) {
    let rotation = Quat::from_rotation_y(player_yaw + CAMERA_HORIZONTAL_OFFSET);
    (rotation * Vec3::NEG_Z, rotation * Vec3::X)
}

/// Builds the follow-camera orientation with a slight horizontal offset from
/// the player's heading, revealing the sides of world geometry.
pub(super) fn follow_camera_rotation(player_yaw: f32, pitch: f32) -> Quat {
    Quat::from_euler(
        EulerRot::YXZ,
        player_yaw + CAMERA_HORIZONTAL_OFFSET,
        pitch,
        0.0,
    )
}

/// Keeps the camera aimed at the player from a fixed, offset horizontal
/// heading. Scrolling only changes its distance and vertical viewing angle.
pub(super) fn follow_camera(
    player: Query<(&Transform, &Player)>,
    mut camera: Query<
        (&mut Transform, &GameplayCamera),
        (With<GameplayCamera>, Without<Player>),
    >,
) {
    let Ok((player_transform, player)) = player.single() else {
        return;
    };
    let Ok((mut camera_transform, camera_settings)) = camera.single_mut() else {
        return;
    };

    let target = player_transform.translation + Vec3::Y * CAMERA_TARGET_HEIGHT;
    let orientation = follow_camera_rotation(player.yaw, player.pitch);
    let forward = orientation * Vec3::NEG_Z;
    camera_transform.translation = target - forward * camera_settings.distance;
    camera_transform.look_at(target, Vec3::Y);
}

/// Projects the player's ground position to the separate billboard entity.
pub(super) fn sync_player_visual(
    player: Query<&Transform, With<Player>>,
    mut visual: Query<
        &mut Transform,
        (With<PlayerVisual>, Without<Player>),
    >,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut visual_transform) = visual.single_mut() else {
        return;
    };

    visual_transform.translation =
        player_transform.translation + Vec3::Y * PLAYER_VISUAL_HEIGHT;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_forward_matches_the_camera_horizontal_heading() {
        let (movement_forward, movement_right) = camera_ground_basis(0.3);
        let camera_forward =
            (follow_camera_rotation(0.3, CAMERA_DEFAULT_PITCH) * Vec3::NEG_Z)
                .with_y(0.0)
                .normalize();
        let camera_right = follow_camera_rotation(0.3, CAMERA_DEFAULT_PITCH) * Vec3::X;

        assert!(movement_forward.abs_diff_eq(camera_forward, 1e-6));
        assert!(movement_right.abs_diff_eq(camera_right, 1e-6));
        assert!(movement_forward.y.abs() < 1e-6);
        assert!(movement_right.y.abs() < 1e-6);
    }

    #[test]
    fn saved_camera_pitch_shifts_with_the_new_default() {
        assert!((migrate_saved_camera_pitch(-0.9) - CAMERA_DEFAULT_PITCH).abs() < 1e-6);
        assert!(
            (migrate_saved_camera_pitch(-1.05) - CAMERA_MIN_PITCH).abs() < 1e-6
        );
        assert!(
            (migrate_saved_camera_pitch(-0.75) - CAMERA_MAX_PITCH).abs() < 1e-6
        );
    }

    #[test]
    fn scrolling_in_zooms_in_and_flattens_the_view() {
        assert!(camera_distance_after_scroll(CAMERA_DISTANCE, 1.0) < CAMERA_DISTANCE);
        assert!(camera_pitch_after_scroll(CAMERA_DEFAULT_PITCH, 1.0) > CAMERA_DEFAULT_PITCH);
    }

    #[test]
    fn scrolling_out_zooms_out_and_raises_the_view() {
        assert!(camera_distance_after_scroll(CAMERA_DISTANCE, -1.0) > CAMERA_DISTANCE);
        assert!(camera_pitch_after_scroll(CAMERA_DEFAULT_PITCH, -1.0) < CAMERA_DEFAULT_PITCH);
    }

    #[test]
    fn camera_adjustments_stay_within_their_limits() {
        assert_eq!(
            camera_distance_after_scroll(CAMERA_MIN_DISTANCE, 10.0),
            CAMERA_MIN_DISTANCE
        );
        assert_eq!(
            camera_distance_after_scroll(CAMERA_MAX_DISTANCE, -10.0),
            CAMERA_MAX_DISTANCE
        );
        assert_eq!(
            camera_pitch_after_scroll(CAMERA_MIN_PITCH, -10.0),
            CAMERA_MIN_PITCH
        );
        assert_eq!(
            camera_pitch_after_scroll(CAMERA_MAX_PITCH, 10.0),
            CAMERA_MAX_PITCH
        );
    }
}
