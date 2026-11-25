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

fn move_against_test_blockers(start: Vec3, delta: Vec3, blockers: &[GlobalTransform]) -> Vec3 {
    move_with_collisions(start, delta, |candidate, intended_movement| {
        blockers.iter().find_map(|blocker| {
            let contact = movement_collision_contact(candidate, blocker)?;
            (intended_movement.dot(contact.normal) < -1e-5).then_some(contact)
        })
    })
}

#[test]
fn movement_stops_at_an_authored_wall() {
    let blocker = GlobalTransform::from(Transform::from_scale(Vec3::new(0.1, 1.0, 2.0)));

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
    let blocker = GlobalTransform::from(Transform::from_scale(Vec3::new(0.1, 1.0, 4.0)));

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
        Transform::from_xyz(0.0, 3.0, 0.0).with_scale(Vec3::new(1.0, 0.25, 1.0)),
    );

    assert!(movement_collision_contact(Vec3::ZERO, &blocker).is_none());
}

#[test]
fn diagonal_movement_cannot_cut_through_a_wall_corner() {
    let blocker = GlobalTransform::from(Transform::from_scale(Vec3::new(0.1, 1.0, 0.1)));

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
fn middle_drag_keeps_camera_pan_on_the_horizontal_plane() {
    let offset = camera_pan_offset_after_drag(
        Vec3::new(0.0, 4.0, 0.0),
        Vec2::new(100.0, 50.0),
        Vec3::new(1.0, 0.4, 0.0),
        Vec3::new(0.0, 0.8, -0.6),
        0.05,
    );

    assert!(offset.x < 0.0);
    assert!(offset.z < 0.0);
    assert_eq!(offset.y, 0.0);
}

#[test]
fn middle_drag_pan_accumulates_from_existing_camera_position() {
    let first = camera_pan_offset_after_drag(
        Vec3::new(2.0, 0.0, 0.0),
        Vec2::new(20.0, 0.0),
        Vec3::X,
        Vec3::Y,
        0.05,
    );
    let second = camera_pan_offset_after_drag(first, Vec2::new(20.0, 0.0), Vec3::X, Vec3::Y, 0.05);

    assert!(second.abs_diff_eq(Vec3::ZERO, 1e-6));
}

#[test]
fn scroll_tilt_uses_the_default_configurable_increment_and_previous_range() {
    let sensitivity = PlayerSettings::default().camera_pitch_sensitivity;
    let tilted = camera_pitch_after_scroll(CAMERA_DEFAULT_PITCH, 1.0, sensitivity);

    assert!((tilted - (CAMERA_DEFAULT_PITCH + 0.02)).abs() < 1e-6);
    assert!((CAMERA_MAX_PITCH - CAMERA_MIN_PITCH - 0.30).abs() < 1e-6);
}

#[test]
fn scrolling_in_zooms_in_and_flattens_the_view() {
    assert!(
        camera_distance_after_scroll(
            CAMERA_DISTANCE,
            1.0,
            0.65,
            CAMERA_MIN_DISTANCE,
            CAMERA_MAX_DISTANCE
        ) < CAMERA_DISTANCE
    );
    assert!(
        camera_pitch_after_scroll(CAMERA_DEFAULT_PITCH, 1.0, 0.02) > CAMERA_DEFAULT_PITCH
    );
}

#[test]
fn scrolling_out_zooms_out_and_raises_the_view() {
    assert!(
        camera_distance_after_scroll(
            CAMERA_DISTANCE,
            -1.0,
            0.65,
            CAMERA_MIN_DISTANCE,
            CAMERA_MAX_DISTANCE
        ) > CAMERA_DISTANCE
    );
    assert!(
        camera_pitch_after_scroll(CAMERA_DEFAULT_PITCH, -1.0, 0.02) < CAMERA_DEFAULT_PITCH
    );
}

#[test]
fn camera_adjustments_stay_within_their_limits() {
    assert_eq!(
        camera_distance_after_scroll(
            CAMERA_MIN_DISTANCE,
            10.0,
            0.65,
            CAMERA_MIN_DISTANCE,
            CAMERA_MAX_DISTANCE
        ),
        CAMERA_MIN_DISTANCE
    );
    assert_eq!(
        camera_distance_after_scroll(
            CAMERA_MAX_DISTANCE,
            -10.0,
            0.65,
            CAMERA_MIN_DISTANCE,
            CAMERA_MAX_DISTANCE
        ),
        CAMERA_MAX_DISTANCE
    );
    assert_eq!(
        camera_pitch_after_scroll(CAMERA_MIN_PITCH, -10.0, 0.02),
        CAMERA_MIN_PITCH
    );
    assert_eq!(
        camera_pitch_after_scroll(CAMERA_MAX_PITCH, 10.0, 0.02),
        CAMERA_MAX_PITCH
    );
}
