use bevy::prelude::*;

pub(in crate::gameplay) const PLAYER_COLLIDER_RADIUS: f32 = 0.32;
pub(in crate::gameplay) const PLAYER_COLLIDER_HEIGHT: f32 = 1.6;
pub(in crate::gameplay) const MAX_COLLISION_STEP: f32 = 0.1;
pub(in crate::gameplay) const MAX_COLLISION_RESOLVE_PASSES: usize = 6;

/// An invisible, Blender-authored box that blocks player movement.
///
/// Attach this to an Empty and use its world transform to size and orient the
/// box. The collision volume uses local bounds of -1 to 1 on every axis.
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::player"]
pub struct MovementBlocker;

/// Advances in small increments so a fast frame or sprint cannot tunnel
/// through a thin authored wall. Contact correction preserves tangential
/// movement so the player can slide along the blocker.
pub(in crate::gameplay) fn move_with_collisions(
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
pub(in crate::gameplay) struct MovementContact {
    pub(in crate::gameplay) normal: Vec3,
    pub(in crate::gameplay) penetration: f32,
}

/// Returns the outward horizontal contact normal and penetration depth when
/// the player's circular footprint overlaps an upright, oriented box.
/// Blockers are rotated around Y; their vertical scale and position determine
/// whether they overlap the body.
pub(in crate::gameplay) fn movement_collision_contact(
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
