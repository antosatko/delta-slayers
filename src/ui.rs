use bevy::prelude::*;

use crate::ui_stack::UiStack;
use crate::zone::{NearbyZone, VisitedZones, ZoneKind};

#[derive(Component)]
pub struct PanelRoot(pub ZoneKind);

#[derive(Component)]
pub struct PopupRoot;

#[derive(Component)]
pub struct FastTravelButton(pub ZoneKind);

#[derive(Component)]
pub struct ConfirmPopupButton;

pub fn panel_visibility_system(
    ui_stack: Res<UiStack>,
    mut panels: Query<(&PanelRoot, &mut Visibility)>,
) {
    let active = ui_stack.active_panel();
    for (root, mut visibility) in &mut panels {
        *visibility = if active == Some(root.0) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn popup_visibility_system(
    nearby: Res<NearbyZone>,
    ui_stack: Res<UiStack>,
    mut popups: Query<&mut Visibility, With<PopupRoot>>,
) {
    let show = nearby.0.is_some() && ui_stack.active_panel().is_none() && !ui_stack.inventory_open();
    for mut visibility in &mut popups {
        *visibility = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn fast_travel_button_visibility_system(
    ui_stack: Res<UiStack>,
    visited: Res<VisitedZones>,
    mut buttons: Query<(&FastTravelButton, &mut Visibility)>,
) {
    for (button, mut visibility) in &mut buttons {
        *visibility = if ui_stack.inventory_open() && visited.0.contains(&button.0) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn fast_travel_click_system(
    buttons: Query<(&Interaction, &FastTravelButton), Changed<Interaction>>,
    mut request: ResMut<crate::zone::FastTravelRequest>,
) {
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed {
            request.0 = Some(button.0);
        }
    }
}