use bevy::{
    input::mouse::{AccumulatedMouseScroll, MouseScrollPixelsPerLine},
    prelude::*,
    window::PrimaryWindow,
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
const PLAYER_WALK_SPEED: f32 = 5.0;
const PLAYER_SPRINT_SPEED: f32 = 8.0;
const TRAJECTORY_ARROW_SPACING: f32 = 1.2;
const TRAJECTORY_ARROW_LENGTH: f32 = 0.72;
const TRAJECTORY_HEIGHT: f32 = 0.12;
const TRAJECTORY_ARROW_SPEED: f32 = 0.85;
const DESTINATION_RING_RADIUS: f32 = 0.48;
const DESTINATION_BEACON_HEIGHT: f32 = 0.72;
const MOVE_TARGET_HOLD_DELAY: f32 = 0.3;
const MOVE_TARGET_REFRESH_INTERVAL: f32 = 0.1;

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

/// The current ground destination plus click-and-hold retargeting state.
#[derive(Resource, Default)]
pub(super) struct MovementTarget {
    pub(super) destination: Option<Vec3>,
    hold_elapsed: f32,
    refresh_elapsed: f32,
    ignore_click_until_release: bool,
}

impl MovementTarget {
    pub(super) fn clear(&mut self) {
        self.destination = None;
        self.hold_elapsed = 0.0;
        self.refresh_elapsed = 0.0;
        self.ignore_click_until_release = false;
    }

    fn should_update_from_cursor(
        &mut self,
        pressed: bool,
        just_pressed: bool,
        delta_secs: f32,
        blocked_by_ui: bool,
    ) -> bool {
        if just_pressed {
            self.hold_elapsed = 0.0;
            self.refresh_elapsed = 0.0;
            self.ignore_click_until_release = blocked_by_ui;
            return !blocked_by_ui;
        }

        if !pressed {
            self.hold_elapsed = 0.0;
            self.refresh_elapsed = 0.0;
            self.ignore_click_until_release = false;
            return false;
        }

        if blocked_by_ui {
            self.ignore_click_until_release = true;
            return false;
        }
        if self.ignore_click_until_release {
            return false;
        }

        self.hold_elapsed += delta_secs;
        if self.hold_elapsed < MOVE_TARGET_HOLD_DELAY {
            return false;
        }

        self.refresh_elapsed += delta_secs;
        if self.refresh_elapsed + 1e-5 >= MOVE_TARGET_REFRESH_INTERVAL {
            self.refresh_elapsed =
                (self.refresh_elapsed - MOVE_TARGET_REFRESH_INTERVAL).max(0.0);
            return true;
        }

        false
    }
}

/// Scroll changes the follow camera's distance and vertical viewing angle.
pub(super) fn adjust_camera(
    mut player: Query<&mut Player>,
    mut camera: Query<&mut GameplayCamera>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    scroll_conversion: Res<MouseScrollPixelsPerLine>,
) {
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

/// A quick click sets one destination. Holding the button for a short
/// delay switches to periodic retargeting from the current cursor position.
pub(super) fn set_move_target_from_cursor(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &Transform), With<GameplayCamera>>,
    player: Query<&Transform, With<Player>>,
    mut target: ResMut<MovementTarget>,
    time: Res<Time>,
    ui_state: Res<UiStack>,
    ui_surfaces: Query<&RelativeCursorPosition>,
) {
    let just_pressed = mouse.just_pressed(MouseButton::Left);
    let pressed = mouse.pressed(MouseButton::Left);
    let pointer_over_ui = ui_surfaces.iter().any(|cursor| cursor.cursor_over);
    let blocked_by_ui = pointer_over_ui
        || ui_state.paused()
        || ui_state.inventory_open()
        || ui_state.game_menu_open();
    if !target.should_update_from_cursor(
        pressed,
        just_pressed,
        time.delta_secs(),
        blocked_by_ui,
    ) {
        return;
    }

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let Ok(player_transform) = player.single() else {
        return;
    };

    let (camera, camera_transform) = *camera;
    let camera_global = GlobalTransform::from(*camera_transform);
    let Ok(ray) = camera.viewport_to_world(&camera_global, cursor_position) else {
        return;
    };

    let ground_origin = Vec3::new(0.0, player_transform.translation.y, 0.0);
    let Some(destination) =
        ray.plane_intersection_point(ground_origin, InfinitePlane3d::new(Vec3::Y))
    else {
        return;
    };

    if destination.is_finite() {
        target.destination = Some(destination.with_y(player_transform.translation.y));
    }
}

/// Moves continuously toward the last clicked position. A new click replaces
/// the current destination; shift temporarily increases movement speed.
pub(super) fn move_player_to_target(
    mut player: Query<&mut Transform, With<Player>>,
    mut target: ResMut<MovementTarget>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Some(destination) = target.destination else {
        return;
    };
    let Ok(mut transform) = player.single_mut() else {
        target.destination = None;
        return;
    };

    let mut offset = destination - transform.translation;
    offset.y = 0.0;
    let distance = offset.length();
    let sprinting = input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight);
    let speed = if sprinting {
        PLAYER_SPRINT_SPEED
    } else {
        PLAYER_WALK_SPEED
    };
    let step = speed * time.delta_secs();

    if distance <= step || distance <= 0.03 {
        transform.translation.x = destination.x;
        transform.translation.z = destination.z;
        target.destination = None;
        return;
    }

    transform.translation += offset / distance * step;
}

/// Draws a sequence of small 3D arrows from the player to the selected
/// destination. The path follows the character as it moves.
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
        let arrow_end_distance =
            (arrow_start_distance + TRAJECTORY_ARROW_LENGTH).min(distance);
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
    fn click_sets_a_destination_immediately() {
        let mut target = MovementTarget::default();

        assert!(target.should_update_from_cursor(true, true, 0.016, false));
        assert!(!target.should_update_from_cursor(true, false, 0.1, false));
    }

    #[test]
    fn held_click_waits_then_refreshes_periodically() {
        let mut target = MovementTarget::default();

        assert!(target.should_update_from_cursor(true, true, 0.016, false));
        assert!(!target.should_update_from_cursor(true, false, 0.2, false));
        assert!(target.should_update_from_cursor(true, false, 0.1, false));
        assert!(!target.should_update_from_cursor(true, false, 0.04, false));
        assert!(target.should_update_from_cursor(true, false, 0.06, false));
    }

    #[test]
    fn releasing_stops_periodic_retargeting() {
        let mut target = MovementTarget::default();

        assert!(target.should_update_from_cursor(true, true, 0.016, false));
        assert!(!target.should_update_from_cursor(false, false, 0.016, false));
        assert!(!target.should_update_from_cursor(true, false, 0.2, false));
    }

    #[test]
    fn clearing_destination_also_resets_hold_state() {
        let mut target = MovementTarget::default();
        assert!(target.should_update_from_cursor(true, true, 0.016, false));
        target.clear();

        assert!(target.destination.is_none());
        assert!(!target.should_update_from_cursor(true, false, 0.2, false));
    }

    #[test]
    fn clicking_ui_does_not_start_movement_even_after_dragging_into_world() {
        let mut target = MovementTarget::default();

        assert!(!target.should_update_from_cursor(true, true, 0.016, true));
        assert!(!target.should_update_from_cursor(true, false, 0.5, false));
        assert!(!target.should_update_from_cursor(false, false, 0.016, false));
        assert!(target.should_update_from_cursor(true, true, 0.016, false));
    }

    #[test]
    fn moving_pointer_over_ui_stops_held_retargeting_until_release() {
        let mut target = MovementTarget::default();

        assert!(target.should_update_from_cursor(true, true, 0.016, false));
        assert!(!target.should_update_from_cursor(true, false, 0.1, true));
        assert!(!target.should_update_from_cursor(true, false, 0.5, false));
        assert!(!target.should_update_from_cursor(false, false, 0.016, false));
        assert!(target.should_update_from_cursor(true, true, 0.016, false));
    }

    #[test]
    fn camera_relative_ground_target_stays_on_the_ground_plane() {
        let ground = Vec3::new(0.0, 2.5, 0.0);
        let destination = Vec3::new(4.0, 9.0, -7.0).with_y(ground.y);
        assert_eq!(destination.y, 2.5);
        assert_eq!(destination.x, 4.0);
        assert_eq!(destination.z, -7.0);
    }

    #[test]
    fn trajectory_arrows_slowly_advance_toward_the_destination() {
        let initial = trajectory_arrow_phase(0.0);
        let later = trajectory_arrow_phase(1.0);
        assert!(later > initial);
        assert!(later < TRAJECTORY_ARROW_SPACING);

        let cycle_time = TRAJECTORY_ARROW_SPACING / TRAJECTORY_ARROW_SPEED;
        let cycle_phase = trajectory_arrow_phase(cycle_time);
        assert!(cycle_phase < 1e-5 || TRAJECTORY_ARROW_SPACING - cycle_phase < 1e-5);
    }

    #[test]
    fn trajectory_arrows_are_spaced_with_visible_gaps() {
        assert!(TRAJECTORY_ARROW_LENGTH < TRAJECTORY_ARROW_SPACING);
        assert!(DESTINATION_RING_RADIUS > TRAJECTORY_ARROW_LENGTH / 2.0);
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
