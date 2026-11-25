use bevy::{
    feathers::controls::{FeathersNumberInput, NumberInputValue},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    gameplay::{
        exploration::{ActiveExplorationMap, NearbyExplorationPortal},
        player::Player,
        zones::{NearbyZone, TriggerBox, VisitedZones},
    },
    ui::{
        components::{
            ExplorationDebugInfoText, ExplorationDebugPanelRoot, ExplorationDebugPanelState,
            ExplorationParameterControl, ExplorationSetupRoot, ExplorationSetupTileCountText,
            ExplorationSetupViewModel,
            FastTravelButton, FastTravelMenu, GameMenuModal, MinimapZoneDot, PanelRoot,
            PauseOverlay, PopupRoot,
        },
        inventory::InventoryRoot,
        stack::UiStack,
    },
};

pub fn panel_visibility_system(ui: Res<UiStack>, mut panels: Query<(&PanelRoot, &mut Visibility)>) {
    let active = ui.active_panel();
    for (panel, mut visibility) in &mut panels {
        *visibility = if active == Some(panel.0) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn popup_visibility_system(
    nearby: Res<NearbyZone>,
    nearby_portal: Res<NearbyExplorationPortal>,
    ui: Res<UiStack>,
    mut popups: Query<&mut Visibility, With<PopupRoot>>,
) {
    let visible = (nearby.0.is_some() || nearby_portal.0.is_some())
        && ui.active_panel().is_none()
        && !ui.inventory_open()
        && !ui.paused()
        && !ui.game_menu_open()
        && !ui.exploration_setup_open();

    for mut visibility in &mut popups {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn fast_travel_menu_visibility_system(
    ui: Res<UiStack>,
    active_map: Res<ActiveExplorationMap>,
    mut menus: Query<&mut Visibility, With<FastTravelMenu>>,
) {
    for mut visibility in &mut menus {
        *visibility = if ui.inventory_open() && active_map.0.is_none() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn fast_travel_button_visibility_system(
    ui: Res<UiStack>,
    visited: Res<VisitedZones>,
    active_map: Res<ActiveExplorationMap>,
    mut buttons: Query<(&FastTravelButton, &mut Visibility)>,
) {
    for (button, mut visibility) in &mut buttons {
        *visibility = if ui.inventory_open()
            && active_map.0.is_none()
            && visited.0.contains(&button.0)
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Shows the generated map's actual parameters after entering it in debug builds.
pub fn exploration_debug_panel_projection(
    active_map: Res<ActiveExplorationMap>,
    mut panel_state: ResMut<ExplorationDebugPanelState>,
    mut was_active: Local<bool>,
    mut panels: Query<&mut Visibility, With<ExplorationDebugPanelRoot>>,
    mut labels: Query<&mut Text, With<ExplorationDebugInfoText>>,
) {
    let is_active = active_map.0.is_some();

    if is_active && !*was_active {
        panel_state.visible = cfg!(debug_assertions);

        if let Some(map) = active_map.0 {
            let config = map.config;
            let tile_count = 1 + 3 * config.radius * (config.radius + 1);
            let details = format!(
                "Seed: {}\nHex radius: {} ({} tiles)\nTile size: {:.2}\nHeight multiplier: {:.2}\nTerrain frequency: {:.3}\nBiome scale: {:.3}\nVegetation density: {:.0}%",
                map.seed,
                config.radius,
                tile_count,
                config.tile_size,
                config.height_multiplier,
                config.terrain_frequency,
                config.biome_scale,
                config.vegetation_density * 100.0,
            );

            for mut label in &mut labels {
                label.0.clone_from(&details);
            }
        }
    }

    if !is_active {
        panel_state.visible = false;
    }

    for mut panel in &mut panels {
        *panel = if panel_state.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    *was_active = is_active;
}

/// Projects the player-facing portal setup modal and synchronizes its Feathers inputs.
pub fn exploration_setup_projection(
    ui: Res<UiStack>,
    view_model: Res<ExplorationSetupViewModel>,
    mut commands: Commands,
    mut roots: Query<&mut Visibility, With<ExplorationSetupRoot>>,
    controls: Query<
        (Entity, &ExplorationParameterControl, &NumberInputValue),
        With<FeathersNumberInput>,
    >,
    mut summaries: Query<&mut Text, With<ExplorationSetupTileCountText>>,
) {
    let visible = ui.exploration_setup_open();
    for mut root in &mut roots {
        *root = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    let config = view_model.config;
    let tile_count = 1 + 3 * config.radius * (config.radius + 1);
    let summary = format!(
        "{tile_count} hexes · approx. {:.0} world units across",
        config.radius as f32 * config.tile_size * 3.0
    );
    for mut text in &mut summaries {
        if text.0 != summary {
            text.0.clone_from(&summary);
        }
    }

    if !view_model.is_changed() {
        return;
    }

    for (entity, control, current) in &controls {
        let next = match control {
            ExplorationParameterControl::Seed => NumberInputValue::I64(
                config.seed.map_or(0, |seed| seed.min(i64::MAX as u64) as i64),
            ),
            ExplorationParameterControl::Radius => {
                NumberInputValue::I64(config.radius as i64)
            }
            ExplorationParameterControl::TileSize => NumberInputValue::F32(config.tile_size),
            ExplorationParameterControl::HeightMultiplier => {
                NumberInputValue::F32(config.height_multiplier)
            }
            ExplorationParameterControl::TerrainFrequency => {
                NumberInputValue::F32(config.terrain_frequency)
            }
            ExplorationParameterControl::BiomeScale => {
                NumberInputValue::F32(config.biome_scale)
            }
            ExplorationParameterControl::VegetationDensity => {
                NumberInputValue::F32(config.vegetation_density * 100.0)
            }
        };

        if *current != next {
            commands.entity(entity).insert(next);
        }
    }
}

pub fn inventory_visibility_system(
    ui: Res<UiStack>,
    mut roots: Query<&mut Visibility, With<InventoryRoot>>,
) {
    for mut visibility in &mut roots {
        *visibility = if ui.inventory_open() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn game_menu_visibility_system(
    ui: Res<UiStack>,
    mut menus: Query<&mut Visibility, With<GameMenuModal>>,
) {
    for mut visibility in &mut menus {
        *visibility = if ui.game_menu_open() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn pause_overlay_visibility_system(
    ui: Res<UiStack>,
    mut overlays: Query<&mut Visibility, With<PauseOverlay>>,
) {
    for mut visibility in &mut overlays {
        *visibility = if ui.paused() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn sync_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;
}

pub fn update_minimap(
    player: Query<&Transform, With<Player>>,
    zones: Query<(&GlobalTransform, &TriggerBox)>,
    mut dots: Query<(&MinimapZoneDot, &mut Node, &mut Visibility)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };

    let player_position = player_transform.translation;

    for (dot, mut node, mut visibility) in &mut dots {
        let Some((transform, _)) = zones.iter().find(|(_, zone)| zone.kind == dot.0) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let relative = transform.translation() - player_position;
        let (x, y) = minimap_position(relative, *player_transform.forward());

        if !(0.0..=114.0).contains(&x) || !(0.0..=114.0).contains(&y) {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
            node.left = px(x);
            node.top = px(y);
        }
    }
}

/// Projects a world-space offset into a player-oriented minimap.
///
/// The player's horizontal forward direction always points toward the top
/// edge of the map; the right vector defines its horizontal axis.
fn minimap_position(relative: Vec3, player_forward: Vec3) -> (f32, f32) {
    let mut forward = player_forward;
    forward.y = 0.0;
    if forward.length_squared() < 1e-6 {
        forward = Vec3::NEG_Z;
    } else {
        forward = forward.normalize();
    }

    let right = forward.cross(Vec3::Y).normalize();
    let map_x = 60.0 + relative.dot(right) * 3.0 - 3.0;
    let map_y = 60.0 - relative.dot(forward) * 3.0 - 3.0;
    (map_x, map_y)
}

#[cfg(test)]
mod minimap_tests {
    use super::*;

    #[test]
    fn forward_is_always_at_the_top_of_the_minimap() {
        let north_facing = Vec3::NEG_Z;
        let east_facing = Vec3::X;

        let north = minimap_position(Vec3::NEG_Z * 10.0, north_facing);
        let east = minimap_position(Vec3::X * 10.0, east_facing);

        assert!(north.1 < 60.0);
        assert!(east.1 < 60.0);
    }

    #[test]
    fn right_of_player_is_always_on_the_right_of_the_minimap() {
        let north_facing = Vec3::NEG_Z;
        let east_facing = Vec3::X;

        let north_right = minimap_position(Vec3::X * 10.0, north_facing);
        let east_right = minimap_position(Vec3::Z * 10.0, east_facing);

        assert!(north_right.0 > 60.0);
        assert!(east_right.0 > 60.0);
    }
}
