use bevy::{camera::primitives::Aabb, prelude::*};
use std::collections::HashSet;

use crate::gameplay::player::{GameplayCamera, PLAYER_VISUAL_HEIGHT, Player};

const DEFAULT_OCCLUDED_OPACITY: f32 = 0.35;
const OCCLUSION_FADE_SPEED: f32 = 9.0;
const PLAYER_SEGMENT_END: f32 = 0.98;

/// Marks a Blender-authored object whose mesh becomes translucent while it
/// intersects the camera-to-player line of sight.
#[derive(Component, Reflect, Clone, Copy, Debug)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::occlusion"]
pub struct PlayerOccluder {
    /// Material opacity while this object blocks the player (0 = invisible, 1 = opaque).
    pub occluded_opacity: f32,
}

impl Default for PlayerOccluder {
    fn default() -> Self {
        Self {
            occluded_opacity: DEFAULT_OCCLUDED_OPACITY,
        }
    }
}

/// Per-renderable material data. A private material copy prevents fading one
/// tagged object from changing other meshes that share its imported material.
#[derive(Component)]
pub struct PlayerOccluderMaterialState {
    original_base_color: Color,
    original_alpha_mode: AlphaMode,
    opacity: f32,
}

/// Fades tagged objects when any mesh in their hierarchy intersects the
/// segment from the gameplay camera to the player's visual center.
pub(super) fn fade_occluding_objects(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    camera: Query<&Transform, With<GameplayCamera>>,
    occluders: Query<(Entity, &PlayerOccluder)>,
    children: Query<&Children>,
    mesh_bounds: Query<(&GlobalTransform, &Aabb), With<Mesh3d>>,
    mut renderables: Query<
        (
            Entity,
            &MeshMaterial3d<StandardMaterial>,
            Option<&mut PlayerOccluderMaterialState>,
        ),
        With<Mesh3d>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let (Ok(player), Ok(camera)) = (player.single(), camera.single()) else {
        return;
    };

    let camera_position = camera.translation;
    let player_position = player.translation + Vec3::Y * PLAYER_VISUAL_HEIGHT;
    if !camera_position.is_finite() || !player_position.is_finite() {
        return;
    }

    let delta_secs = time.delta_secs();
    let blend = if delta_secs.is_finite() && delta_secs > 0.0 {
        1.0 - (-OCCLUSION_FADE_SPEED * delta_secs).exp()
    } else {
        0.0
    };

    // Avoid processing a descendant twice when an authored tag is nested
    // inside another tagged object's hierarchy.
    let mut processed = HashSet::new();

    for (root, occluder) in &occluders {
        let mut pending = vec![root];
        let mut object_entities = Vec::new();

        while let Some(entity) = pending.pop() {
            if !processed.insert(entity) {
                continue;
            }
            object_entities.push(entity);
            if let Ok(nested) = children.get(entity) {
                pending.extend(nested.iter());
            }
        }

        let is_occluding = object_entities.iter().any(|entity| {
            let Ok((transform, aabb)) = mesh_bounds.get(*entity) else {
                return false;
            };
            let (min, max) = transformed_aabb_bounds(aabb, transform);
            segment_intersects_aabb(camera_position, player_position, min, max)
        });

        let target_opacity = if is_occluding {
            if occluder.occluded_opacity.is_finite() {
                occluder.occluded_opacity.clamp(0.0, 1.0)
            } else {
                DEFAULT_OCCLUDED_OPACITY
            }
        } else {
            1.0
        };

        for entity in object_entities {
            let Ok((_, material_handle, state)) = renderables.get_mut(entity) else {
                continue;
            };

            if let Some(mut state) = state {
                state.opacity += (target_opacity - state.opacity) * blend;
                if (target_opacity - state.opacity).abs() < 0.001 {
                    state.opacity = target_opacity;
                }

                let Some(mut material) = materials.get_mut(&material_handle.0) else {
                    continue;
                };
                material.base_color = state
                    .original_base_color
                    .with_alpha(state.original_base_color.alpha() * state.opacity);
                material.alpha_mode = if state.opacity < 0.999 {
                    AlphaMode::Blend
                } else {
                    state.original_alpha_mode.clone()
                };
            } else if is_occluding {
                let Some(original) = materials.get(&material_handle.0).cloned() else {
                    continue;
                };

                let runtime_material = materials.add(original.clone());
                commands.entity(entity).insert((
                    MeshMaterial3d(runtime_material),
                    PlayerOccluderMaterialState {
                        original_base_color: original.base_color,
                        original_alpha_mode: original.alpha_mode,
                        opacity: 1.0,
                    },
                ));
            }
        }
    }
}

/// Converts a mesh's local AABB to a world-space axis-aligned box, accounting
/// for the full transform hierarchy, including rotation and non-uniform scale.
fn transformed_aabb_bounds(aabb: &Aabb, transform: &GlobalTransform) -> (Vec3, Vec3) {
    let center = Vec3::new(aabb.center.x, aabb.center.y, aabb.center.z);
    let half_extents = Vec3::new(
        aabb.half_extents.x,
        aabb.half_extents.y,
        aabb.half_extents.z,
    );

    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    let affine = transform.affine();

    for x in [-1.0, 1.0] {
        for y in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                let local = center + half_extents * Vec3::new(x, y, z);
                let world = affine.transform_point3(local);
                min = min.min(world);
                max = max.max(world);
            }
        }
    }

    (min, max)
}

/// Slab intersection for a finite line segment. The final 2% is excluded so
/// the player's own visual bounds cannot count as an obstruction in front of it.
fn segment_intersects_aabb(start: Vec3, end: Vec3, min: Vec3, max: Vec3) -> bool {
    let direction = end - start;
    let mut enter = 0.0_f32;
    let mut exit = PLAYER_SEGMENT_END;

    for axis in 0..3 {
        let origin = start[axis];
        let delta = direction[axis];

        if delta.abs() <= f32::EPSILON {
            if origin < min[axis] || origin > max[axis] {
                return false;
            }
            continue;
        }

        let inverse = delta.recip();
        let mut near = (min[axis] - origin) * inverse;
        let mut far = (max[axis] - origin) * inverse;
        if near > far {
            std::mem::swap(&mut near, &mut far);
        }

        enter = enter.max(near);
        exit = exit.min(far);
        if enter > exit {
            return false;
        }
    }

    exit >= 0.0 && enter <= PLAYER_SEGMENT_END
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_detects_objects_between_camera_and_player() {
        let start = Vec3::ZERO;
        let end = Vec3::Z * 10.0;

        assert!(segment_intersects_aabb(
            start,
            end,
            Vec3::new(-1.0, -1.0, 4.0),
            Vec3::new(1.0, 1.0, 6.0),
        ));
    }

    #[test]
    fn segment_ignores_objects_behind_the_player_or_outside_the_view_line() {
        let start = Vec3::ZERO;
        let end = Vec3::Z * 10.0;

        assert!(!segment_intersects_aabb(
            start,
            end,
            Vec3::new(-1.0, -1.0, 11.0),
            Vec3::new(1.0, 1.0, 12.0),
        ));
        assert!(!segment_intersects_aabb(
            start,
            end,
            Vec3::new(3.0, -1.0, 4.0),
            Vec3::new(4.0, 1.0, 6.0),
        ));
    }

    #[test]
    fn segment_intersection_respects_rotated_world_bounds() {
        let aabb = Aabb {
            center: Vec3::ZERO.into(),
            half_extents: Vec3::new(0.5, 1.0, 0.5).into(),
        };
        let transform = GlobalTransform::from(
            Transform::from_xyz(0.0, 0.0, 5.0)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4)),
        );
        let (min, max) = transformed_aabb_bounds(&aabb, &transform);

        assert!(segment_intersects_aabb(
            Vec3::ZERO,
            Vec3::Z * 10.0,
            min,
            max,
        ));
    }
}
