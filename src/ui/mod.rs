mod components;
mod inventory;
mod screens;
mod systems;

pub mod stack;

use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};

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
            .init_resource::<components::ExplorationDebugPanelState>()
            .init_resource::<components::DeveloperSettingsPanelState>()
            .init_resource::<components::ExplorationSetupViewModel>()
            .configure_sets(Update, (UiSet::Input, UiSet::Projection).chain())
            .add_systems(OnEnter(AppState::MainMenu), screens::menu::scene.spawn())
            .add_systems(
                OnEnter(AppState::Load),
                screens::placeholders::load_screen.spawn(),
            )
            .add_systems(
                OnEnter(AppState::Settings),
                screens::settings::scene.spawn(),
            )
            .add_systems(
                OnEnter(AppState::Credits),
                screens::placeholders::credits_screen.spawn(),
            )
            .add_systems(OnEnter(AppState::Loading), screens::loading::scene.spawn())
            .add_systems(
                OnEnter(AppState::LoadingPreview),
                screens::loading::preview_scene.spawn(),
            )
            .add_systems(
                Update,
                close_loading_preview_on_input.run_if(in_state(AppState::LoadingPreview)),
            )
            .add_systems(
                OnEnter(AppState::InGame),
                (screens::gameplay::scene.spawn(), systems::sync_cursor),
            )
            .add_systems(
                Update,
                systems::handle_ui_input
                    .in_set(UiSet::Input)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                systems::player_settings_projection
                    .in_set(UiSet::Projection)
                    .run_if(in_state(AppState::Settings)),
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
                    systems::settings_panel_visibility_system,
                    systems::developer_settings_projection,
                    systems::player_settings_projection,
                    systems::update_minimap,
                    systems::exploration_debug_panel_projection,
                    systems::exploration_setup_projection,
                    inventory::sync_inventory_view_model,
                )
                    .chain()
                    .in_set(UiSet::Projection)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

fn close_loading_preview_on_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepad: Res<ButtonInput<GamepadButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let keyboard_input = keyboard.get_just_pressed().next().is_some();
    let mouse_button_input = mouse.get_just_pressed().next().is_some();
    let gamepad_input = gamepad.get_just_pressed().next().is_some();
    let mouse_motion_input = mouse_motion.delta != Vec2::ZERO;
    let mouse_scroll_input = mouse_scroll.delta != Vec2::ZERO;

    if keyboard_input
        || mouse_button_input
        || gamepad_input
        || mouse_motion_input
        || mouse_scroll_input
    {
        next_state.set(AppState::MainMenu);
    }
}
