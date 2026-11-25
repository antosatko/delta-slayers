mod billboard;
pub(crate) mod exploration;
mod npc;
mod occlusion;
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
        app.insert_gizmo_config(
            player::TrajectoryGizmoConfigGroup,
            GizmoConfig {
                depth_bias: -1.0,
                ..default()
            },
        )
            .register_type::<occlusion::PlayerOccluder>()
            .register_type::<zones::ZoneKind>()
            .register_type::<zones::TriggerBox>()
            .register_type::<exploration::ExplorationPortal>()
            .register_type::<player::MovementBlocker>()
            .register_type::<spawning::SpawnPointKind>()
            .register_type::<spawning::SpawnPoint>()
            .register_type::<billboard::Billboard>()
            .register_type::<billboard::SquashOnMove>()
            .init_resource::<spawning::WorldStartup>()
            .init_resource::<zones::NearbyZone>()
            .init_resource::<zones::VisitedZones>()
            .init_resource::<zones::FastTravelRequest>()
            .init_resource::<exploration::NearbyExplorationPortal>()
            .init_resource::<exploration::ExplorationTravelRequest>()
            .init_resource::<exploration::ActiveExplorationMap>()
            .init_resource::<exploration::ExplorationSession>()
            .init_resource::<inventory::InventoryModel>()
            .init_resource::<GameStartMode>()
            .init_resource::<player::MovementTarget>()
            .init_resource::<player::PlayerSettings>()
            .add_systems(OnEnter(AppState::Loading), world::setup_game)
            .add_systems(
                Update,
                spawning::apply_player_spawn_point
                    .before(player::follow_camera)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                npc::spawn_godo
                    .after(spawning::apply_player_spawn_point)
                    .before(npc::chase_player)
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
                    player::adjust_camera.run_if(camera_input_enabled),
                    player::move_player_to_target.run_if(gameplay_simulation_enabled),
                    npc::chase_player.run_if(gameplay_simulation_enabled),
                    exploration::apply_tile_materials,
                    exploration::sync_ground_height,
                    player::follow_camera,
                    occlusion::fade_occluding_objects,
                    player::draw_move_trajectory.run_if(gameplay_simulation_enabled),
                    player::sync_player_visual,
                    billboard::animate_moving_sprites,
                    billboard::billboard_system,
                    zones::zone_detection_system.run_if(gameplay_simulation_enabled),
                    exploration::detect_portal.run_if(gameplay_simulation_enabled),
                )
                    .chain()
                    .after(UiSet::Input)
                    .before(UiSet::Projection)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                systems::fast_travel
                    .run_if(in_state(AppState::InGame))
                    .after(player::move_player_to_target)
                    .before(UiSet::Projection),
            )
            .add_systems(
                Update,
                exploration::process_travel_request
                    .after(exploration::detect_portal)
                    .after(systems::fast_travel)
                    .after(UiSet::Input)
                    .before(UiSet::Projection)
                    .run_if(gameplay_simulation_enabled)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                systems::return_to_menu
                    .run_if(in_state(AppState::InGame))
                    .after(exploration::process_travel_request)
                    .after(UiSet::Input),
            );
    }
}

fn gameplay_simulation_enabled(ui_state: Res<UiStack>) -> bool {
    gameplay_simulation_allowed(&ui_state)
}

fn camera_input_enabled(ui_state: Res<UiStack>) -> bool {
    camera_input_allowed(&ui_state)
}

fn gameplay_simulation_allowed(ui_state: &UiStack) -> bool {
    !ui_state.paused() && !ui_state.game_menu_open() && !ui_state.exploration_setup_open()
}

fn camera_input_allowed(ui_state: &UiStack) -> bool {
    gameplay_simulation_allowed(ui_state) && !ui_state.inventory_open()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::stack::UiLayer;

    #[test]
    fn opening_inventory_does_not_pause_gameplay() {
        let mut ui = UiStack::default();
        ui.push(UiLayer::Inventory);

        assert!(gameplay_simulation_allowed(&ui));
        assert!(!camera_input_allowed(&ui));
    }

    #[test]
    fn pause_and_game_menu_still_suspend_gameplay() {
        let mut ui = UiStack::default();
        ui.push(UiLayer::Pause);
        assert!(!gameplay_simulation_allowed(&ui));

        ui.reset();
        ui.push(UiLayer::GameMenu);
        assert!(!gameplay_simulation_allowed(&ui));
    }

    #[test]
    fn exploration_setup_suspends_gameplay_until_closed() {
        let mut ui = UiStack::default();
        ui.push(UiLayer::ExplorationSetup);

        assert!(ui.exploration_setup_open());
        assert!(!gameplay_simulation_allowed(&ui));

        ui.remove(UiLayer::ExplorationSetup);
        assert!(gameplay_simulation_allowed(&ui));
    }
}
