use bevy::prelude::*;

use crate::gameplay::player::Player;

/// High-level interaction category used by UI panels and fast travel.
#[derive(Reflect, Default, Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[reflect(Default)]
#[type_path = "delta_slayers::gameplay::zones"]
pub enum ZoneKind {
    #[default]
    Shop,
    Guild,
}

/// Authored on a Blender object or Empty to make it an interaction zone.
///
/// `radius` is measured in world units from the object's world-space origin.
#[derive(Component, Reflect, Debug)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::zones"]
pub struct TriggerZone {
    pub kind: ZoneKind,
    pub radius: f32,
}

impl Default for TriggerZone {
    fn default() -> Self {
        Self {
            kind: ZoneKind::Shop,
            radius: 2.5,
        }
    }
}

#[derive(Resource, Default)]
pub struct NearbyZone(pub Option<ZoneKind>);

#[derive(Resource, Default)]
pub struct VisitedZones(pub std::collections::HashSet<ZoneKind>);

#[derive(Resource, Default)]
pub struct FastTravelRequest(pub Option<ZoneKind>);

/// Detects authored zones in world space, including zones nested under
/// transformed glTF objects.
pub fn zone_detection_system(
    player_q: Query<&GlobalTransform, With<Player>>,
    zones: Query<(&GlobalTransform, &TriggerZone)>,
    mut nearby: ResMut<NearbyZone>,
    mut visited: ResMut<VisitedZones>,
) {
    let Ok(player_transform) = player_q.single() else {
        return;
    };
    let player_pos = player_transform.translation();

    let mut found = None;
    for (zone_transform, zone) in &zones {
        if player_pos.distance(zone_transform.translation()) <= zone.radius {
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
