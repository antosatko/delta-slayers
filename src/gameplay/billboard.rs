use bevy::prelude::*;

const BILLBOARD_VIEW_TILT: f32 = 0.35;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::billboard"]
pub struct Billboard;

/// Faces billboards toward the camera horizontally and partially follows its
/// vertical viewing angle. Keeping some upright bias avoids fully screen-flat
/// sprites while still making them readable from the angled follow camera.
pub fn billboard_system(
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    mut billboards: Query<&mut Transform, With<Billboard>>,
) {
    let Ok(camera) = camera_query.single() else {
        return;
    };

    let camera_position = camera.translation();
    for mut transform in &mut billboards {
        let to_camera = camera_position - transform.translation;
        let Some(rotation) = billboard_facing_rotation(to_camera) else {
            continue;
        };
        transform.rotation = rotation;
    }
}

fn billboard_facing_rotation(to_camera: Vec3) -> Option<Quat> {
    let horizontal = Vec3::new(to_camera.x, 0.0, to_camera.z);
    let horizontal_length = horizontal.length();
    if horizontal_length < 1e-6 {
        return None;
    }

    let yaw = horizontal.x.atan2(horizontal.z);
    let view_pitch = to_camera.y.atan2(horizontal_length);
    let upright = Quat::from_rotation_y(yaw);
    let view_aligned = upright * Quat::from_rotation_x(-view_pitch);

    Some(upright.slerp(view_aligned, BILLBOARD_VIEW_TILT))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn billboard_partially_tracks_camera_elevation() {
        let to_camera = Vec3::new(0.0, 1.0, 1.0).normalize();
        let facing = billboard_facing_rotation(to_camera)
            .expect("camera direction has a horizontal component");
        let normal = facing * Vec3::Z;

        assert!(normal.y > 0.0);
        assert!(normal.angle_between(to_camera) > 0.0);
        assert!(normal.angle_between(to_camera) < to_camera.angle_between(Vec3::Z));
    }

    #[test]
    fn billboard_remains_upright_when_camera_is_level() {
        let facing = billboard_facing_rotation(Vec3::Z)
            .expect("camera direction has a horizontal component");

        assert!(facing.abs_diff_eq(Quat::IDENTITY, 1e-6));
    }

    #[test]
    fn billboard_handles_camera_directly_above_without_nan() {
        assert!(billboard_facing_rotation(Vec3::Y).is_none());
    }
}
