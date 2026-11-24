mod billboard;
mod persistence;
mod systems;
mod world;

pub mod inventory;
pub mod player;
pub mod zones;

use bevy::prelude::*;

use crate::{app::state::AppState, ui::{stack::UiStack, UiSet}};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<zones::NearbyZone>()
            .init_resource::<zones::VisitedZones>()
            .init_resource::<zones::FastTravelRequest>()
            .init_resource::<inventory::InventoryModel>()
            .add_systems(OnEnter(AppState::InGame), world::setup_game)
            .add_systems(
                Update,
                (
                    player::player_look.run_if(gameplay_input_enabled),
                    player::player_move.run_if(gameplay_input_enabled),
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
                    .after(player::player_move),
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
    !ui_state.paused() && !ui_state.inventory_open()
}
