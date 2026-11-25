use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    gameplay::{
        player::Player,
        zones::{NearbyZone, TriggerBox, VisitedZones, ZoneKind},
    },
    ui::{
        components::{
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
    ui: Res<UiStack>,
    mut popups: Query<&mut Visibility, With<PopupRoot>>,
) {
    let visible = nearby.0.is_some()
        && ui.active_panel().is_none()
        && !ui.inventory_open()
        && !ui.paused()
        && !ui.game_menu_open();

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
    mut menus: Query<&mut Visibility, With<FastTravelMenu>>,
) {
    for mut visibility in &mut menus {
        *visibility = if ui.inventory_open() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn fast_travel_button_visibility_system(
    ui: Res<UiStack>,
    visited: Res<VisitedZones>,
    mut buttons: Query<(&FastTravelButton, &mut Visibility)>,
) {
    for (button, mut visibility) in &mut buttons {
        *visibility = if ui.inventory_open() && visited.0.contains(&button.0) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
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
    let zone_positions: Vec<(ZoneKind, Vec3)> = zones
        .iter()
        .map(|(transform, zone)| (zone.kind, transform.translation()))
        .collect();

    for (dot, mut node, mut visibility) in &mut dots {
        let Some((_, position)) = zone_positions.iter().find(|(kind, _)| *kind == dot.0) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let relative = *position - player_position;
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
        let north_facing = Dir3::new(Vec3::NEG_Z).expect("valid direction");
        let east_facing = Dir3::new(Vec3::X).expect("valid direction");

        let north_right = minimap_position(Vec3::X * 10.0, north_facing);
        let east_right = minimap_position(Vec3::Z * 10.0, east_facing);

        assert!(north_right.0 > 60.0);
        assert!(east_right.0 > 60.0);
    }
}
