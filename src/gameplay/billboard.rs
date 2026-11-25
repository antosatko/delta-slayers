use bevy::prelude::*;

const BILLBOARD_VIEW_TILT: f32 = 0.35;
const SQUASH_SMOOTHING: f32 = 12.0;
const SQUASH_FULL_INTENSITY_SPEED: f32 = 3.0;
const SQUASH_MAX_ANIMATED_SPEED: f32 = 40.0;
const SQUASH_CYCLES_PER_UNIT: f32 = 0.35;
const SQUASH_HORIZONTAL_STRETCH: f32 = 0.025;
const SQUASH_VERTICAL_COMPRESSION: f32 = 0.04;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::billboard"]
pub struct Billboard;

/// Opt-in movement-driven squash animation for an entity's visual transform.
///
/// Attach this to a moving sprite or billboard. The animation changes only its
/// X/Y scale; the gameplay entity and its world position remain untouched.
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::billboard"]
pub struct SquashOnMove;

#[derive(Component)]
struct SquashState {
    previous_position: Vec3,
    base_scale: Vec3,
    phase: f32,
    intensity: f32,
}

/// Updates the subtle squash/stretch on entities tagged with `SquashOnMove`.
///
/// Large one-frame displacements are treated as teleports rather than movement,
/// preventing fast travel and spawn placement from triggering a squash pulse.
pub fn animate_moving_sprites(
    mut commands: Commands,
    time: Res<Time>,
    mut sprites: Query<(Entity, &mut Transform, Option<&mut SquashState>), With<SquashOnMove>>,
) {
    let delta_secs = time.delta_secs();
    if delta_secs <= 0.0 {
        return;
    }

    for (entity, mut transform, state) in &mut sprites {
        let Some(mut state) = state else {
            commands.entity(entity).insert(SquashState {
                previous_position: transform.translation,
                base_scale: transform.scale,
                phase: 0.0,
                intensity: 0.0,
            });
            continue;
        };

        let previous_position = state.previous_position;
        state.previous_position = transform.translation;
        let base_scale = state.base_scale;

        let distance = transform.translation.distance(previous_position);
        let speed = distance / delta_secs;
        let target_intensity = movement_intensity(speed);
        let moving = distance > 0.0 && target_intensity > 0.0;

        if moving {
            state.phase = (state.phase + distance * std::f32::consts::TAU * SQUASH_CYCLES_PER_UNIT)
                .rem_euclid(std::f32::consts::TAU);
        }

        let blend = 1.0 - (-SQUASH_SMOOTHING * delta_secs).exp();
        state.intensity += (target_intensity - state.intensity) * blend;

        transform.scale = squash_scale(base_scale, state.phase, state.intensity);
    }
}

fn movement_intensity(speed: f32) -> f32 {
    if speed.is_finite() && speed <= SQUASH_MAX_ANIMATED_SPEED {
        (speed / SQUASH_FULL_INTENSITY_SPEED).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn squash_scale(base_scale: Vec3, phase: f32, intensity: f32) -> Vec3 {
    let pulse = phase.sin() * intensity;
    Vec3::new(
        base_scale.x * (1.0 + SQUASH_HORIZONTAL_STRETCH * pulse),
        base_scale.y * (1.0 - SQUASH_VERTICAL_COMPRESSION * pulse),
        base_scale.z,
    )
}

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
    fn movement_squash_compresses_height_and_stretches_width() {
        let base_scale = Vec3::new(2.0, 3.0, 0.5);

        let scale = squash_scale(base_scale, std::f32::consts::FRAC_PI_2, 1.0);

        assert!(scale.x > base_scale.x);
        assert!(scale.y < base_scale.y);
        assert_eq!(scale.z, base_scale.z);
    }

    #[test]
    fn teleports_do_not_trigger_movement_squash() {
        assert!(movement_intensity(8.0) > 0.0);
        assert_eq!(movement_intensity(10_000.0), 0.0);
        assert_eq!(movement_intensity(f32::NAN), 0.0);
    }

    #[test]
    fn idle_movement_animation_preserves_the_authored_scale() {
        let base_scale = Vec3::new(1.5, 2.0, 0.75);

        assert_eq!(squash_scale(base_scale, 2.1, 0.0), base_scale);
    }

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
