mod components;
mod inventory;
mod screens;
mod systems;

pub mod stack;

use bevy::prelude::*;

use crate::app::state::AppState;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiSet {
    Input,
    Projection,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<stack::UiStack>()
            .init_resource::<inventory::InventoryViewModel>()
            .init_resource::<components::PopupConfirmRequested>()
            .configure_sets(Update, (UiSet::Input, UiSet::Projection).chain())
            .add_systems(OnEnter(AppState::MainMenu), screens::menu::scene.spawn())
            .add_systems(
                OnEnter(AppState::Continue),
                screens::placeholders::continue_screen.spawn(),
            )
            .add_systems(
                OnEnter(AppState::Load),
                screens::placeholders::load_screen.spawn(),
            )
            .add_systems(
                OnEnter(AppState::Settings),
                screens::placeholders::settings_screen.spawn(),
            )
            .add_systems(
                OnEnter(AppState::Credits),
                screens::placeholders::credits_screen.spawn(),
            )
            .add_systems(OnEnter(AppState::Loading), screens::loading::scene.spawn())
            .add_systems(OnEnter(AppState::InGame), screens::gameplay::scene.spawn())
            .add_systems(
                Update,
                systems::handle_ui_input
                    .in_set(UiSet::Input)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    systems::panel_visibility_system,
                    systems::popup_visibility_system,
                    systems::fast_travel_menu_visibility_system,
                    systems::fast_travel_button_visibility_system,
                    systems::inventory_visibility_system,
                    systems::game_menu_visibility_system,
                    systems::pause_overlay_visibility_system,
                    systems::sync_cursor,
                    systems::update_minimap,
                    inventory::sync_inventory_view_model,
                )
                    .chain()
                    .in_set(UiSet::Projection)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
