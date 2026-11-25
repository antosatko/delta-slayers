mod billboard;
mod persistence;
mod spawning;
mod systems;
mod world;

pub mod inventory;
pub mod player;
pub mod zones;

use bevy::prelude::*;

use crate::{
    app::state::AppState,
    ui::{UiSet, stack::UiStack},
};

/// Selects whether a new gameplay session restores the quick-save or starts fresh.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum GameStartMode {
    #[default]
    QuickLoad,
    NewGame,
}

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<zones::ZoneKind>()
            .register_type::<zones::TriggerBox>()
            .register_type::<spawning::SpawnPointKind>()
            .register_type::<spawning::SpawnPoint>()
            .register_type::<billboard::Billboard>()
            .init_resource::<spawning::WorldStartup>()
            .init_resource::<zones::NearbyZone>()
            .init_resource::<zones::VisitedZones>()
            .init_resource::<zones::FastTravelRequest>()
            .init_resource::<inventory::InventoryModel>()
            .init_resource::<GameStartMode>()
            .init_resource::<player::MovementTarget>()
            .add_systems(OnEnter(AppState::Loading), world::setup_game)
            .add_systems(
                Update,
                spawning::apply_player_spawn_point
                    .before(player::follow_camera)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                player::set_move_target_from_cursor
                    .after(UiSet::Input)
                    .before(player::move_player_to_target)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    player::adjust_camera.run_if(gameplay_input_enabled),
                    player::move_player_to_target.run_if(gameplay_input_enabled),
                    player::follow_camera,
                    player::draw_move_trajectory.run_if(gameplay_input_enabled),
                    player::sync_player_visual,
                    billboard::billboard_system,
                    zones::zone_detection_system.run_if(gameplay_input_enabled),
                )
                    .chain()
                    .after(UiSet::Input)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                systems::fast_travel
                    .run_if(in_state(AppState::InGame))
                    .after(player::move_player_to_target),
            )
            .add_systems(
                Update,
                systems::return_to_menu
                    .run_if(in_state(AppState::InGame))
                    .after(UiSet::Input),
            );
    }
}

fn gameplay_input_enabled(ui_state: Res<UiStack>) -> bool {
    !ui_state.paused() && !ui_state.inventory_open() && !ui_state.game_menu_open()
}
