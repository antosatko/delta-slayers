use bevy::{camera::primitives::MeshAabb, prelude::*};

use super::{
    inventory::{InventoryItem, InventoryModel},
    player::{INTERACTION_HEIGHT, Player},
};

const PICKUP_INTERACTION_RADIUS: f32 = 1.75;
const PICKUP_MAX_VERTICAL_DISTANCE: f32 = 2.25;
const DEFAULT_PICKUP_COOLDOWN: f32 = 0.5;

/// The action performed when a pickup is used. More actions can be added here.
#[derive(Reflect, Default, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(Default)]
#[type_path = "delta_slayers::gameplay::pickups"]
pub enum PickupAction {
    #[default]
    AddToInventory,
}

/// Attach to the root Blender object (or Empty) that should be interactable.
///
/// Each successful interaction grants one item. For finite pickups, `quantity`
/// is the remaining stock; an infinite pickup ignores that limit. Cooldown is
/// the minimum delay between successful interactions.
#[derive(Component, Reflect, Clone, Copy, Debug)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::pickups"]
pub struct Pickup {
    pub action: PickupAction,
    pub item: InventoryItem,
    pub quantity: u32,
    pub infinite: bool,
    /// Minimum time in seconds between successful pickups.
    pub cooldown: f32,
}

impl Default for Pickup {
    fn default() -> Self {
        Self {
            action: PickupAction::AddToInventory,
            item: InventoryItem::Gold,
            quantity: 1,
            infinite: false,
            cooldown: DEFAULT_PICKUP_COOLDOWN,
        }
    }
}

/// The closest pickup currently within interaction range of the player.
#[derive(Resource, Default)]
pub(crate) struct NearbyPickup(pub(crate) Option<Entity>);

/// Set by the shared E / interaction prompt input and consumed by the pickup system.
#[derive(Resource, Default)]
pub(crate) struct PickupRequest(pub(crate) Option<Entity>);

#[derive(Component)]
pub(super) struct PickupRuntime {
    remaining: u32,
    cooldown_remaining: f32,
}

impl PickupRuntime {
    fn from_config(pickup: &Pickup) -> Self {
        Self {
            remaining: pickup.quantity,
            cooldown_remaining: 0.0,
        }
    }

    fn tick(&mut self, delta_secs: f32) {
        if delta_secs.is_finite() && delta_secs > 0.0 {
            self.cooldown_remaining = (self.cooldown_remaining - delta_secs).max(0.0);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PickupResult {
    OnCooldown,
    Empty,
    Collected { depleted: bool },
}

fn collect_one(
    pickup: &Pickup,
    runtime: &mut PickupRuntime,
    inventory: &mut InventoryModel,
) -> PickupResult {
    if runtime.cooldown_remaining > 0.0 {
        return PickupResult::OnCooldown;
    }
    if !pickup.infinite && runtime.remaining == 0 {
        return PickupResult::Empty;
    }

    match pickup.action {
        PickupAction::AddToInventory => inventory.add_item(pickup.item, 1),
    }

    if !pickup.infinite {
        runtime.remaining -= 1;
    }

    runtime.cooldown_remaining = if pickup.cooldown.is_finite() {
        pickup.cooldown.max(0.0)
    } else {
        DEFAULT_PICKUP_COOLDOWN
    };

    PickupResult::Collected {
        depleted: !pickup.infinite && runtime.remaining == 0,
    }
}

/// Initializes runtime stock and advances each pickup's cooldown.
pub(super) fn update_pickup_states(
    mut commands: Commands,
    time: Res<Time>,
    mut pickups: Query<(Entity, &Pickup, Option<&mut PickupRuntime>)>,
) {
    for (entity, pickup, runtime) in &mut pickups {
        if let Some(mut runtime) = runtime {
            runtime.tick(time.delta_secs());
        } else {
            commands
                .entity(entity)
                .insert(PickupRuntime::from_config(pickup));
        }
    }
}

/// Finds the nearest pickup in front of the player's normal interaction probe.
pub(super) fn detect_nearby_pickup(
    player: Query<&Transform, With<Player>>,
    pickups: Query<(
        Entity,
        &GlobalTransform,
        &Pickup,
        Option<&PickupRuntime>,
        Option<&Mesh3d>,
    )>,
    meshes: Res<Assets<Mesh>>,
    mut nearby: ResMut<NearbyPickup>,
) {
    let Ok(player_transform) = player.single() else {
        nearby.0 = None;
        return;
    };

    let player_position = player_transform.translation + Vec3::Y * INTERACTION_HEIGHT;
    let mut nearest: Option<(Entity, f32)> = None;

    for (entity, transform, pickup, runtime, mesh_3d) in &pickups {
        let remaining = runtime.map_or(pickup.quantity, |state| state.remaining);
        if !pickup.infinite && remaining == 0 {
            continue;
        }

        let origin = transform.translation();
        if !origin.is_finite() {
            continue;
        }

        // Use the visible mesh bounds instead of the Blender object origin
        // when the pickup is attached directly to a mesh object.
        let closest_point = mesh_3d
            .and_then(|mesh_3d| meshes.get(&mesh_3d.0))
            .and_then(|mesh| closest_point_on_mesh_bounds(player_position, transform, mesh))
            .unwrap_or(origin);
        let offset = closest_point - player_position;
        if offset.y.abs() > PICKUP_MAX_VERTICAL_DISTANCE {
            continue;
        }

        let horizontal_distance_squared = Vec2::new(offset.x, offset.z).length_squared();
        if horizontal_distance_squared > PICKUP_INTERACTION_RADIUS.powi(2) {
            continue;
        }

        if nearest.map_or(true, |(_, distance)| horizontal_distance_squared < distance) {
            nearest = Some((entity, horizontal_distance_squared));
        }
    }

    nearby.0 = nearest.map(|(entity, _)| entity);
}

fn closest_point_on_mesh_bounds(
    world_point: Vec3,
    transform: &GlobalTransform,
    mesh: &Mesh,
) -> Option<Vec3> {
    let aabb = mesh.get_aabb()?;
    let center = Vec3::new(aabb.center.x, aabb.center.y, aabb.center.z);
    let half_extents = Vec3::new(
        aabb.half_extents.x,
        aabb.half_extents.y,
        aabb.half_extents.z,
    );
    let local_point = transform.affine().inverse().transform_point3(world_point);
    let closest_local = center + (local_point - center).clamp(-half_extents, half_extents);
    let closest_world = transform.transform_point(closest_local);
    closest_world.is_finite().then_some(closest_world)
}

/// Applies the requested pickup after the current frame's proximity check.
pub(super) fn handle_pickup_request(
    mut commands: Commands,
    mut request: ResMut<PickupRequest>,
    nearby: Res<NearbyPickup>,
    mut pickups: Query<(&Pickup, &mut PickupRuntime)>,
    mut inventory: ResMut<InventoryModel>,
) {
    let Some(entity) = request.0.take() else {
        return;
    };
    if nearby.0 != Some(entity) {
        return;
    }

    let Ok((pickup, mut runtime)) = pickups.get_mut(entity) else {
        return;
    };

    match collect_one(pickup, &mut runtime, &mut inventory) {
        PickupResult::Collected { depleted: true } => {
            // ChildOf uses linked spawning, so removing the authored root also
            // removes its visual hierarchy.
            commands.entity(entity).despawn();
        }
        PickupResult::Collected { depleted: false }
        | PickupResult::OnCooldown
        | PickupResult::Empty => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_bounds_use_surface_distance_instead_of_object_origin() {
        let center = Vec3::ZERO;
        let half_extents = Vec3::new(1.4, 1.6, 1.0);
        let player_probe = Vec3::new(1.7, 1.7, 0.0);
        let closest = center + (player_probe - center).clamp(-half_extents, half_extents);

        assert_eq!(closest, Vec3::new(1.4, 1.6, 0.0));
        let horizontal_distance =
            Vec2::new(closest.x - player_probe.x, closest.z - player_probe.z).length();
        assert!(horizontal_distance < PICKUP_INTERACTION_RADIUS);
    }

    #[test]
    fn point_inside_mesh_bounds_has_zero_surface_distance() {
        let point = Vec3::new(0.5, 0.2, -0.3);
        assert_eq!(
            Vec3::ZERO + (point - Vec3::ZERO).clamp(-Vec3::ONE, Vec3::ONE),
            point
        );
    }

    #[test]
    fn finite_pickup_grants_one_item_per_interaction_and_depletes() {
        let pickup = Pickup {
            item: InventoryItem::Wood,
            quantity: 2,
            cooldown: 0.0,
            ..default()
        };
        let mut runtime = PickupRuntime::from_config(&pickup);
        let mut inventory = InventoryModel::default();

        assert_eq!(
            collect_one(&pickup, &mut runtime, &mut inventory),
            PickupResult::Collected { depleted: false }
        );
        assert_eq!(inventory.wood, 1);
        assert_eq!(runtime.remaining, 1);

        assert_eq!(
            collect_one(&pickup, &mut runtime, &mut inventory),
            PickupResult::Collected { depleted: true }
        );
        assert_eq!(inventory.wood, 2);
        assert_eq!(runtime.remaining, 0);
        assert_eq!(
            collect_one(&pickup, &mut runtime, &mut inventory),
            PickupResult::Empty
        );
        assert_eq!(inventory.wood, 2);
    }

    #[test]
    fn infinite_pickup_ignores_quantity_and_never_depletes() {
        let pickup = Pickup {
            item: InventoryItem::Stone,
            quantity: 1,
            infinite: true,
            cooldown: 0.0,
            ..default()
        };
        let mut runtime = PickupRuntime::from_config(&pickup);
        let mut inventory = InventoryModel::default();

        for _ in 0..3 {
            assert_eq!(
                collect_one(&pickup, &mut runtime, &mut inventory),
                PickupResult::Collected { depleted: false }
            );
        }

        assert_eq!(inventory.stone, 3);
        assert_eq!(runtime.remaining, 1);
    }

    #[test]
    fn pickup_cooldown_prevents_another_collection_until_it_expires() {
        let pickup = Pickup {
            quantity: 2,
            cooldown: 1.0,
            ..default()
        };
        let mut runtime = PickupRuntime::from_config(&pickup);
        let mut inventory = InventoryModel::default();

        assert_eq!(
            collect_one(&pickup, &mut runtime, &mut inventory),
            PickupResult::Collected { depleted: false }
        );
        assert_eq!(
            collect_one(&pickup, &mut runtime, &mut inventory),
            PickupResult::OnCooldown
        );

        runtime.tick(1.0);

        assert_eq!(
            collect_one(&pickup, &mut runtime, &mut inventory),
            PickupResult::Collected { depleted: true }
        );
        assert_eq!(inventory.gold, 2);
    }
}
