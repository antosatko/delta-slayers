use bevy::{
    gltf::GltfAssetLabel,
    prelude::*,
    world_serialization::{WorldAssetRoot, WorldInstanceReady},
};

use crate::{
    app::state::AppState,
    gameplay::{
        persistence::load_game,
        player::Player,
        spawning::WorldStartup,
        zones::{FastTravelRequest, NearbyZone, VisitedZones},
    },
    ui::stack::UiStack,
};

pub(super) fn setup_game(
    mut commands: Commands,
    mut nearby: ResMut<NearbyZone>,
    mut request: ResMut<FastTravelRequest>,
    mut visited: ResMut<VisitedZones>,
    mut ui_state: ResMut<UiStack>,
    asset_server: Res<AssetServer>,
    mut startup: ResMut<WorldStartup>,
) {
    nearby.0 = None;
    ui_state.reset();
    request.0 = None;

    let (saved_pos, saved_yaw, saved_pitch, saved_visited) = load_game();
    startup.has_saved_position = saved_pos != Vec3::ZERO;
    startup.spawn_point_applied = startup.has_saved_position;
    startup.scene_ready = false;
    visited.0 = saved_visited;

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.85),
            illuminance: 12000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit::<AppState>(AppState::InGame),
    ));

    let spawn_pos = if saved_pos == Vec3::ZERO {
        Vec3::new(0.0, 1.7, 0.0)
    } else {
        Vec3::new(saved_pos.x, 1.7, saved_pos.z)
    };
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, spawn_pos.z)
            .with_rotation(Quat::from_euler(EulerRot::YXZ, saved_yaw, saved_pitch, 0.0)),
        Player {
            yaw: saved_yaw,
            pitch: saved_pitch,
        },
        DespawnOnExit::<AppState>(AppState::InGame),
    ));

    commands
        .spawn((
            WorldAssetRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("worlds/town-map.glb"),
                ),
            ),
            DespawnOnExit::<AppState>(AppState::InGame),
        ))
        .observe(world_instance_ready);
}

fn world_instance_ready(
    _event: On<WorldInstanceReady>,
    mut startup: ResMut<WorldStartup>,
) {
    startup.scene_ready = true;
}
