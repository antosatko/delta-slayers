use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
};

use crate::state::CursorGrabbed;

#[derive(Component)]
pub struct Player {
    pub yaw: f32,
    pub pitch: f32,
}

pub fn player_look(
    mut query: Query<(&mut Player, &mut Transform)>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    cursor_grabbed: Res<CursorGrabbed>,
) {
    if !cursor_grabbed.0 {
        return;
    }
    for (mut player, mut transform) in &mut query {
        player.yaw -= mouse_motion.delta.x * 0.003;
        player.pitch -= mouse_motion.delta.y * 0.003;
        player.pitch = player.pitch.clamp(-1.4, 1.4);
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
            dir = dir.normalize();
            transform.translation += dir * speed * dt;
        }
        transform.translation.y = 1.7;
    }
}