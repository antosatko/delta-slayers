use bevy::prelude::*;

mod camera;
mod collision;
mod destination;
mod movement;
mod visual;

#[cfg(test)]
use camera::{
    camera_distance_after_scroll, camera_pan_offset_after_drag, camera_pitch_after_scroll,
};

pub(crate) use camera::GameplayCamera;
pub use camera::{CAMERA_DEFAULT_PITCH, CAMERA_DISTANCE, CAMERA_MAX_PITCH, CAMERA_MIN_PITCH};
pub(super) use camera::{
    CAMERA_MAX_DISTANCE, CAMERA_MIN_DISTANCE, adjust_camera, follow_camera, follow_camera_rotation,
    migrate_saved_camera_pitch,
};
pub use collision::MovementBlocker;
pub(super) use collision::{
    PLAYER_COLLIDER_RADIUS, move_with_collisions, movement_collision_contact,
};
pub(super) use destination::{
    TrajectoryGizmoConfigGroup, draw_move_trajectory,
};
pub(super) use movement::{
    MovementTarget, move_player_to_target, set_move_target_from_cursor,
};
pub(super) use visual::{PLAYER_VISUAL_HEIGHT, PlayerVisual, sync_player_visual};

pub const INTERACTION_HEIGHT: f32 = 1.7;

#[derive(Resource, Clone, Copy, Debug)]
pub struct PlayerSettings {
    pub camera_pan_sensitivity: f32,
    pub camera_zoom_sensitivity: f32,
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
