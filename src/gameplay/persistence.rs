use bevy::prelude::*;
use std::{collections::HashSet, io};

use crate::gameplay::{
    inventory::{InventoryItem, InventoryModel, ItemKind, ResourceKind, DEFAULT_MAX_CAPACITY},
    player::{self, CAMERA_DEFAULT_PITCH, CAMERA_MAX_PITCH, CAMERA_MIN_PITCH},
    zones::ZoneKind,
};

const SAVE_VERSION: u32 = 3;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SaveData {
    pub(super) position: Vec3,
    pub(super) yaw: f32,
    pub(super) pitch: f32,
    pub(super) visited_zones: HashSet<ZoneKind>,
    pub(super) inventory: InventoryModel,
}

fn zone_kind_str(kind: ZoneKind) -> &'static str {
    match kind {
        ZoneKind::Shop => "Shop",
        ZoneKind::Guild => "Guild",
    }
}

fn parse_zone_kind(value: &str) -> Option<ZoneKind> {
    match value {
        "Shop" => Some(ZoneKind::Shop),
        "Guild" => Some(ZoneKind::Guild),
        _ => None,
    }
}

pub(super) fn save_game(
    position: Vec3,
    yaw: f32,
    pitch: f32,
    visited: &HashSet<ZoneKind>,
    inventory: &InventoryModel,
) -> io::Result<()> {
    if !position.is_finite() || !yaw.is_finite() || !pitch.is_finite() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "save contains non-finite player state",
        ));
    }

    let save = SaveData {
        position,
        yaw,
        pitch,
        visited_zones: visited.clone(),
        inventory: inventory.clone(),
    };

    std::fs::write("save.txt", serialize_save(&save))
}

pub(super) fn load_game() -> Option<SaveData> {
    let content = match std::fs::read_to_string("save.txt") {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return None,
        Err(error) => {
            warn!("Could not read save.txt: {error}");
            return None;
        }
    };

    let save = parse_save(&content);
    if save.is_none() {
        warn!("save.txt is malformed or contains non-finite values; starting without a save");
    }
    save
}

fn serialize_save(save: &SaveData) -> String {
    let mut visited_zones: Vec<&str> = save
        .visited_zones
        .iter()
        .map(|kind| zone_kind_str(*kind))
        .collect();
    visited_zones.sort_unstable();

    let resources = ResourceKind::ALL
        .into_iter()
        .map(|kind| format!("{} {}", kind.save_key(), save.inventory.resource_amount(kind)))
        .collect::<Vec<_>>()
        .join("\n");
    let items = save.inventory.items()
        .iter()
        .map(|item| format!("{}:{}", item.id, item.kind.save_key()))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "version {SAVE_VERSION}\npos {} {} {}\nyaw {}\npitch {}\nvisited {}\ncapacity {}\n{resources}\nitems {items}\n",
        save.position.x,
        save.position.y,
        save.position.z,
        save.yaw,
        save.pitch,
        visited_zones.join(","),
        save.inventory.max_capacity_subunits,
    )
}

/// Versionless saves predate camera pitch. Versions 1 and 2 predate inventory
/// persistence and receive the current starter inventory on migration.
fn parse_save(content: &str) -> Option<SaveData> {
    let mut version = 1;
    let mut position = None;
    let mut yaw = 0.0;
    let mut pitch = None;
    let mut visited_zones = HashSet::new();
    let mut capacity = DEFAULT_MAX_CAPACITY;
    let mut gold = 0;
    let mut wood = 0;
    let mut stone = 0;
    let mut items: Option<Vec<InventoryItem>> = None;

    for line in content.lines() {
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else {
            continue;
        };

        match key {
            "version" => version = parts.next()?.parse().ok()?,
            "pos" => {
                let x = parse_finite(parts.next()?)?;
                let y = parse_finite(parts.next()?)?;
                let z = parse_finite(parts.next()?)?;
                position = Some(Vec3::new(x, y, z));
            }
            "yaw" => yaw = parse_finite(parts.next()?)?,
            "pitch" => pitch = Some(parse_finite(parts.next()?)?),
            "visited" => {
                if let Some(kinds) = parts.next() {
                    for kind in kinds.split(',') {
                        if let Some(kind) = parse_zone_kind(kind) {
                            visited_zones.insert(kind);
                        }
                    }
                }
            }
            "capacity" => capacity = parts.next()?.parse().ok()?,
            "gold" => gold = parts.next()?.parse().ok()?,
            "wood" => wood = parts.next()?.parse().ok()?,
            "stone" => stone = parts.next()?.parse().ok()?,
            "items" => {
                let mut parsed = Vec::new();
                if let Some(serialized_items) = parts.next() {
                    for entry in serialized_items.split(',') {
                        let Some((id, kind)) = entry.split_once(':') else {
                            return None;
                        };
                        let id = id.parse().ok()?;
                        let kind = ItemKind::from_save_key(kind)?;
                        parsed.push(InventoryItem { id, kind });
                    }
                }
                items = Some(parsed);
            }
            _ => {}
        }
    }

    if version > SAVE_VERSION {
        return None;
    }

    let position = position?;
    let pitch = match pitch {
        Some(pitch) if version < 2 => player::migrate_saved_camera_pitch(pitch),
        Some(pitch) => pitch.clamp(CAMERA_MIN_PITCH, CAMERA_MAX_PITCH),
        None => CAMERA_DEFAULT_PITCH,
    };

    let inventory = if version >= 3 {
        InventoryModel::from_saved(capacity, gold, wood, stone, items?)
    } else {
        InventoryModel::default()
    };

    Some(SaveData {
        position,
        yaw,
        pitch,
        visited_zones,
        inventory,
    })
}

fn parse_finite(value: &str) -> Option<f32> {
    let value = value.parse::<f32>().ok()?;
    value.is_finite().then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versioned_save_round_trip_preserves_player_and_inventory_state() {
        let mut inventory = InventoryModel::default();
        inventory.add_resource_manually(ResourceKind::Gold, 123);
        inventory.add_item_manually(ItemKind::Salad);
        inventory.max_capacity_subunits = 12_345;
        let save = SaveData {
            position: Vec3::ZERO,
            yaw: 1.25,
            pitch: -0.78,
            visited_zones: [ZoneKind::Shop, ZoneKind::Guild].into(),
            inventory,
        };

        let loaded = parse_save(&serialize_save(&save)).unwrap();

        assert_eq!(loaded, save);
    }

    #[test]
    fn legacy_pitch_is_migrated_once_and_inventory_defaults() {
        let legacy = "pos 3 0 -2\nyaw 1.0\npitch -0.9\nvisited Shop,Guild\n";
        let loaded = parse_save(legacy).unwrap();
        let migrated_pitch = player::migrate_saved_camera_pitch(-0.9);

        assert_eq!(loaded.pitch, migrated_pitch);
        assert_eq!(loaded.inventory, InventoryModel::default());

        let loaded_again = parse_save(&serialize_save(&loaded)).unwrap();
        assert_eq!(loaded_again.pitch, migrated_pitch);
    }

    #[test]
    fn version_two_save_migrates_without_changing_camera_pitch() {
        let version_two = "version 2\npos 3 0 -2\nyaw 1.0\npitch -0.9\nvisited Shop\n";
        let loaded = parse_save(version_two).unwrap();

        assert_eq!(loaded.pitch, -0.9);
        assert_eq!(loaded.inventory, InventoryModel::default());
    }

    #[test]
    fn malformed_or_non_finite_positions_are_rejected() {
        assert!(parse_save("pos NaN 0 0").is_none());
        assert!(parse_save("yaw 1.0").is_none());
    }

    #[test]
    fn unsupported_future_versions_are_rejected() {
        assert!(parse_save("version 999\npos 0 0 0").is_none());
    }
}
