use bevy::prelude::*;

use crate::gameplay::player::Player;

/// The purpose of an authored spawn point. Only Player is consumed at the
/// moment; Enemy and Npc points are available for future spawn systems.
#[derive(Reflect, Default, Clone, Copy, PartialEq, Eq, Debug)]
#[reflect(Default)]
#[type_path = "delta_slayers::gameplay::spawning"]
pub enum SpawnPointKind {
    #[default]
    Player,
    Enemy,
    Npc,
}

/// Place this component on a Blender Empty (or another object) to author a
/// spawn location. Higher priorities win when multiple Player points exist.
#[derive(Component, Reflect, Default, Clone, Copy, Debug)]
#[reflect(Component, Default)]
#[type_path = "delta_slayers::gameplay::spawning"]
pub struct SpawnPoint {
    pub kind: SpawnPointKind,
    pub priority: i32,
}

/// Tracks the initial world/spawn-point handshake for the current game session.
#[derive(Resource, Default)]
pub(super) struct WorldStartup {
    pub(super) scene_ready: bool,
    pub(super) has_saved_position: bool,
    pub(super) spawn_point_applied: bool,
}

/// Applies the highest-priority authored Player spawn point once the Blender
/// world has finished spawning. Saved positions always take precedence.
pub(super) fn apply_player_spawn_point(
    mut startup: ResMut<WorldStartup>,
    spawn_points: Query<(&GlobalTransform, &SpawnPoint)>,
    mut player: Query<(&mut Transform, &mut Player)>,
) {
    if !startup.scene_ready || startup.spawn_point_applied {
        return;
    }

    startup.spawn_point_applied = true;
    if startup.has_saved_position {
        return;
    }

    let selected = spawn_points
        .iter()
        .filter(|(_, point)| point.kind == SpawnPointKind::Player)
        .max_by_key(|(_, point)| point.priority);

    let Some((spawn_transform, _)) = selected else {
        return;
    };

    let (_, rotation, translation) = spawn_transform.to_scale_rotation_translation();
    let (yaw, _, _) = rotation.to_euler(EulerRot::YXZ);

    let Ok((mut transform, mut player)) = player.single_mut() else {
        warn!("A Player SpawnPoint exists, but no player entity was found");
        return;
    };

    transform.translation = translation;
    transform.rotation = Quat::from_rotation_y(yaw);
    player.yaw = yaw;
}
