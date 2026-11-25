use bevy::prelude::*;

use super::Player;

pub(in crate::gameplay) const PLAYER_VISUAL_HEIGHT: f32 = 0.8;

#[derive(Component)]
pub(in crate::gameplay) struct PlayerVisual;

/// Projects the player's ground position to the separate billboard entity.
pub(in crate::gameplay) fn sync_player_visual(
    player: Query<&Transform, With<Player>>,
    mut visual: Query<&mut Transform, (With<PlayerVisual>, Without<Player>)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let Ok(mut visual_transform) = visual.single_mut() else {
        return;
    };

    visual_transform.translation = player_transform.translation + Vec3::Y * PLAYER_VISUAL_HEIGHT;
}
