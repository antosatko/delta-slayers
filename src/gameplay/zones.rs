use bevy::prelude::*;

use crate::gameplay::player::Player;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum ZoneKind {
    #[default]
    Shop,
    Guild,
}

#[derive(Component)]
pub struct TriggerZone {
    pub kind: ZoneKind,
    pub radius: f32,
}

#[derive(Resource, Default)]
pub struct NearbyZone(pub Option<ZoneKind>);

#[derive(Resource, Default)]
pub struct VisitedZones(pub std::collections::HashSet<ZoneKind>);

#[derive(Resource, Default)]
pub struct FastTravelRequest(pub Option<ZoneKind>);

pub fn zone_detection_system(
    player_q: Query<&Transform, With<Player>>,
    zones: Query<(&Transform, &TriggerZone)>,
    mut nearby: ResMut<NearbyZone>,
    mut visited: ResMut<VisitedZones>,
) {
    let Ok(player_transform) = player_q.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    let mut found = None;
    for (zone_transform, zone) in &zones {
        if player_pos.distance(zone_transform.translation) <= zone.radius {
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