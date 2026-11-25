use bevy::{
    gltf::GltfAssetLabel,
    prelude::*,
    world_serialization::{WorldAssetRoot, WorldInstanceReady},
};

use crate::{
    app::state::AppState,
    gameplay::{
        billboard::Billboard,
        persistence::load_game,
        player::{self, GameplayCamera, Player, PlayerVisual},
        spawning::WorldStartup,
        zones::{FastTravelRequest, NearbyZone, VisitedZones},
    },
    ui::stack::UiStack,
};

pub(super) fn setup_game(
    mut commands: Commands,
    start_mode: Res<super::GameStartMode>,
    mut nearby: ResMut<NearbyZone>,
    mut request: ResMut<FastTravelRequest>,
    mut visited: ResMut<VisitedZones>,
    mut ui_state: ResMut<UiStack>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut startup: ResMut<WorldStartup>,
    mut movement_target: ResMut<player::MovementTarget>,
) {
    nearby.0 = None;
    movement_target.clear();
    ui_state.reset();
    request.0 = None;

    let (saved_pos, saved_yaw, saved_pitch, saved_visited) = match *start_mode {
        super::GameStartMode::QuickLoad => load_game(),
        super::GameStartMode::NewGame => (
            Vec3::ZERO,
            0.0,
            player::CAMERA_DEFAULT_PITCH,
            std::collections::HashSet::new(),
        ),
    };
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

    // Preserve the full saved world position, including its elevation.
    let spawn_pos = saved_pos;
    let initial_pitch = if startup.has_saved_position {
        player::migrate_saved_camera_pitch(saved_pitch)
    } else {
        player::CAMERA_DEFAULT_PITCH
    };
    let player_rotation = Quat::from_rotation_y(saved_yaw);
    let camera_rotation = player::follow_camera_rotation(saved_yaw, initial_pitch);
    let camera_target = spawn_pos + Vec3::Y * 0.8;
    let camera_position =
        camera_target - camera_rotation * Vec3::NEG_Z * player::CAMERA_DISTANCE;

    // The player model owns the world-space ground position. The camera and
    // billboard are separate entities so neither affects gameplay queries.
    commands.spawn((
        Transform::from_translation(spawn_pos).with_rotation(player_rotation),
        Player {
            yaw: saved_yaw,
            pitch: initial_pitch,
        },
        DespawnOnExit::<AppState>(AppState::InGame),
    ));

    commands.spawn((
        Camera3d::default(),
        GameplayCamera {
            distance: player::CAMERA_DISTANCE,
        },
        Transform::from_translation(camera_position).looking_at(camera_target, Vec3::Y),
        DespawnOnExit::<AppState>(AppState::InGame),
    ));

    let player_mesh = meshes.add(Rectangle::new(0.95, 1.68));
    let player_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("character/portrait.png")),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(player_mesh),
        MeshMaterial3d(player_material),
        Transform::from_translation(spawn_pos + Vec3::Y * 0.8),
        PlayerVisual,
        Billboard,
        DespawnOnExit::<AppState>(AppState::InGame),
    ));

    commands
        .spawn((
            WorldAssetRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset("worlds/town-map.glb")),
            ),
            DespawnOnExit::<AppState>(AppState::InGame),
        ))
        .observe(world_instance_ready);
}

fn world_instance_ready(
    _event: On<WorldInstanceReady>,
    mut startup: ResMut<WorldStartup>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    startup.scene_ready = true;
    info!("World scene is ready; entering gameplay");
    next_state.set(AppState::InGame);
}
