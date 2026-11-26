use std::collections::HashMap;

use bevy::{
    feathers::controls::{FeathersCheckbox, FeathersNumberInput, NumberInputValue},
    prelude::*,
    ui::Checked,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    gameplay::{
        exploration::{
            ActiveExplorationMap, Biome, ExplorationTile, NearbyExplorationPortal, UnreachableTile,
            hex_center, world_to_axial,
        },
        player::{Player, PlayerSettings},
        pickups::NearbyPickup,
        zones::{NearbyZone, TriggerBox, VisitedZones},
    },
    ui::{
        components::{
            DeveloperSettingsContentRoot, DeveloperSettingsPanelState, ExplorationDebugInfoText,
            ExplorationDebugPanelRoot, ExplorationDebugPanelState, ExplorationParameterControl,
            ExplorationSetupRoot, ExplorationSetupTileCountText, ExplorationSetupViewModel,
            FastTravelButton, FastTravelMenu, GameMenuModal, MinimapCompass, MinimapExplorationDot,
            MinimapZoneDot, PanelRoot, PauseOverlay, PlayerSettingControl, PlayerSettingToggle,
            PopupRoot,
            SettingsPanelRoot,
        },
        inventory::InventoryRoot,
        stack::{UiLayer, UiStack},
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
    nearby_pickup: Res<NearbyPickup>,
    ui: Res<UiStack>,
    mut popups: Query<&mut Visibility, With<PopupRoot>>,
) {
    let visible = (nearby.0.is_some()
        || nearby_portal.0.is_some()
        || nearby_pickup.0.is_some())
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
        *visibility =
            if ui.inventory_open() && active_map.0.is_none() && visited.0.contains(&button.0) {
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
                "Seed: {}\nHex radius: {} ({} tiles)\nTile radius: {:.2} m\nHeight multiplier: {:.2}\nMaximum jump: {:.2} m\nTerrain frequency: {:.3}\nBiome scale: {:.3}",
                map.seed,
                config.radius,
                tile_count,
                config.tile_size,
                config.height_multiplier,
                config.max_jump_height,
                config.terrain_frequency,
                config.biome_scale,
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
                config
                    .seed
                    .map_or(0, |seed| seed.min(i64::MAX as u64) as i64),
            ),
            ExplorationParameterControl::Radius => NumberInputValue::I64(config.radius as i64),
            ExplorationParameterControl::HeightMultiplier => {
                NumberInputValue::F32(config.height_multiplier)
            }
            ExplorationParameterControl::TerrainFrequency => {
                NumberInputValue::F32(config.terrain_frequency)
            }
            ExplorationParameterControl::BiomeScale => NumberInputValue::F32(config.biome_scale),
            ExplorationParameterControl::MaxJumpHeight => {
                NumberInputValue::F32(config.max_jump_height)
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

pub fn settings_panel_visibility_system(
    ui: Res<UiStack>,
    mut roots: Query<&mut Visibility, With<SettingsPanelRoot>>,
) {
    for mut visibility in &mut roots {
        *visibility = if ui.contains(crate::ui::stack::UiLayer::Settings) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn developer_settings_visible(
    ui: &UiStack,
    state: &mut DeveloperSettingsPanelState,
) -> bool {
    if !ui.contains(UiLayer::Settings) {
        state.expanded = false;
        return false;
    }
    state.expanded
}

pub fn developer_settings_projection(
    ui: Res<UiStack>,
    mut state: ResMut<DeveloperSettingsPanelState>,
    mut content: Query<&mut Visibility, With<DeveloperSettingsContentRoot>>,
) {
    let visible = developer_settings_visible(&ui, &mut state);
    for mut visibility in &mut content {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn player_settings_projection(
    settings: Res<PlayerSettings>,
    controls: Query<(Entity, &PlayerSettingControl, &NumberInputValue), With<FeathersNumberInput>>,
    toggles: Query<(Entity, &PlayerSettingToggle, Has<Checked>), With<FeathersCheckbox>>,
    mut commands: Commands,
) {
    for (entity, control, current) in &controls {
        let value = match control {
            PlayerSettingControl::PanSensitivity => settings.camera_pan_sensitivity,
            PlayerSettingControl::ZoomSensitivity => settings.camera_zoom_sensitivity,
            PlayerSettingControl::PitchSensitivity => settings.camera_pitch_sensitivity,
            PlayerSettingControl::WalkSpeed => settings.walk_speed,
            PlayerSettingControl::SprintSpeed => settings.sprint_speed,
        };
        let next = NumberInputValue::F32(value);
        if *current != next {
            commands.entity(entity).insert(next);
        }
    }

    for (entity, control, is_checked) in &toggles {
        let should_be_checked = match control {
            PlayerSettingToggle::CameraAutoLock => settings.camera_auto_lock,
        };
        if is_checked != should_be_checked {
            if should_be_checked {
                commands.entity(entity).insert(Checked);
            } else {
                commands.entity(entity).remove::<Checked>();
            }
        }
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
    active_map: Res<ActiveExplorationMap>,
    exploration_tiles: Query<(&ExplorationTile, Has<UnreachableTile>)>,
    mut compass: Query<
        &mut Text,
        (With<MinimapCompass>, Without<MinimapZoneDot>),
    >,
    mut town_dots: Query<
        (
            &MinimapZoneDot,
            &mut Node,
            &mut Visibility,
            &mut Text,
            &mut TextColor,
        ),
        (Without<MinimapExplorationDot>, Without<MinimapCompass>),
    >,
    mut exploration_dots: Query<
        (
            &MinimapExplorationDot,
            &mut Node,
            &mut Visibility,
            &mut BackgroundColor,
        ),
        Without<MinimapZoneDot>,
    >,
) {
    let Ok(player_transform) = player.single() else {
        for (_, _, mut visibility, _, _) in &mut town_dots {
            *visibility = Visibility::Hidden;
        }
        for (_, _, mut visibility, _) in &mut exploration_dots {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    let player_position = player_transform.translation;
    let player_forward = *player_transform.forward();
    let map = active_map.0;

    let north_arrow = minimap_direction_glyph(Vec3::NEG_Z, player_forward);
    for mut arrow in &mut compass {
        if arrow.0 != north_arrow {
            arrow.0 = north_arrow.to_owned();
        }
    }

    for (dot, mut node, mut visibility, mut text, mut color) in &mut town_dots {
        if map.is_some() {
            *visibility = Visibility::Hidden;
            continue;
        }

        let Some((transform, _)) = zones.iter().find(|(_, zone)| zone.kind == dot.0) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let relative = transform.translation() - player_position;
        let distance = Vec2::new(relative.x, relative.z).length();
        let (x, y) = minimap_position(relative, player_forward);
        let marker_center = Vec2::new(x + 7.0, y + 7.0);
        let direction = Vec2::new(
            relative.dot(minimap_right(player_forward)),
            -relative.dot(minimap_forward(player_forward)),
        )
        .normalize_or_zero();

        let inside_minimap = marker_center.distance(Vec2::splat(60.0)) <= MINIMAP_POI_RADIUS;
        let (position, glyph) = if inside_minimap {
            (marker_center, "•")
        } else {
            (
                Vec2::splat(60.0) + direction * MINIMAP_POI_RADIUS,
                minimap_direction_glyph(relative, player_forward),
            )
        };

        *visibility = Visibility::Visible;
        node.left = px(position.x - MINIMAP_POI_MARKER_HALF_SIZE);
        node.top = px(position.y - MINIMAP_POI_MARKER_HALF_SIZE);
        if text.0 != glyph {
            text.0 = glyph.to_owned();
        }
        color.0 = color.0.with_alpha(poi_marker_opacity(distance));
    }

    let Some(map) = map else {
        for (_, _, mut visibility, _) in &mut exploration_dots {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    let local_player = player_position - map.origin;
    let (center_q, center_r) = world_to_axial(
        Vec2::new(local_player.x, local_player.z),
        map.config.tile_size,
    );
    let tile_index: HashMap<(i32, i32), (Biome, bool)> = exploration_tiles
        .iter()
        .map(|(tile, unreachable)| ((tile.q, tile.r), (tile.biome, unreachable)))
        .collect();

    for (dot, mut node, mut visibility, mut background) in &mut exploration_dots {
        let coordinates = (center_q + dot.0, center_r + dot.1);
        let Some((biome, unreachable)) = tile_index.get(&coordinates) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let center = hex_center(coordinates.0, coordinates.1, map.config.tile_size);
        let relative = Vec3::new(center.x - local_player.x, 0.0, center.y - local_player.z);
        let (x, y) = minimap_position_with_scale(relative, *player_transform.forward(), 1.55);

        let marker_center = Vec2::new(x + 3.0, y + 3.0);
        if !(0.0..=114.0).contains(&x)
            || !(0.0..=114.0).contains(&y)
            || marker_center.distance(Vec2::splat(60.0)) > 53.0
        {
            *visibility = Visibility::Hidden;
            continue;
        }

        *visibility = Visibility::Visible;
        node.left = px(x);
        node.top = px(y);
        *background = BackgroundColor(if *unreachable {
            Color::srgb(0.82, 0.18, 0.16)
        } else {
            biome.minimap_color()
        });
    }
}

const MINIMAP_POI_RADIUS: f32 = 48.0;
const MINIMAP_POI_MARKER_HALF_SIZE: f32 = 7.0;
const POI_FULL_OPACITY_DISTANCE: f32 = 24.0;
const POI_FADE_DISTANCE: f32 = 100.0;
const POI_MIN_OPACITY: f32 = 0.18;

fn poi_marker_opacity(distance: f32) -> f32 {
    if distance <= POI_FULL_OPACITY_DISTANCE {
        return 1.0;
    }

    let fade = ((distance - POI_FULL_OPACITY_DISTANCE)
        / (POI_FADE_DISTANCE - POI_FULL_OPACITY_DISTANCE))
        .clamp(0.0, 1.0);
    1.0 - fade * (1.0 - POI_MIN_OPACITY)
}

fn minimap_forward(player_forward: Vec3) -> Vec3 {
    let horizontal = Vec3::new(player_forward.x, 0.0, player_forward.z);
    if horizontal.length_squared() < 1e-6 {
        Vec3::NEG_Z
    } else {
        horizontal.normalize()
    }
}

fn minimap_right(player_forward: Vec3) -> Vec3 {
    minimap_forward(player_forward).cross(Vec3::Y).normalize()
}

/// Returns an eight-way arrow in minimap coordinates, where north is up.
fn minimap_direction_glyph(relative: Vec3, player_forward: Vec3) -> &'static str {
    const ARROWS: [&str; 8] = ["↑", "↗", "→", "↘", "↓", "↙", "←", "↖"];

    let forward = minimap_forward(player_forward);
    let right = forward.cross(Vec3::Y).normalize();
    let screen_direction = Vec2::new(relative.dot(right), -relative.dot(forward));
    if screen_direction.length_squared() < 1e-6 {
        return ARROWS[0];
    }

    let angle = screen_direction.x.atan2(-screen_direction.y);
    let sector = (angle / std::f32::consts::FRAC_PI_4).round() as i32;
    ARROWS[sector.rem_euclid(8) as usize]
}

/// Projects a world-space offset into a player-oriented minimap.
///
/// The player's horizontal forward direction always points toward the top
/// edge of the map; the right vector defines its horizontal axis.
fn minimap_position(relative: Vec3, player_forward: Vec3) -> (f32, f32) {
    minimap_position_with_scale(relative, player_forward, 3.0)
}

fn minimap_position_with_scale(relative: Vec3, player_forward: Vec3, scale: f32) -> (f32, f32) {
    let mut forward = player_forward;
    forward.y = 0.0;
    if forward.length_squared() < 1e-6 {
        forward = Vec3::NEG_Z;
    } else {
        forward = forward.normalize();
    }

    let right = forward.cross(Vec3::Y).normalize();
    let map_x = 60.0 + relative.dot(right) * scale - 7.0;
    let map_y = 60.0 - relative.dot(forward) * scale - 7.0;
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

    #[test]
    fn poi_arrow_points_toward_north_relative_to_player_heading() {
        assert_eq!(minimap_direction_glyph(Vec3::NEG_Z, Vec3::NEG_Z), "↑");
        assert_eq!(minimap_direction_glyph(Vec3::NEG_Z, Vec3::X), "←");
    }

    #[test]
    fn poi_arrow_points_right_for_a_point_east_of_north_facing_player() {
        assert_eq!(minimap_direction_glyph(Vec3::X, Vec3::NEG_Z), "→");
    }

    #[test]
    fn poi_opacity_is_full_below_threshold_and_fades_with_distance() {
        assert_eq!(poi_marker_opacity(POI_FULL_OPACITY_DISTANCE), 1.0);
        assert!(poi_marker_opacity(60.0) < 1.0);
        assert_eq!(poi_marker_opacity(POI_FADE_DISTANCE), POI_MIN_OPACITY);
        assert_eq!(poi_marker_opacity(500.0), POI_MIN_OPACITY);
    }
}


#[cfg(test)]
mod developer_settings_tests {
    use super::*;

    #[test]
    fn developer_settings_hide_and_collapse_when_settings_close() {
        let mut ui = UiStack::default();
        let mut state = DeveloperSettingsPanelState { expanded: true };

        assert!(!developer_settings_visible(&ui, &mut state));
        assert!(!state.expanded);

        ui.push(UiLayer::Settings);
        state.expanded = true;
        assert!(developer_settings_visible(&ui, &mut state));

        ui.remove(UiLayer::Settings);
        assert!(!developer_settings_visible(&ui, &mut state));
        assert!(!state.expanded);
    }
}
