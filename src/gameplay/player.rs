use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
};

use crate::ui::stack::UiStack;

pub const CAMERA_DISTANCE: f32 = 18.0;
const CAMERA_TARGET_HEIGHT: f32 = 0.8;
const PLAYER_VISUAL_HEIGHT: f32 = 0.8;

#[derive(Component)]
pub struct Player {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Component)]
pub(super) struct GameplayCamera;

#[derive(Component)]
pub(super) struct PlayerVisual;

pub fn player_look(
    mut query: Query<(&mut Player, &mut Transform)>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    ui: Res<UiStack>,
) {
    if !ui.cursor_grabbed() {
        return;
    }

    for (mut player, mut transform) in &mut query {
        player.yaw -= mouse_motion.delta.x * 0.003;
        player.pitch -= mouse_motion.delta.y * 0.003;
        // Keep the camera above the player while still allowing the view to
        // move between a shallow follow angle and a near top-down angle.
        player.pitch = player.pitch.clamp(-1.35, -0.45);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);
    }
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

/// Keeps the independent gameplay camera above and behind the player.
/// Camera pitch and heading come from the player's look state.
pub(super) fn follow_camera(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<
        &mut Transform,
        (With<GameplayCamera>, Without<Player>),
    >,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };

    let target = player_transform.translation + Vec3::Y * CAMERA_TARGET_HEIGHT;
    let forward = *player_transform.forward();
    camera_transform.translation = target - forward * CAMERA_DISTANCE;
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
