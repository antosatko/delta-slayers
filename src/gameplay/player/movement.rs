use bevy::{
    prelude::*,
    ui::RelativeCursorPosition,
    window::PrimaryWindow,
};

use crate::ui::stack::UiStack;

use super::{GameplayCamera, MovementBlocker, Player, PlayerSettings};
use super::collision::{move_with_collisions, movement_collision_contact};

const MOVE_TARGET_HOLD_DELAY: f32 = 0.3;
const MOVE_TARGET_REFRESH_INTERVAL: f32 = 0.1;

/// The active click destination and click-and-hold retargeting state.
#[derive(Resource, Default)]
pub(in crate::gameplay) struct MovementTarget {
    pub(in crate::gameplay) destination: Option<Vec3>,
    hold_elapsed: f32,
    refresh_elapsed: f32,
    ignore_click_until_release: bool,
}

impl MovementTarget {
    pub(in crate::gameplay) fn clear(&mut self) {
        self.destination = None;
        self.hold_elapsed = 0.0;
        self.refresh_elapsed = 0.0;
        self.ignore_click_until_release = false;
    }

    pub(in crate::gameplay) fn should_update_from_cursor(
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

/// A quick click sets one destination. Holding the button for a short
/// delay switches to periodic retargeting from the current cursor position.
pub(in crate::gameplay) fn set_move_target_from_cursor(
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
pub(in crate::gameplay) fn move_player_to_target(
    mut player: Query<&mut Transform, With<Player>>,
    camera: Query<&Transform, (With<GameplayCamera>, Without<Player>)>,
    blockers: Query<&GlobalTransform, With<MovementBlocker>>,
    mut target: ResMut<MovementTarget>,
    settings: Res<PlayerSettings>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = player.single_mut() else {
        target.clear();
        return;
    };

    let sprinting = input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight);
    let speed = if sprinting { settings.sprint_speed } else { settings.walk_speed };
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

pub(in crate::gameplay) fn keyboard_movement_direction(input: &ButtonInput<KeyCode>, camera_forward: Vec3) -> Option<Vec3> {
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


