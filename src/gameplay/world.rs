use bevy::{
    asset::RenderAssetUsages,
    gltf::GltfAssetLabel,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    world_serialization::{WorldAssetRoot, WorldInstanceReady},
};

use crate::{
    app::state::AppState,
    gameplay::{
        billboard::Billboard,
        persistence::load_game,
        player::Player,
        spawning::{SpawnPoint, WorldStartup},
        zones::{FastTravelRequest, NearbyZone, TriggerZone, VisitedZones, ZoneKind},
    },
    ui::stack::UiStack,
};

pub(super) fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
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

    const WORLD_SCENE_PATH: &str = "worlds/town-map.glb";
    let has_blender_world = std::path::Path::new("assets/worlds/town-map.glb").is_file();
    startup.scene_ready = !has_blender_world;
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
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, spawn_pos.z).with_rotation(Quat::from_euler(
            EulerRot::YXZ,
            saved_yaw,
            saved_pitch,
            0.0,
        )),
        Player {
            yaw: saved_yaw,
            pitch: saved_pitch,
        },
        DespawnOnExit::<AppState>(AppState::InGame),
    ));

    if has_blender_world {
        commands
            .spawn((
                WorldAssetRoot(
                    asset_server.load(GltfAssetLabel::Scene(0).from_asset(WORLD_SCENE_PATH)),
                ),
                DespawnOnExit::<AppState>(AppState::InGame),
            ))
            .observe(world_instance_ready);
    } else {
        // Until a Blender export is present, keep a small procedural test level.
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

        // The fallback spawn point exercises the same spawn-point path as a
        // Blender-authored Empty.
        commands.spawn((
            Transform::default(),
            SpawnPoint::default(),
            DespawnOnExit::<AppState>(AppState::InGame),
        ));
    }
}

fn world_instance_ready(_event: On<WorldInstanceReady>, mut startup: ResMut<WorldStartup>) {
    startup.scene_ready = true;
}
