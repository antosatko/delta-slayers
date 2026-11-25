use bevy::prelude::*;

use crate::{
    app::state::AppState,
    gameplay::{
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
) {
    let Some(kind) = request.0.take() else {
        return;
    };
    movement_target.clear();

    let destination = zones
        .iter()
        .find(|(_, zone)| zone.kind == kind)
        .map(|(transform, _)| transform.translation());

    if let Some(position) = destination {
        if let Ok(mut player_transform) = player.single_mut() {
            player_transform.translation = fast_travel_spawn_position(position);
        }
    }
}

fn fast_travel_spawn_position(destination: Vec3) -> Vec3 {
    // Blender/Bevy use Y as up. Offset the exit along Z without flattening
    // the authored destination's elevation.
    Vec3::new(destination.x, destination.y, destination.z + 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_travel_preserves_destination_height_at_any_elevation() {
        for height in [-24.0, 0.0, 7.5, 48.0] {
            let destination = Vec3::new(12.0, height, -4.0);
            let spawn = fast_travel_spawn_position(destination);

            assert_eq!(spawn.y, height);
            assert_eq!(spawn.x, destination.x);
            assert_eq!(spawn.z, destination.z + 2.0);
        }
    }
}

pub(super) fn return_to_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ui_state: ResMut<UiStack>,
    player_query: Query<(&Transform, &Player)>,
    visited: Res<VisitedZones>,
) {
    if !keys.just_pressed(KeyCode::Backspace) {
        return;
    }

    if !ui_state.is_empty() {
        ui_state.pop_top();
        return;
    }

    if let Ok((transform, player)) = player_query.single() {
        if let Err(error) = save_game(transform.translation, player.yaw, player.pitch, &visited.0) {
            warn!("Could not save the current game: {error}");
        }
    }
    next_state.set(AppState::MainMenu);
}
