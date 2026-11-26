use bevy::prelude::*;

use crate::{
    app::state::AppState,
    gameplay::{
        exploration::{ActiveExplorationMap, ExplorationSession},
        inventory::InventoryModel,
        persistence::save_game,
        player::{MovementTarget, Player},
        zones::{FastTravelRequest, TriggerBox, VisitedZones},
    },
    ui::stack::UiStack,
};

pub(super) fn fast_travel(
    mut request: ResMut<FastTravelRequest>,
    mut movement_target: ResMut<MovementTarget>,
    mut player: Query<&mut Transform, With<Player>>,
    zones: Query<(&GlobalTransform, &TriggerBox)>,
    active_map: Res<ActiveExplorationMap>,
    inventory: Res<InventoryModel>,
) {
    let Some(kind) = request.0.take() else {
        return;
    };
    if active_map.0.is_some() || !inventory.can_use_portal() {
        return;
    }
    movement_target.clear();

    let destination = zones
        .iter()
        .find(|(_, zone)| zone.kind == kind)
        .map(|(transform, _)| transform.translation());

    if let Some(destination) = destination
        && let Ok(mut player_transform) = player.single_mut() {
            let player_height = player_transform.translation.y;
            player_transform.translation = fast_travel_spawn_position(destination, player_height);
        }
}

fn fast_travel_spawn_position(destination: Vec3, player_height: f32) -> Vec3 {
    // Use the destination's horizontal position, but retain the player's
    // current elevation. Trigger-box height must not move the player vertically.
    Vec3::new(destination.x, player_height, destination.z + 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_travel_preserves_player_height_regardless_of_destination_height() {
        let player_heights = [-12.0, 0.0, 3.5, 48.0];
        let destination_heights = [-30.0, 0.0, 7.5, 120.0];

        for (player_height, destination_height) in
            player_heights.into_iter().zip(destination_heights)
        {
            let destination = Vec3::new(12.0, destination_height, -4.0);
            let spawn = fast_travel_spawn_position(destination, player_height);

            assert_eq!(spawn.y, player_height);
            assert_eq!(spawn.x, destination.x);
            assert_eq!(spawn.z, destination.z + 2.0);
        }
    }

    #[test]
    fn fast_travel_does_not_inherit_trigger_height() {
        let destination = Vec3::new(-3.0, 100.0, 8.0);
        let player_height = 2.25;

        let spawn = fast_travel_spawn_position(destination, player_height);

        assert_eq!(spawn.y, 2.25);
        assert_ne!(spawn.y, destination.y);
    }
}

pub(super) fn return_to_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ui_state: ResMut<UiStack>,
    player_query: Query<(&Transform, &Player)>,
    visited: Res<VisitedZones>,
    exploration: Res<ExplorationSession>,
    inventory: Res<InventoryModel>,
) {
    if !keys.just_pressed(KeyCode::Backspace) {
        return;
    }

    if !ui_state.is_empty() {
        ui_state.pop_top();
        return;
    }

    let save_state = exploration
        .0
        .map(|anchor| (anchor.transform.translation, anchor.yaw, anchor.pitch))
        .or_else(|| {
            player_query
                .single()
                .ok()
                .map(|(transform, player)| (transform.translation, player.yaw, player.pitch))
        });
    if let Some((position, yaw, pitch)) = save_state
        && let Err(error) = save_game(position, yaw, pitch, &visited.0, &inventory) {
            warn!("Could not save the current game: {error}");
        }
    next_state.set(AppState::MainMenu);
}
