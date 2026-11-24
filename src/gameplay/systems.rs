use bevy::prelude::*;

use crate::{
    app::state::AppState,
    gameplay::{persistence::save_game, player::Player, zones::{FastTravelRequest, TriggerZone, VisitedZones}},
    ui::stack::UiStack,
};

pub(super) fn fast_travel(
    mut request: ResMut<FastTravelRequest>,
    mut queries: ParamSet<(
        Query<&mut Transform, With<Player>>,
        Query<(&Transform, &TriggerZone)>,
    )>,
) {
    let Some(kind) = request.0.take() else {
        return;
    };

    let destination = {
        let zones = queries.p1();
        zones
            .iter()
            .find(|(_, zone)| zone.kind == kind)
            .map(|(transform, _)| transform.translation)
    };

    if let Some(position) = destination {
        if let Ok(mut player_transform) = queries.p0().single_mut() {
            player_transform.translation = position + Vec3::new(0.0, 1.7, 2.0);
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
        save_game(transform.translation, player.yaw, player.pitch, &visited.0);
    }
    next_state.set(AppState::MainMenu);
}
