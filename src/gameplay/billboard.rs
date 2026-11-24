use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::billboard"]
pub struct Billboard;

pub fn billboard_system(
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    mut billboards: Query<&mut Transform, With<Billboard>>,
) {
    let Ok(cam) = camera_query.single() else {
        return;
    };
    let cam_pos = cam.translation();
    for mut transform in &mut billboards {
        let dir = cam_pos - transform.translation;
        // Only rotate around the vertical (Y) axis so the billboard stays
        // upright: it faces the camera horizontally but never tilts up/down.
        let horizontal = Vec3::new(dir.x, 0.0, dir.z);
        if horizontal.length_squared() < 1e-6 {
            continue;
        }
        let angle = horizontal.x.atan2(horizontal.z);
        transform.rotation = Quat::from_rotation_y(angle);
    }
}