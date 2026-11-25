use bevy::{
    input::mouse::{AccumulatedMouseScroll, MouseScrollPixelsPerLine},
    prelude::*,
    ui::RelativeCursorPosition,
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
pub(super) const PLAYER_VISUAL_HEIGHT: f32 = 0.8;
pub(super) const PLAYER_COLLIDER_RADIUS: f32 = 0.32;
pub(super) const PLAYER_COLLIDER_HEIGHT: f32 = 1.6;
pub(super) const MAX_COLLISION_STEP: f32 = 0.1;
pub(super) const MAX_COLLISION_RESOLVE_PASSES: usize = 6;
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

/// An invisible, Blender-authored box that blocks player movement.
///
/// Attach this to an Empty and use its world transform to size and orient the
/// box. The collision volume uses local bounds of -1 to 1 on every axis.
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::player"]
pub struct MovementBlocker;

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
            self.refresh_elapsed = (self.refresh_elapsed - MOVE_TARGET_REFRESH_INTERVAL).max(0.0);
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
    ui_surfaces: Query<(&RelativeCursorPosition, &InheritedVisibility)>,
) {
    let just_pressed = mouse.just_pressed(MouseButton::Left);
    let pressed = mouse.pressed(MouseButton::Left);
    let pointer_over_ui = ui_surfaces
        .iter()
        .any(|(cursor, visibility)| visibility.get() && cursor.cursor_over);
    let blocked_by_ui = pointer_over_ui
        || ui_state.paused()
        || ui_state.inventory_open()
        || ui_state.game_menu_open();
    if !target.should_update_from_cursor(pressed, just_pressed, time.delta_secs(), blocked_by_ui) {
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

/// Moves with WASD/arrows when pressed, otherwise follows the last clicked
/// destination. Keyboard movement takes priority and cancels the current path.
pub(super) fn move_player_to_target(
    mut player: Query<&mut Transform, With<Player>>,
    camera: Query<&Transform, (With<GameplayCamera>, Without<Player>)>,
    blockers: Query<&GlobalTransform, With<MovementBlocker>>,
    mut target: ResMut<MovementTarget>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = player.single_mut() else {
        target.clear();
        return;
    };

    let sprinting = input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight);
    let speed = if sprinting {
        PLAYER_SPRINT_SPEED
    } else {
        PLAYER_WALK_SPEED
    };
    let delta_secs = time.delta_secs();
    if !delta_secs.is_finite() || delta_secs <= 0.0 {
        return;
    }
    let step = speed * delta_secs;

    let movement = if let Some(camera_transform) = camera.iter().next() {
        let camera_forward = camera_transform.forward().as_vec3();
        let camera_forward = Vec3::new(camera_forward.x, 0.0, camera_forward.z);
        keyboard_movement_direction(&input, camera_forward).map(|direction| {
            target.clear();
            direction * step
        })
    } else {
        None
    };

    let movement = movement.or_else(|| {
        let destination = target.destination?;
        let mut offset = destination - transform.translation;
        offset.y = 0.0;
        let distance = offset.length();

        if distance <= 0.03 {
            target.destination = None;
            return None;
        }

        Some(offset / distance * distance.min(step))
    });

    let Some(movement) = movement else {
        return;
    };

    let next_position = move_with_collisions(
        transform.translation,
        movement,
        |candidate, intended_movement| {
            blockers.iter().find_map(|blocker| {
                let contact = movement_collision_contact(candidate, blocker)?;
                (intended_movement.dot(contact.normal) < -1e-5).then_some(contact)
            })
        },
    );

    transform.translation.x = next_position.x;
    transform.translation.z = next_position.z;

    if let Some(destination) = target.destination {
        let remaining = Vec2::new(
            destination.x - transform.translation.x,
            destination.z - transform.translation.z,
        );
        if remaining.length() <= 0.03 {
            target.destination = None;
        }
    }
}

/// Advances in small increments so a fast frame or sprint cannot tunnel
/// through a thin authored wall. Overlapping steps are corrected to the wall
/// surface, preserving tangential movement so the player can slide along it.
pub(super) fn move_with_collisions(
    start: Vec3,
    delta: Vec3,
    mut collision_contact: impl FnMut(Vec3, Vec3) -> Option<MovementContact>,
) -> Vec3 {
    let distance = Vec2::new(delta.x, delta.z).length();
    if !distance.is_finite() || distance <= f32::EPSILON || !delta.is_finite() {
        return start;
    }

    let step_count = (distance / MAX_COLLISION_STEP).ceil().max(1.0) as usize;
    let step = delta / step_count as f32;
    let mut position = start;

    for _ in 0..step_count {
        let mut movement = step;
        for _ in 0..MAX_COLLISION_RESOLVE_PASSES {
            if movement.length_squared() <= f32::EPSILON {
                break;
            }

            let candidate = position + movement;
            let Some(contact) = collision_contact(candidate, movement) else {
                break;
            };

            let corrected_candidate = candidate + contact.normal * contact.penetration;
            movement = corrected_candidate - position;
        }
        position += movement;
    }

    position
}

#[derive(Clone, Copy)]
pub(super) struct MovementContact {
    pub(super) normal: Vec3,
    pub(super) penetration: f32,
}

/// Returns the outward horizontal contact normal and penetration depth when
/// the player's circular footprint overlaps an upright, oriented box.
/// Blockers are rotated around Y; their vertical scale and position determine
/// whether they overlap the body.
pub(super) fn movement_collision_contact(
    player_position: Vec3,
    blocker: &GlobalTransform,
) -> Option<MovementContact> {
    let (scale, rotation, translation) = blocker.to_scale_rotation_translation();
    let half_extents = scale.abs();
    if !half_extents.is_finite()
        || half_extents.x <= f32::EPSILON
        || half_extents.y <= f32::EPSILON
        || half_extents.z <= f32::EPSILON
    {
        return None;
    }

    let player_center = player_position + Vec3::Y * (PLAYER_COLLIDER_HEIGHT * 0.5);
    let local_center = rotation.inverse() * (player_center - translation);

    let player_bottom = local_center.y - PLAYER_COLLIDER_HEIGHT * 0.5;
    let player_top = local_center.y + PLAYER_COLLIDER_HEIGHT * 0.5;
    if player_top <= -half_extents.y || player_bottom >= half_extents.y {
        return None;
    }

    let local_xz = Vec2::new(local_center.x, local_center.z);
    let closest = local_xz.clamp(
        Vec2::new(-half_extents.x, -half_extents.z),
        Vec2::new(half_extents.x, half_extents.z),
    );
    let difference = local_xz - closest;
    let distance_squared = difference.length_squared();

    let (local_normal, penetration) = if distance_squared > 1e-8 {
        let distance = distance_squared.sqrt();
        if distance >= PLAYER_COLLIDER_RADIUS {
            return None;
        }
        (
            Vec3::new(difference.x, 0.0, difference.y) / distance,
            PLAYER_COLLIDER_RADIUS - distance,
        )
    } else {
        // The center is inside the box footprint. Pick its nearest face so
        // motion toward the interior is blocked while motion out is allowed.
        let distance_to_x_face = half_extents.x - local_center.x.abs();
        let distance_to_z_face = half_extents.z - local_center.z.abs();
        if distance_to_x_face <= distance_to_z_face {
            (
                Vec3::new(if local_center.x >= 0.0 { 1.0 } else { -1.0 }, 0.0, 0.0),
                PLAYER_COLLIDER_RADIUS + distance_to_x_face,
            )
        } else {
            (
                Vec3::new(0.0, 0.0, if local_center.z >= 0.0 { 1.0 } else { -1.0 }),
                PLAYER_COLLIDER_RADIUS + distance_to_z_face,
            )
        }
    };

    let world_normal = rotation * local_normal;
    let horizontal_normal = Vec3::new(world_normal.x, 0.0, world_normal.z).normalize_or_zero();
    (horizontal_normal != Vec3::ZERO).then_some(MovementContact {
        normal: horizontal_normal,
        penetration,
    })
}

fn keyboard_movement_direction(input: &ButtonInput<KeyCode>, camera_forward: Vec3) -> Option<Vec3> {
    let left = input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft);
    let right = input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight);
    let forward = input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp);
    let backward = input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown);

    let local_direction = Vec3::new(
        (right as i8 - left as i8) as f32,
        0.0,
        (backward as i8 - forward as i8) as f32,
    );
    if local_direction == Vec3::ZERO {
        return None;
    }

    let forward = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize_or_zero();
    if forward == Vec3::ZERO {
        return None;
    }
    let right = forward.cross(Vec3::Y);

    Some((right * local_direction.x - forward * local_direction.z).normalize())
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
    mut camera: Query<(&mut Transform, &GameplayCamera), (With<GameplayCamera>, Without<Player>)>,
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
    mut visual: Query<&mut Transform, (With<PlayerVisual>, Without<Player>)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut visual_transform) = visual.single_mut() else {
        return;
    };

    visual_transform.translation = player_transform.translation + Vec3::Y * PLAYER_VISUAL_HEIGHT;
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
    fn wasd_and_arrow_keys_move_in_the_same_direction() {
        let mut wasd = ButtonInput::<KeyCode>::default();
        wasd.press(KeyCode::KeyW);
        let mut arrows = ButtonInput::<KeyCode>::default();
        arrows.press(KeyCode::ArrowUp);

        let camera_forward = Vec3::NEG_Z;
        let wasd_direction = keyboard_movement_direction(&wasd, camera_forward).unwrap();
        let arrow_direction = keyboard_movement_direction(&arrows, camera_forward).unwrap();

        assert!(wasd_direction.abs_diff_eq(camera_forward, 1e-6));
        assert!(wasd_direction.abs_diff_eq(arrow_direction, 1e-6));
    }

    #[test]
    fn keyboard_diagonal_movement_is_normalized() {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyW);
        input.press(KeyCode::KeyD);

        let direction = keyboard_movement_direction(&input, Vec3::NEG_Z).unwrap();

        assert!((direction.length() - 1.0).abs() < 1e-6);
        assert!(direction.x > 0.0);
        assert!(direction.z < 0.0);
    }

    #[test]
    fn keyboard_movement_uses_camera_heading() {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyW);

        let camera_forward = Vec3::X;
        let direction = keyboard_movement_direction(&input, camera_forward).unwrap();

        assert!(direction.abs_diff_eq(Vec3::X, 1e-6));
    }

    #[test]
    fn camera_relative_left_and_right_follow_camera_basis() {
        let mut left = ButtonInput::<KeyCode>::default();
        left.press(KeyCode::KeyA);
        let mut right = ButtonInput::<KeyCode>::default();
        right.press(KeyCode::KeyD);

        let camera_forward = Vec3::X;
        let left_direction = keyboard_movement_direction(&left, camera_forward).unwrap();
        let right_direction = keyboard_movement_direction(&right, camera_forward).unwrap();

        assert!(left_direction.abs_diff_eq(Vec3::NEG_Z, 1e-6));
        assert!(right_direction.abs_diff_eq(Vec3::Z, 1e-6));
    }

    #[test]
    fn opposing_keyboard_directions_cancel_out() {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyW);
        input.press(KeyCode::ArrowDown);

        assert!(keyboard_movement_direction(&input, Vec3::NEG_Z).is_none());
    }

    fn move_against_test_blockers(
        start: Vec3,
        delta: Vec3,
        blockers: &[GlobalTransform],
    ) -> Vec3 {
        move_with_collisions(start, delta, |candidate, intended_movement| {
            blockers.iter().find_map(|blocker| {
                let contact = movement_collision_contact(candidate, blocker)?;
                (intended_movement.dot(contact.normal) < -1e-5).then_some(contact)
            })
        })
    }

    #[test]
    fn movement_stops_at_an_authored_wall() {
        let blocker = GlobalTransform::from(
            Transform::from_scale(Vec3::new(0.1, 1.0, 2.0)),
        );

        let result = move_against_test_blockers(
            Vec3::new(-2.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
            &[blocker],
        );

        let wall_edge = -0.1;
        assert!(result.x >= wall_edge - PLAYER_COLLIDER_RADIUS - 0.02);
        assert!(result.x <= wall_edge - PLAYER_COLLIDER_RADIUS + 0.02);
    }

    #[test]
    fn movement_slides_along_a_wall_instead_of_stopping_completely() {
        let blocker = GlobalTransform::from(
            Transform::from_scale(Vec3::new(0.1, 1.0, 4.0)),
        );

        let result = move_against_test_blockers(
            Vec3::new(-2.0, 0.0, -2.0),
            Vec3::new(3.0, 0.0, 4.0),
            &[blocker],
        );

        assert!((result.x - (-0.1 - PLAYER_COLLIDER_RADIUS)).abs() < 0.03);
        assert!(result.z > 1.8);
    }

    #[test]
    fn collision_uses_the_blockers_world_rotation() {
        let blocker = GlobalTransform::from(
            Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::new(0.1, 1.0, 2.0)),
        );

        let contact = movement_collision_contact(Vec3::new(0.0, 0.0, 0.3), &blocker)
            .expect("player footprint overlaps the rotated wall");

        assert!(contact.normal.z > 0.99);
        assert!(contact.normal.x.abs() < 0.01);
    }

    #[test]
    fn collision_does_not_apply_when_the_wall_is_above_the_player() {
        let blocker = GlobalTransform::from(
            Transform::from_xyz(0.0, 3.0, 0.0)
                .with_scale(Vec3::new(1.0, 0.25, 1.0)),
        );

        assert!(movement_collision_contact(Vec3::ZERO, &blocker).is_none());
    }

    #[test]
    fn diagonal_movement_cannot_cut_through_a_wall_corner() {
        let blocker = GlobalTransform::from(
            Transform::from_scale(Vec3::new(0.1, 1.0, 0.1)),
        );

        let result = move_against_test_blockers(
            Vec3::new(-2.0, 0.0, -2.0),
            Vec3::new(2.0, 0.0, 2.0),
            &[blocker],
        );

        let corner_distance = Vec2::new(result.x + 0.1, result.z + 0.1).length();
        assert!(corner_distance >= PLAYER_COLLIDER_RADIUS - 0.02);
        assert!(result.x < 0.1 && result.z < 0.1);
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
