use bevy::prelude::*;

use crate::gameplay::player::{Player, INTERACTION_HEIGHT};

/// High-level interaction category used by UI panels and fast travel.
#[derive(Reflect, Default, Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[reflect(Default)]
#[type_path = "delta_slayers::gameplay::zones"]
pub enum ZoneKind {
    #[default]
    Shop,
    Guild,
}

/// Authored on a Blender cube to define an oriented interaction volume.
///
/// The default Blender cube has local bounds from -1.0 to 1.0 on each axis.
/// Its world transform (including rotation and scale) defines the trigger box.
#[derive(Component, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::zones"]
pub struct TriggerBox {
    pub kind: ZoneKind,
}

impl Default for TriggerBox {
    fn default() -> Self {
        Self {
            kind: ZoneKind::Shop,
        }
    }
}

#[derive(Resource, Default)]
pub struct NearbyZone(pub Option<ZoneKind>);

#[derive(Resource, Default)]
pub struct VisitedZones(pub std::collections::HashSet<ZoneKind>);

#[derive(Resource, Default)]
pub struct FastTravelRequest(pub Option<ZoneKind>);

/// Detects authored trigger boxes in world space, including rotated and
/// scaled boxes nested under transformed glTF objects.
pub fn zone_detection_system(
    player_q: Query<&Transform, With<Player>>,
    zones: Query<(&GlobalTransform, &TriggerBox)>,
    mut nearby: ResMut<NearbyZone>,
    mut visited: ResMut<VisitedZones>,
) {
    let Ok(player_transform) = player_q.single() else {
        return;
    };
    // Keep authored trigger volumes working as before: interaction is tested
    // around the player's upper body, while the Player transform stays at ground level.
    let player_pos = player_transform.translation + Vec3::Y * INTERACTION_HEIGHT;

    let mut found = None;
    for (zone_transform, zone) in &zones {
        let local_player_pos = zone_transform
            .affine()
            .inverse()
            .transform_point3(player_pos);

        if local_player_pos.abs().cmple(Vec3::ONE).all() {
            found = Some(zone.kind);
            break;
        }
    }

    if found != nearby.0 {
        nearby.0 = found;
        if let Some(kind) = found {
            visited.0.insert(kind);
        }
    }
}
