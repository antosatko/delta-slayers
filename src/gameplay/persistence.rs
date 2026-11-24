use bevy::prelude::*;

use crate::gameplay::zones::ZoneKind;

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
    visited: &std::collections::HashSet<ZoneKind>,
) {
    let visited_zones: Vec<&str> = visited.iter().map(|kind| zone_kind_str(*kind)).collect();
    let content = format!(
        "pos {} {} {}\nyaw {}\npitch {}\nvisited {}\n",
        position.x,
        position.y,
        position.z,
        yaw,
        pitch,
        visited_zones.join(",")
    );
    let _ = std::fs::write("save.txt", content);
}

pub(super) fn load_game() -> (Vec3, f32, f32, std::collections::HashSet<ZoneKind>) {
    let mut position = Vec3::ZERO;
    let mut yaw = 0.0;
    let mut pitch = 0.0;
    let mut visited = std::collections::HashSet::new();

    if let Ok(content) = std::fs::read_to_string("save.txt") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "pos" if parts.len() >= 4 => {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>(),
                    ) {
                        position = Vec3::new(x, y, z);
                    }
                }
                "yaw" if parts.len() >= 2 => {
                    if let Ok(value) = parts[1].parse::<f32>() {
                        yaw = value;
                    }
                }
                "pitch" if parts.len() >= 2 => {
                    if let Ok(value) = parts[1].parse::<f32>() {
                        pitch = value;
                    }
                }
                "visited" if parts.len() >= 2 => {
                    for token in parts[1].split(',') {
                        if let Some(kind) = parse_zone_kind(token) {
                            visited.insert(kind);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    (position, yaw, pitch, visited)
}
