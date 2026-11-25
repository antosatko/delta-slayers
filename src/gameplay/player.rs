use bevy::{
    input::mouse::{AccumulatedMouseScroll, MouseScrollPixelsPerLine},
    prelude::*,
};

use crate::ui::stack::UiStack;

pub const CAMERA_DISTANCE: f32 = 18.0;
pub const CAMERA_DEFAULT_PITCH: f32 = -0.9;
pub const CAMERA_MIN_PITCH: f32 = -1.05;
pub const CAMERA_MAX_PITCH: f32 = -0.75;
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
    mut query: Query<&mut Transform, With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let sprinting = input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight);
    let speed = if sprinting { 8.0 } else { 5.0 };

    for mut transform in &mut query {
        let forward = *transform.forward();
        let right = *transform.right();
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

/// Keeps the camera aimed at the player from a fixed horizontal heading.
/// Scrolling only changes its distance and vertical viewing angle.
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
    let orientation = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);
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
        assert_eq!(camera_distance_after_scroll(CAMERA_MIN_DISTANCE, 10.0), CAMERA_MIN_DISTANCE);
        assert_eq!(camera_distance_after_scroll(CAMERA_MAX_DISTANCE, -10.0), CAMERA_MAX_DISTANCE);
        assert_eq!(camera_pitch_after_scroll(CAMERA_MIN_PITCH, -10.0), CAMERA_MIN_PITCH);
        assert_eq!(camera_pitch_after_scroll(CAMERA_MAX_PITCH, 10.0), CAMERA_MAX_PITCH);
    }
}
