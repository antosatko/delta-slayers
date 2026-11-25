use bevy::prelude::*;

mod camera;
mod collision;
mod destination;
mod movement;
mod visual;

pub use camera::{
    CAMERA_DEFAULT_PITCH, CAMERA_DISTANCE, CAMERA_MAX_PITCH,
    CAMERA_MIN_PITCH,
};
pub use collision::MovementBlocker;
pub(crate) use camera::GameplayCamera;
pub(super) use camera::{
    adjust_camera, camera_distance_after_scroll, camera_pan_offset_after_drag,
    camera_pitch_after_scroll, follow_camera, follow_camera_rotation,
    migrate_saved_camera_pitch, CAMERA_MAX_DISTANCE, CAMERA_MIN_DISTANCE,
};
pub(super) use collision::{
    move_with_collisions, movement_collision_contact, PLAYER_COLLIDER_RADIUS,
};
pub(super) use destination::{
    draw_move_trajectory, trajectory_arrow_phase, DESTINATION_RING_RADIUS,
    TRAJECTORY_ARROW_LENGTH, TRAJECTORY_ARROW_SPACING, TRAJECTORY_ARROW_SPEED,
};
pub(super) use movement::{
    keyboard_movement_direction, move_player_to_target, set_move_target_from_cursor,
    MovementTarget,
};
pub(super) use visual::{sync_player_visual, PlayerVisual, PLAYER_VISUAL_HEIGHT};

pub const INTERACTION_HEIGHT: f32 = 1.7;

#[derive(Resource, Clone, Copy, Debug)]
pub struct PlayerSettings {
    pub camera_pan_sensitivity: f32,
    pub camera_zoom_sensitivity: f32,
    pub camera_pitch_sensitivity: f32,
    pub camera_min_distance: f32,
    pub camera_max_distance: f32,
    pub walk_speed: f32,
    pub sprint_speed: f32,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            camera_pan_sensitivity: 0.0025,
            camera_zoom_sensitivity: 0.65,
            camera_pitch_sensitivity: 0.02,
            camera_min_distance: CAMERA_MIN_DISTANCE,
            camera_max_distance: CAMERA_MAX_DISTANCE,
            walk_speed: 5.0,
            sprint_speed: 8.0,
        }
    }
}

#[derive(Component)]
pub struct Player {
    pub yaw: f32,
    /// Vertical angle of the follow camera. Negative values look down.
    pub pitch: f32,
}

#[cfg(test)]
mod tests;
