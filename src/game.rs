use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::{
    billboard::{self, Billboard},
    player::{self, Player},
    state::AppState,
    ui_stack::UiStack,
    zone::{self, FastTravelRequest, NearbyZone, TriggerZone, VisitedZones, ZoneKind},
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NearbyZone>()
            .init_resource::<VisitedZones>()
            .init_resource::<FastTravelRequest>()
            .add_systems(OnEnter(AppState::InGame), setup_game)
            .add_systems(
                Update,
                (
                    player::player_look.run_if(gameplay_input_enabled),
                    player::player_move.run_if(gameplay_input_enabled),
                    billboard::billboard_system,
                    zone::zone_detection_system.run_if(gameplay_input_enabled),
                )
                    .chain()
                    .after(crate::ui::UiSet::Input)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                fast_travel
                    .run_if(in_state(AppState::InGame))
                    .after(player::player_move),
            )
            .add_systems(
                Update,
                return_to_menu
                    .run_if(in_state(AppState::InGame))
                    .after(crate::ui::UiSet::Input),
            );
    }
}

fn gameplay_input_enabled(ui_state: Res<UiStack>) -> bool {
    !ui_state.paused() && !ui_state.inventory_open()
}

/// Creates world entities only. All UI construction belongs to the BSN scene
/// layer and is registered by UiPlugin.
fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut nearby: ResMut<NearbyZone>,
    mut request: ResMut<FastTravelRequest>,
    mut visited: ResMut<VisitedZones>,
    mut ui_state: ResMut<UiStack>,
) {
    nearby.0 = None;
    ui_state.reset();
    request.0 = None;

    let (saved_pos, saved_yaw, saved_pitch, saved_visited) = load_game();
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

    let ground_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.0)));
    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.72, 0.32),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });
    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_material),
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

    // Procedural placeholder texture used by the billboard entities.
    let mut image = Image::new_fill(
        Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    for y in 0..64u32 {
        for x in 0..64u32 {
            let radius = ((x as f32 - 32.0).powi(2) + (y as f32 - 32.0).powi(2)).sqrt();
            let bytes = image.pixel_bytes_mut(UVec3::new(x, y, 0)).unwrap();

            if radius < 28.0 {
                bytes[0] = 220;
                bytes[1] = 60;
                bytes[2] = 80;
                bytes[3] = 255;

                for (eye_x, eye_y) in [(22i32, 24i32), (42i32, 24i32)] {
                    if (x as i32 - eye_x).abs() <= 3 && (y as i32 - eye_y).abs() <= 4 {
                        bytes[0] = 255;
                        bytes[1] = 255;
                        bytes[2] = 255;
                    }
                }

                if (y as i32 - 38).abs() <= 2 && (x as i32 - 32).abs() <= 10 {
                    bytes[0] = 40;
                    bytes[1] = 20;
                    bytes[2] = 20;
                }
            } else {
                bytes[0] = 0;
                bytes[1] = 0;
                bytes[2] = 0;
                bytes[3] = 0;
            }
        }
    }

    let texture = images.add(image);
    let billboard_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(texture),
        cull_mode: None,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let billboard_mesh = meshes.add(Rectangle::new(1.0, 1.5));

    for position in [
        Vec3::new(3.0, 0.75, 0.0),
        Vec3::new(-2.0, 0.75, 4.0),
        Vec3::new(0.0, 0.75, -6.0),
        Vec3::new(5.0, 0.75, -3.0),
    ] {
        commands.spawn((
            Mesh3d(billboard_mesh.clone()),
            MeshMaterial3d(billboard_material.clone()),
            Transform::from_translation(position),
            Billboard,
            DespawnOnExit::<AppState>(AppState::InGame),
        ));
    }

    for (kind, position, radius, color) in [
        (
            ZoneKind::Shop,
            Vec3::new(8.0, 0.0, 0.0),
            2.5,
            Color::srgba(0.25, 0.65, 1.0, 0.45),
        ),
        (
            ZoneKind::Guild,
            Vec3::new(-8.0, 0.0, 0.0),
            2.5,
            Color::srgba(1.0, 0.75, 0.2, 0.45),
        ),
    ] {
        let zone_mesh = meshes.add(Cylinder::new(radius, 0.2));
        let zone_material = materials.add(StandardMaterial {
            base_color: color,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            unlit: true,
            ..default()
        });

        commands.spawn((
            Mesh3d(zone_mesh),
            MeshMaterial3d(zone_material),
            Transform::from_translation(position),
            TriggerZone { kind, radius },
            DespawnOnExit::<AppState>(AppState::InGame),
        ));
    }
}

fn fast_travel(
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

fn return_to_menu(
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

fn zone_kind_str(kind: ZoneKind) -> &'static str {
    match kind {
        ZoneKind::Shop => "Shop",
        ZoneKind::Guild => "Guild",
    }
}

fn parse_zone_kind(value: &str) -> Option<ZoneKind> {
    match value {
        "Shop" => Some(ZoneKind::Shop),
        "Guild" => Some(ZoneKind::Guild),
        _ => None,
    }
}

fn save_game(
    position: Vec3,
    yaw: f32,
    pitch: f32,
    visited: &std::collections::HashSet<ZoneKind>,
) {
    let visited_zones: Vec<&str> = visited.iter().map(|kind| zone_kind_str(*kind)).collect();
    let content = format!(
        "pos {} {} {}\nyaw {}\npitch {}\nvisited {}\n",
        position.x,
        position.y,
        position.z,
        yaw,
        pitch,
        visited_zones.join(",")
    );
    let _ = std::fs::write("save.txt", content);
}

fn load_game() -> (Vec3, f32, f32, std::collections::HashSet<ZoneKind>) {
    let mut position = Vec3::ZERO;
    let mut yaw = 0.0;
    let mut pitch = 0.0;
    let mut visited = std::collections::HashSet::new();

    if let Ok(content) = std::fs::read_to_string("save.txt") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "pos" if parts.len() >= 4 => {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>(),
                    ) {
                        position = Vec3::new(x, y, z);
                    }
                }
                "yaw" if parts.len() >= 2 => {
                    if let Ok(value) = parts[1].parse::<f32>() {
                        yaw = value;
                    }
                }
                "pitch" if parts.len() >= 2 => {
                    if let Ok(value) = parts[1].parse::<f32>() {
                        pitch = value;
                    }
                }
                "visited" if parts.len() >= 2 => {
                    for token in parts[1].split(',') {
                        if let Some(kind) = parse_zone_kind(token) {
                            visited.insert(kind);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    (position, yaw, pitch, visited)
}
