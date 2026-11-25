use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollPixelsPerLine},
    prelude::*,
    ui::RelativeCursorPosition,
};

use crate::ui::stack::UiStack;

use super::{Player, PlayerSettings};

pub const CAMERA_DISTANCE: f32 = 18.0;
pub const CAMERA_HORIZONTAL_OFFSET: f32 = 0.45;
pub const CAMERA_DEFAULT_PITCH: f32 = -0.82;
pub const CAMERA_MIN_PITCH: f32 = -1.45;
pub const CAMERA_MAX_PITCH: f32 = -0.15;
pub(super) const CAMERA_MIN_DISTANCE: f32 = 14.0;
pub(super) const CAMERA_MAX_DISTANCE: f32 = 22.0;

const CAMERA_PITCH_SAVE_ADJUSTMENT: f32 = 0.08;
const CAMERA_TARGET_HEIGHT: f32 = 0.8;

/// Camera transform state, kept separate from the player's movement transform.
#[derive(Component)]
pub(crate) struct GameplayCamera {
    pub(crate) distance: f32,
    /// World-space translation applied to the camera and its look target.
    pub(crate) pan_offset: Vec3,
}

pub(super) fn adjust_camera(
    mut player: Query<&mut Player>,
    mut camera: Query<&mut GameplayCamera>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    scroll_conversion: Res<MouseScrollPixelsPerLine>,
    settings: Res<PlayerSettings>,
    ui_surfaces: Query<(&RelativeCursorPosition, &InheritedVisibility)>,
    mut pan_blocked_until_release: Local<bool>,
) {
    let pointer_over_ui = ui_surfaces
        .iter()
        .any(|(cursor, visibility)| visibility.get() && cursor.cursor_over);

    if mouse_buttons.just_pressed(MouseButton::Middle) {
        *pan_blocked_until_release = pointer_over_ui;
    }
    if mouse_buttons.pressed(MouseButton::Middle) && pointer_over_ui {
        *pan_blocked_until_release = true;
    }
    if mouse_buttons.just_released(MouseButton::Middle) {
        *pan_blocked_until_release = false;
    }

    let motion = mouse_motion.delta;
    if mouse_buttons.pressed(MouseButton::Middle)
        && !*pan_blocked_until_release
        && motion.is_finite()
        && motion != Vec2::ZERO
    {
        if let (Ok(player), Ok(mut camera)) = (player.single_mut(), camera.single_mut()) {
            let orientation = follow_camera_rotation(player.yaw, player.pitch);
            let right = horizontal_camera_axis(orientation * Vec3::X);
            let up = horizontal_camera_axis(orientation * Vec3::Y);
            let scale = camera.distance * settings.camera_pan_sensitivity;
            camera.pan_offset =
                camera_pan_offset_after_drag(camera.pan_offset, motion, right, up, scale);
        }
    }

    let scroll = mouse_scroll.to_lines(&scroll_conversion).delta.y;
    if !scroll.is_finite() || scroll == 0.0 {
        return;
    }

    if let Ok(mut player) = player.single_mut() {
        player.pitch = camera_pitch_after_scroll(player.pitch, scroll, settings.camera_pitch_sensitivity);
    }
    if let Ok(mut camera) = camera.single_mut() {
        camera.distance = camera_distance_after_scroll(camera.distance, scroll, settings.camera_zoom_sensitivity, settings.camera_min_distance, settings.camera_max_distance);
    }
}

pub(super) fn camera_pitch_after_scroll(pitch: f32, scroll: f32, sensitivity: f32) -> f32 {
    (pitch + scroll * sensitivity).clamp(CAMERA_MIN_PITCH, CAMERA_MAX_PITCH)
}

fn horizontal_camera_axis(axis: Vec3) -> Vec3 {
    Vec3::new(axis.x, 0.0, axis.z).normalize_or_zero()
}

pub(super) fn camera_pan_offset_after_drag(
    offset: Vec3,
    delta: Vec2,
    camera_right: Vec3,
    camera_up: Vec3,
    scale: f32,
) -> Vec3 {
    let offset = offset.with_y(0.0);
    offset - horizontal_camera_axis(camera_right) * delta.x * scale
        + horizontal_camera_axis(camera_up) * delta.y * scale
}

pub(super) fn camera_distance_after_scroll(distance: f32, scroll: f32, sensitivity: f32, min: f32, max: f32) -> f32 {
    (distance - scroll * sensitivity).clamp(min, max)
}

/// A quick click sets one destination. Holding the button for a short
/// delay switches to periodic retargeting from the current cursor position.

pub(super) fn migrate_saved_camera_pitch(pitch: f32) -> f32 {
    (pitch + CAMERA_PITCH_SAVE_ADJUSTMENT).clamp(CAMERA_MIN_PITCH, CAMERA_MAX_PITCH)
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

/// Keeps the camera following the player while preserving a world-space pan
/// offset. Panning translates the camera and focus together without rotation.
pub(super) fn follow_camera(
    player: Query<(&Transform, &Player)>,
    mut camera: Query<(&mut Transform, &mut GameplayCamera), (With<GameplayCamera>, Without<Player>)>,
    settings: Res<PlayerSettings>,
) {
    let Ok((player_transform, player)) = player.single() else {
        return;
    };
    let Ok((mut camera_transform, mut camera_settings)) = camera.single_mut() else {
        return;
    };

    camera_settings.distance = camera_settings
        .distance
        .clamp(settings.camera_min_distance, settings.camera_max_distance);
    let target = player_transform.translation
        + Vec3::Y * CAMERA_TARGET_HEIGHT
        + camera_settings.pan_offset;
    let orientation = follow_camera_rotation(player.yaw, player.pitch);
    let forward = orientation * Vec3::NEG_Z;
    camera_transform.translation = target - forward * camera_settings.distance;
    camera_transform.look_at(target, Vec3::Y);
}

/// Projects the player's ground position to the separate billboard entity.

