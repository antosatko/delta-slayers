use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    ui_widgets::{Activate, Button},
    window::{CursorGrabMode, CursorOptions},
};

use crate::{
    billboard::Billboard,
    inventory::{self, InventoryRoot},
    player::Player,
    state::{AppState, CursorGrabbed},
    ui::{self, ConfirmPopupButton, FastTravelButton, PanelRoot, PopupRoot},
    ui_stack::{UiLayer, UiStack},
    zone::{
        self, FastTravelRequest, NearbyZone, TriggerZone, VisitedZones, ZoneKind,
    },
};

#[derive(Component)]
struct PauseOverlay;

#[derive(Resource, Default)]
struct InventoryMenuOpen(pub bool);

#[derive(Component)]
struct InventoryMenuModal;

#[derive(Component)]
struct FastTravelMenu;

#[derive(Resource, Default)]
struct PopupConfirmRequested(pub bool);

#[derive(Component)]
struct MinimapPlayerDot;

#[derive(Component)]
struct MinimapZoneDot(ZoneKind);

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NearbyZone::default())
            .insert_resource(VisitedZones::default())
            .insert_resource(FastTravelRequest::default())
            .insert_resource(CursorGrabbed::default())
            .insert_resource(PopupConfirmRequested::default())
            .insert_resource(UiStack::default())
            .insert_resource(InventoryMenuOpen::default())
            .add_systems(OnEnter(AppState::InGame), setup_game)
            .add_systems(
                Update,
                (
                    crate::player::player_look.run_if(not_paused),
                    crate::player::player_move.run_if(not_paused),
                    crate::billboard::billboard_system,
                    zone::zone_detection_system.run_if(not_paused),
                    ui::panel_visibility_system,
                    ui::popup_visibility_system,
                    ui::fast_travel_button_visibility_system,
                    pause_overlay_system,
                    inventory::inventory_visibility_system,
                    sync_fast_travel_menu,
                    inventory_menu_visibility_system,
                    sync_cursor,
                    update_minimap,
                    handle_input,
                    return_to_menu,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                fast_travel.run_if(in_state(AppState::InGame)).after(crate::player::player_look).after(crate::player::player_move),
            );
    }
}

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut nearby: ResMut<NearbyZone>,
    mut request: ResMut<FastTravelRequest>,
    mut visited: ResMut<VisitedZones>,
    mut ui_stack: ResMut<UiStack>,
    mut inventory_menu_open: ResMut<InventoryMenuOpen>,
) {
    nearby.0 = None;
    request.0 = None;
    ui_stack.clear();
    inventory_menu_open.0 = false;

    let (saved_pos, saved_yaw, saved_pitch, saved_visited) = load_game();
    visited.0 = saved_visited;

    let _ = asset_server;

    // Lighting
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

    // Ground
    let ground_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.0)));
    let ground_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.72, 0.32),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });
    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_mat),
        DespawnOnExit::<AppState>(AppState::InGame),
    ));

    // Player / camera
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

    // UI camera (overlays)
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        DespawnOnExit::<AppState>(AppState::InGame),
    ));

    // Procedural 2D sprite texture for billboards
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
            let cx = 32.0f32;
            let cy = 32.0f32;
            let r = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
            let bytes = image.pixel_bytes_mut(UVec3::new(x, y, 0)).unwrap();
            if r < 28.0 {
                bytes[0] = 220;
                bytes[1] = 60;
                bytes[2] = 80;
                bytes[3] = 255;
                let (ex1, ey1) = (22i32, 24i32);
                let (ex2, ey2) = (42i32, 24i32);
                for (ex, ey) in [(ex1, ey1), (ex2, ey2)] {
                    if (x as i32 - ex).abs() <= 3 && (y as i32 - ey).abs() <= 4 {
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

    let billboard_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(texture),
        cull_mode: None,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let billboard_mesh = meshes.add(Rectangle::new(1.0, 1.5));

    let positions = [
        Vec3::new(3.0, 0.75, 0.0),
        Vec3::new(-2.0, 0.75, 4.0),
        Vec3::new(0.0, 0.75, -6.0),
        Vec3::new(5.0, 0.75, -3.0),
    ];
    for pos in positions {
        commands.spawn((
            Mesh3d(billboard_mesh.clone()),
            MeshMaterial3d(billboard_mat.clone()),
            Transform::from_translation(pos),
            Billboard,
            DespawnOnExit::<AppState>(AppState::InGame),
        ));
    }

    // Trigger zones on the map
    let zones = [
        (ZoneKind::Shop, Vec3::new(8.0, 0.0, 0.0), 2.5, Color::srgba(0.25, 0.65, 1.0, 0.45)),
        (ZoneKind::Guild, Vec3::new(-8.0, 0.0, 0.0), 2.5, Color::srgba(1.0, 0.75, 0.2, 0.45)),
    ];
    for (kind, pos, radius, color) in zones {
        let zone_mesh = meshes.add(Cylinder::new(radius, 0.2));
        let zone_mat = materials.add(StandardMaterial {
            base_color: color,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            unlit: true,
            ..default()
        });
        commands.spawn((
            Mesh3d(zone_mesh),
            MeshMaterial3d(zone_mat),
            Transform::from_translation(pos),
            TriggerZone { kind, radius },
            DespawnOnExit::<AppState>(AppState::InGame),
        ));
    }

    // UI panels (hidden until confirmed)
    let panels = [
        (ZoneKind::Shop, "Shop", Color::srgba(0.1, 0.1, 0.15, 0.92)),
        (ZoneKind::Guild, "Guild", Color::srgba(0.15, 0.12, 0.08, 0.92)),
    ];
    for (kind, title, color) in panels {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: px(80),
                    left: px(80),
                    width: px(340),
                    height: px(240),
                    padding: UiRect::all(px(16)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    border_radius: BorderRadius::all(px(16)),
                    ..default()
                },
                BackgroundColor(color),
                PanelRoot(kind),
                Visibility::Hidden,
                DespawnOnExit::<AppState>(AppState::InGame),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(title),
                    TextFont {
                        font_size: px(28).into(),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                parent.spawn((
                    Text::new("Walk out of the zone or press Esc to close."),
                    TextFont {
                        font_size: px(14).into(),
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));
            });
    }

    // Popup prompt when entering a zone
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(120),
                left: percent(50),
                margin: UiRect::left(px(-140)),
                width: px(280),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(8),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            PopupRoot,
            Visibility::Hidden,
            DespawnOnExit::<AppState>(AppState::InGame),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Press E to open"),
                TextFont {
                    font_size: px(18).into(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(px(12), px(6)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Open"),
                        TextFont {
                            font_size: px(16).into(),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                })
                .insert(ConfirmPopupButton)
                .observe(|_a: On<Activate>, mut req: ResMut<PopupConfirmRequested>| {
                    req.0 = true;
                });
        });

    // Fast travel menu (top-right), visible only with inventory
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(16),
                right: px(16),
                width: px(200),
                padding: UiRect::all(px(10)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            FastTravelMenu,
            Visibility::Hidden,
            DespawnOnExit::<AppState>(AppState::InGame),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Fast Travel"),
                TextFont {
                    font_size: px(16).into(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            for kind in [ZoneKind::Shop, ZoneKind::Guild] {
                let label = match kind {
                    ZoneKind::Shop => "Shop",
                    ZoneKind::Guild => "Guild",
                };
                parent
                    .spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(px(8), px(4)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
                        Visibility::Hidden,
                    ))
                    .with_children(|p| {
                        p.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: px(14).into(),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    })
                    .insert(FastTravelButton(kind))
                    .observe(move |_a: On<Activate>, mut req: ResMut<FastTravelRequest>| {
                        req.0 = Some(kind);
                    });
            }
        });

    // Minimap (bottom-right)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(16),
                right: px(16),
                width: px(120),
                height: px(120),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            DespawnOnExit::<AppState>(AppState::InGame),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(56),
                    top: px(56),
                    width: px(8),
                    height: px(8),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                MinimapPlayerDot,
            ));
            for kind in [ZoneKind::Shop, ZoneKind::Guild] {
                let color = match kind {
                    ZoneKind::Shop => Color::srgb(0.2, 0.6, 0.9),
                    ZoneKind::Guild => Color::srgb(0.9, 0.7, 0.2),
                };
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        top: px(0),
                        width: px(6),
                        height: px(6),
                        ..default()
                    },
                    BackgroundColor(color),
                    Visibility::Hidden,
                    MinimapZoneDot(kind),
                ));
            }
        });

    // Inventory (hidden until opened)
    let portrait = asset_server.load("character/portrait.png");
    let item_images = [
        asset_server.load("items/evil-pup.png"),
        asset_server.load("items/maurice.webp"),
        asset_server.load("items/salad.jpg"),
        asset_server.load("items/scooby.webp"),
        asset_server.load("items/theo.jpg"),
    ];
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(40),
                left: px(40),
                width: px(820),
                height: px(600),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Row,
                column_gap: px(12),
                border_radius: BorderRadius::all(px(16)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.95)),
            InventoryRoot,
            Visibility::Hidden,
            DespawnOnExit::<AppState>(AppState::InGame),
        ))
        .with_children(|parent| {
            // Top bar
            parent
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    top: px(0),
                    left: px(0),
                    width: percent(100),
                    height: px(32),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    padding: UiRect::horizontal(px(8)),
                    ..default()
                })
                .with_children(|bar| {
                    bar.spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(px(12), px(6)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.35)),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("Menu"),
                            TextFont {
                                font_size: px(14).into(),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    })
                    .observe(|_a: On<Activate>, mut open: ResMut<InventoryMenuOpen>, mut modals: Query<&mut Visibility, With<InventoryMenuModal>>| {
                        open.0 = !open.0;
                        for mut visibility in &mut modals {
                            *visibility = if open.0 {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            };
                        }
                    });
                    // Modal menu
                    bar.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(0),
                            left: px(0),
                            width: px(820),
                            height: px(600),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                        bevy::ui::GlobalZIndex(10),
                        bevy::picking::Pickable::IGNORE,
                        InventoryMenuModal,
                        Visibility::Hidden,
                    ))
                    .with_children(|modal| {
                        modal
                            .spawn((
                                Node {
                                    width: px(260),
                                    padding: UiRect::all(px(16)),
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(8),
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(px(16)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                            ))
                            .with_children(|panel| {
                                panel.spawn((
                                    Text::new("Menu"),
                                    TextFont {
                                        font_size: px(24).into(),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                                for (label, action) in [
                                    ("Continue", 0u8),
                                    ("Exit to Main", 1u8),
                                    ("Exit to Desktop", 2u8),
                                    ("Cancel", 3u8),
                                ] {
                                    panel
                                        .spawn((
                                            Button,
                                            Node {
                                                width: px(200),
                                                padding: UiRect::axes(px(12), px(8)),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border_radius: BorderRadius::all(px(8)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                                        ))
                                        .with_children(|p| {
                                            p.spawn((
                                                Text::new(label),
                                                TextFont {
                                                    font_size: px(16).into(),
                                                    ..default()
                                                },
                                                TextColor(Color::WHITE),
                                            ));
                                        })
                                        .observe(move |_a: On<Activate>, mut open: ResMut<InventoryMenuOpen>, mut next: ResMut<NextState<AppState>>, mut exit: MessageWriter<AppExit>| {
                                            open.0 = false;
                                            match action {
                                                0 => {}
                                                1 => {
                                                    next.set(AppState::MainMenu);
                                                }
                                                2 => {
                                                    exit.write(AppExit::Success);
                                                }
                                                _ => {}
                                            }
                                        });
                                }
                            });
                    });
                });

            // Character preview + gear
            parent
                .spawn(Node {
                    position_type: PositionType::Relative,
                    width: px(360),
                    height: px(560),
                    ..default()
                })
                .with_children(|area| {
                    area.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(0),
                            left: px(0),
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        ImageNode::new(portrait.clone()),
                    ));
                    area.spawn(Node {
                        position_type: PositionType::Absolute,
                        top: px(8),
                        left: px(8),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|col| {
                        spawn_gear_slot(col, "Head");
                        spawn_gear_slot(col, "Neck");
                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(8),
                            ..default()
                        })
                        .with_children(|r| {
                            spawn_gear_slot(r, "L Shldr");
                            spawn_gear_slot(r, "Torso");
                            spawn_gear_slot(r, "R Shldr");
                        });
                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(8),
                            ..default()
                        })
                        .with_children(|r| {
                            spawn_gear_slot(r, "L Hand");
                            spawn_gear_slot(r, "R Hand");
                        });
                        spawn_gear_slot(col, "Legs");
                        spawn_gear_slot(col, "Feet");
                    });
                });

            // Items + resources card
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8),
                        flex_grow: 1.0,
                        padding: UiRect::all(px(8)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.12, 0.16, 0.9)),
                ))
                .with_children(|col| {
                    col.spawn(Node {
                        flex_wrap: FlexWrap::Wrap,
                        row_gap: px(4),
                        column_gap: px(4),
                        ..default()
                    })
                    .with_children(|grid| {
                        for i in 0..12 {
                            grid.spawn((
                                Node {
                                    width: px(64),
                                    height: px(32),
                                    padding: UiRect::all(px(2)),
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(2),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.18, 0.18, 0.22)),
                            ))
                            .with_children(|slot| {
                                if i < item_images.len() {
                                    slot.spawn((
                                        Node {
                                            width: px(32),
                                            height: px(32),
                                            ..default()
                                        },
                                        ImageNode::new(item_images[i].clone()),
                                    ));
                                } else {
                                    slot.spawn((
                                        Node {
                                            width: px(32),
                                            height: px(32),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 0.6)),
                                    ));
                                }
                                slot.spawn((
                                    Text::new("A12 D5"),
                                    TextFont {
                                        font_size: px(8).into(),
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.85, 0.85, 0.85)),
                                ));
                            });
                        }
                    });
                    col.spawn((
                        Node {
                            padding: UiRect::all(px(8)),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(2),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                    ))
                    .with_children(|r| {
                        r.spawn((
                            Text::new("Resources"),
                            TextFont {
                                font_size: px(14).into(),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                        for res in ["Gold: 0", "Wood: 0", "Stone: 0"] {
                            r.spawn((
                                Text::new(res),
                                TextFont {
                                    font_size: px(12).into(),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                            ));
                        }
                    });
                });
        });

    /* DISABLED REST

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(40),
                left: px(40),
                width: px(820),
                height: px(600),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.95)),
            InventoryRoot,
            Visibility::Hidden,
            DespawnOnExit::<AppState>(AppState::InGame),
        ))
        .with_children(|parent| {
            // Top bar
            parent
                .spawn(Node {
                    width: percent(100),
                    height: px(32),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    ..default()
                })
                .with_children(|bar| {
                    bar.spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(px(12), px(6)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.35)),
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("Menu"),
                            TextFont {
                                font_size: px(14).into(),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    })
                    .observe(|_a: On<Activate>, mut open: ResMut<InventoryMenuOpen>, mut modals: Query<&mut Visibility, With<InventoryMenuModal>>| {
                        open.0 = !open.0;
                        for mut visibility in &mut modals {
                            *visibility = if open.0 {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            };
                        }
                    });
                    // Modal menu
                    bar.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(0),
                            left: px(0),
                            width: px(820),
                            height: px(600),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                        bevy::ui::GlobalZIndex(10),
                        bevy::picking::Pickable::IGNORE,
                        InventoryMenuModal,
                        Visibility::Hidden,
                    ))
                    .with_children(|dd| {
                        for (label, action) in [
                            ("Continue", 0u8),
                            ("Exit to Main", 1u8),
                            ("Exit to Desktop", 2u8),
                        ] {
                            dd.spawn((
                                Button,
                                Node {
                                    padding: UiRect::axes(px(8), px(4)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
                            ))
                            .with_children(|p| {
                                p.spawn((
                                    Text::new(label),
                                    TextFont {
                                        font_size: px(13).into(),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            })
                            .observe(move |_a: On<Activate>, mut open: ResMut<InventoryMenuOpen>, mut next: ResMut<NextState<AppState>>, mut exit: MessageWriter<AppExit>| {
                                open.0 = false;
                                match action {
                                    0 => {}
                                    1 => {
                                        next.set(AppState::MainMenu);
                                    }
                                    2 => {
                                        exit.write(AppExit::Success);
                                    }
                                    _ => {}
                                }
                            });
                        }
                    });
                });

            // Content row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(12),
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|row| {
                    // Character preview with anatomical gear slots
                    row
                .spawn(Node {
                    position_type: PositionType::Relative,
                    width: px(360),
                    height: px(560),
                    ..default()
                })
                .with_children(|area| {
                    // Character portrait
                    area.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(0),
                            left: px(0),
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        ImageNode::new(portrait.clone()),
                    ));
                    // Gear slots (absolute, anatomical)
                    // Gear slots (anatomical flex layout)
                    area.spawn(Node {
                        position_type: PositionType::Absolute,
                        top: px(8),
                        left: px(8),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|col| {
                        spawn_gear_slot(col, "Head");
                        spawn_gear_slot(col, "Neck");
                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(8),
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_gear_slot(row, "L Shldr");
                            spawn_gear_slot(row, "Torso");
                            spawn_gear_slot(row, "R Shldr");
                        });
                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(8),
                            ..default()
                        })
                        .with_children(|row| {
                            spawn_gear_slot(row, "L Hand");
                            spawn_gear_slot(row, "R Hand");
                        });
                        spawn_gear_slot(col, "Legs");
                        spawn_gear_slot(col, "Feet");
                    });
                });

            // Items + resources card
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(8),
                        flex_grow: 1.0,
                        padding: UiRect::all(px(8)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.12, 0.16, 0.9)),
                ))
                .with_children(|col| {
                    // Item grid
                    col.spawn(Node {
                        flex_wrap: FlexWrap::Wrap,
                        row_gap: px(4),
                        column_gap: px(4),
                        ..default()
                    })
                    .with_children(|grid| {
                        for i in 0..12 {
                            grid.spawn((
                                Node {
                                    width: px(64),
                                    height: px(32),
                                    padding: UiRect::all(px(2)),
                                    flex_direction: FlexDirection::Row,
                                    column_gap: px(2),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.18, 0.18, 0.22)),
                            ))
                            .with_children(|slot| {
                                // Single icon
                                if i < item_images.len() {
                                    slot.spawn((
                                        Node {
                                            width: px(32),
                                            height: px(32),
                                            ..default()
                                        },
                                        ImageNode::new(item_images[i].clone()),
                                    ));
                                } else {
                                    slot.spawn((
                                        Node {
                                            width: px(32),
                                            height: px(32),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 0.6)),
                                    ));
                                }
                                // Stats (compact, inside slot)
                                slot.spawn((
                                    Text::new("A12 D5"),
                                    TextFont {
                                        font_size: px(8).into(),
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.85, 0.85, 0.85)),
                                ));
                            });
                        }
                    });
                    // Resources tab
                    col.spawn((
                        Node {
                            padding: UiRect::all(px(8)),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(2),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.12, 0.16)),
                    ))
                    .with_children(|r| {
                        r.spawn((
                            Text::new("Resources"),
                            TextFont {
                                font_size: px(14).into(),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                        for res in ["Gold: 0", "Wood: 0", "Stone: 0"] {
                            r.spawn((
                                Text::new(res),
                                TextFont {
                                    font_size: px(12).into(),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                            ));
                        }
                    });


                });
        });

    */

    // Pause overlay
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            PauseOverlay,
            Visibility::Hidden,
            DespawnOnExit::<AppState>(AppState::InGame),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED  (P to resume)"),
                TextFont {
                    font_size: px(48).into(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    nearby: Res<NearbyZone>,
    mut ui_stack: ResMut<UiStack>,
    mut cursor: Single<&mut CursorOptions>,
    mut cursor_grabbed: ResMut<CursorGrabbed>,
    mut confirm_req: ResMut<PopupConfirmRequested>,
    mut inventory_menu_open: ResMut<InventoryMenuOpen>,
    mut ft_menus: Query<&mut Visibility, With<FastTravelMenu>>,
) {
    // Confirm popup -> open panel
    let mut confirm = keys.just_pressed(KeyCode::KeyE) || confirm_req.0;
    if confirm_req.0 {
        confirm_req.0 = false;
    }
    if confirm {
        if let Some(kind) = nearby.0 {
            ui_stack.push(UiLayer::Panel(kind));
        }
    }

    // Right click: open inventory while controlling character, otherwise
    // pop the topmost UI element.
    if mouse.just_pressed(MouseButton::Right) {
        if cursor_grabbed.0 {
            ui_stack.push(UiLayer::Inventory);
        } else {
            if ui_stack.inventory_open() {
                inventory_menu_open.0 = false;
            }
            ui_stack.pop_top();
        }
    }

    // Pause toggle
    if keys.just_pressed(KeyCode::KeyP) {
        if ui_stack.paused() {
            // remove pause
            ui_stack.0.retain(|l| *l != UiLayer::Pause);
        } else {
            ui_stack.push(UiLayer::Pause);
        }
    }

    // Esc pops top UI or releases cursor
    if keys.just_pressed(KeyCode::Escape) {
        if !ui_stack.0.is_empty() {
            if ui_stack.inventory_open() {
                inventory_menu_open.0 = false;
            }
            ui_stack.pop_top();
        } else {
            cursor.visible = true;
            cursor.grab_mode = CursorGrabMode::None;
        }
    }

    // Walking out of a zone (or into a different one) closes its panel.
    if let Some(active) = ui_stack.active_panel() {
        if nearby.0 != Some(active) {
            ui_stack.0.retain(|l| !matches!(l, UiLayer::Panel(_)));
            cursor.visible = false;
            cursor.grab_mode = CursorGrabMode::Locked;
        }
    }

    // Keep fast travel menu in sync with inventory visibility.
    for mut v in &mut ft_menus {
        *v = if ui_stack.inventory_open() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Cursor visibility follows UI stack.
    if ui_stack.0.is_empty() {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor_grabbed.0 = true;
    } else {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
        cursor_grabbed.0 = false;
    }

    cursor_grabbed.0 = cursor.grab_mode == CursorGrabMode::Locked;
}

fn fast_travel(
    mut request: ResMut<FastTravelRequest>,
    mut queries: ParamSet<(
        Query<&mut Transform, With<Player>>,
        Query<(&Transform, &TriggerZone)>,
    )>,
) {
    if let Some(kind) = request.0.take() {
        let zone_pos = {
            let zones = queries.p1();
            let mut found = None;
            for (zone_transform, zone) in &zones {
                if zone.kind == kind {
                    found = Some(zone_transform.translation);
                    break;
                }
            }
            found
        };
        if let Some(pos) = zone_pos {
            let mut players = queries.p0();
            if let Ok(mut player_transform) = players.single_mut() {
                player_transform.translation = pos + Vec3::new(0.0, 1.7, 2.0);
            }
        }
    }
}

fn update_minimap(
    player_q: Query<&Transform, With<Player>>,
    zones: Query<(&Transform, &TriggerZone)>,
    mut queries: ParamSet<(
        Query<&mut Node, With<MinimapPlayerDot>>,
        Query<(&MinimapZoneDot, &mut Node, &mut Visibility)>,
    )>,
) {
    let Ok(player_transform) = player_q.single() else {
        return;
    };
    let player_pos = player_transform.translation;
    let _ = &mut queries.p0(); // touch player dots (keep centered)
    let scale = 4.0;
    let zone_data: Vec<(ZoneKind, Vec3)> = zones
        .iter()
        .map(|(zt, zone)| (zone.kind, zt.translation))
        .collect();
    let mut zone_dots = queries.p1();
    for (dot, mut node, mut vis) in &mut zone_dots {
        let mut found = None;
        for (kind, pos) in &zone_data {
            if *kind == dot.0 {
                found = Some(*pos);
                break;
            }
        }
        if let Some(zpos) = found {
            let rel = zpos - player_pos;
            let x = 60.0 + rel.x * scale - 3.0;
            let y = 60.0 - rel.z * scale - 3.0;
            if x < 0.0 || x > 114.0 || y < 0.0 || y > 114.0 {
                *vis = Visibility::Hidden;
            } else {
                *vis = Visibility::Visible;
                node.left = px(x);
                node.top = px(y);
            }
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn sync_cursor(
    ui_stack: Res<UiStack>,
    mut cursor: Single<&mut CursorOptions>,
    mut cursor_grabbed: ResMut<CursorGrabbed>,
) {
    if ui_stack.0.is_empty() {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor_grabbed.0 = true;
    } else {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
        cursor_grabbed.0 = false;
    }
}

fn spawn_gear_slot(parent: &mut ChildSpawnerCommands, label: &'static str) {
    parent
        .spawn((
            Node {
                width: px(72),
                height: px(36),
                padding: UiRect::all(px(4)),
                flex_direction: FlexDirection::Row,
                column_gap: px(4),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.25, 0.25, 0.3, 0.85)),
        ))
        .with_children(|s| {
            s.spawn((
                Node {
                    width: px(28),
                    height: px(28),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.4, 0.4, 0.45, 0.8)),
            ));
            s.spawn((
                Text::new(label),
                TextFont {
                    font_size: px(10).into(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn sync_fast_travel_menu(
    ui_stack: Res<UiStack>,
    mut menus: Query<&mut Visibility, With<FastTravelMenu>>,
) {
    for mut visibility in &mut menus {
        *visibility = if ui_stack.inventory_open() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn inventory_menu_visibility_system(
    open: Res<InventoryMenuOpen>,
    mut dropdowns: Query<&mut Visibility, With<InventoryMenuModal>>,
) {
    for mut visibility in &mut dropdowns {
        *visibility = if open.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn not_paused(ui_stack: Res<UiStack>) -> bool {
    !ui_stack.paused()
}

fn pause_overlay_system(ui_stack: Res<UiStack>, mut overlays: Query<&mut Visibility, With<PauseOverlay>>) {
    for mut visibility in &mut overlays {
        *visibility = if ui_stack.paused() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn return_to_menu(
    key: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ui_stack: ResMut<UiStack>,
    player_q: Query<(&Transform, &Player)>,
    visited: Res<VisitedZones>,
) {
    if key.just_pressed(KeyCode::Backspace) {
        if !ui_stack.0.is_empty() {
            ui_stack.pop_top();
        } else {
            if let Ok((transform, player)) = player_q.single() {
                save_game(transform.translation, player.yaw, player.pitch, &visited.0);
            }
            next_state.set(AppState::MainMenu);
        }
    }
}

fn zone_kind_str(kind: ZoneKind) -> &'static str {
    match kind {
        ZoneKind::Shop => "Shop",
        ZoneKind::Guild => "Guild",
    }
}

fn parse_zone_kind(s: &str) -> Option<ZoneKind> {
    match s {
        "Shop" => Some(ZoneKind::Shop),
        "Guild" => Some(ZoneKind::Guild),
        _ => None,
    }
}

fn save_game(pos: Vec3, yaw: f32, pitch: f32, visited: &std::collections::HashSet<ZoneKind>) {
    use std::io::Write;
    let visited_str: Vec<&str> = visited.iter().map(|k| zone_kind_str(*k)).collect();
    let content = format!(
        "pos {} {} {}\nyaw {}\npitch {}\nvisited {}\n",
        pos.x,
        pos.y,
        pos.z,
        yaw,
        pitch,
        visited_str.join(",")
    );
    let _ = std::fs::write("save.txt", content);
}

fn load_game() -> (Vec3, f32, f32, std::collections::HashSet<ZoneKind>) {
    let mut pos = Vec3::ZERO;
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
                        pos = Vec3::new(x, y, z);
                    }
                }
                "yaw" if parts.len() >= 2 => {
                    if let Ok(v) = parts[1].parse::<f32>() {
                        yaw = v;
                    }
                }
                "pitch" if parts.len() >= 2 => {
                    if let Ok(v) = parts[1].parse::<f32>() {
                        pitch = v;
                    }
                }
                "visited" if parts.len() >= 2 => {
                    for token in parts[1].split(',') {
                        if let Some(k) = parse_zone_kind(token) {
                            visited.insert(k);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    (pos, yaw, pitch, visited)
}