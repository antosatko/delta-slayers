use bevy::{
    prelude::*,
    ui_widgets::{Activate, Button},
    window::{CursorGrabMode, CursorOptions},
};

use crate::{
    inventory::{self, CurrencyLabel, InventoryRoot},
    state::AppState,
    ui_stack::{UiLayer, UiStack},
    zone::{FastTravelRequest, NearbyZone, VisitedZones, ZoneKind},
};

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiSet {
    Input,
    Projection,
}

#[derive(Component, Default, Clone)]
pub struct PanelRoot(pub ZoneKind);
#[derive(Component, Default, Clone)]
pub struct PopupRoot;
#[derive(Component, Default, Clone)]
pub struct FastTravelButton(pub ZoneKind);
#[derive(Component, Default, Clone)]
pub struct FastTravelMenu;
#[derive(Component, Default, Clone)]
pub struct InventoryMenuModal;
#[derive(Component, Default, Clone)]
pub struct PauseOverlay;
#[derive(Component, Default, Clone)]
pub struct MinimapZoneDot(pub ZoneKind);

#[derive(Resource, Default)]
struct PopupConfirmRequested(bool);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiStack>()
            .init_resource::<inventory::InventoryModel>()
            .init_resource::<inventory::InventoryViewModel>()
            .init_resource::<PopupConfirmRequested>()
            .configure_sets(Update, (UiSet::Input, UiSet::Projection).chain())
            .add_systems(OnEnter(AppState::InGame), game_ui_scene().spawn())
            .add_systems(
                Update,
                handle_ui_input
                    .in_set(UiSet::Input)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (
                    panel_visibility_system,
                    popup_visibility_system,
                    fast_travel_menu_visibility_system,
                    fast_travel_button_visibility_system,
                    inventory_visibility_system,
                    inventory_menu_visibility_system,
                    pause_overlay_visibility_system,
                    sync_cursor,
                    update_minimap,
                    inventory::sync_inventory_view_model,
                )
                    .chain()
                    .in_set(UiSet::Projection)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

/// All in-game UI is declared as BSN scenes. Gameplay setup must not spawn UI.
fn game_ui_scene() -> impl SceneList {
    bsn_list! {
        Camera2d
        Camera {
            order: 1,
        }
        DespawnOnExit::<AppState>(AppState::InGame)
        --
        @shop_panel()
        --
        @guild_panel()
        --
        @interaction_prompt()
        --
        @fast_travel_menu()
        --
        @minimap()
        --
        @inventory_scene()
        --
        @pause_overlay()
    }
}

fn panel(kind: ZoneKind, title: &'static str, color: Color) -> impl Scene {
    bsn! {
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
        }
        BackgroundColor(color)
        PanelRoot(kind)
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Text(title)
            TextFont {
                font_size: px(28),
            }
            TextColor(Color::WHITE)
            --
            Text("Walk out of the zone or press Esc to close.")
            TextFont {
                font_size: px(14),
            }
            TextColor(Color::srgb(0.8, 0.8, 0.8))
        ]
    }
}

fn shop_panel() -> impl Scene {
    panel(ZoneKind::Shop, "Shop", Color::srgba(0.1, 0.1, 0.15, 0.92))
}

fn guild_panel() -> impl Scene {
    panel(ZoneKind::Guild, "Guild", Color::srgba(0.15, 0.12, 0.08, 0.92))
}

fn interaction_prompt() -> impl Scene {
    bsn! {
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
        }
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8))
        PopupRoot
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Text("Press E to open")
            TextFont {
                font_size: px(18),
            }
            TextColor(Color::WHITE)
            --
            Button
            Node {
                padding: UiRect::axes(px(12), px(6)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
            BackgroundColor(Color::srgb(0.3, 0.3, 0.3))
            on(|_activate: On<Activate>, mut request: ResMut<PopupConfirmRequested>| {
                request.0 = true;
            })
            Children [
                Text("Open")
                TextFont {
                    font_size: px(16),
                }
                TextColor(Color::WHITE)
            ]
        ]
    }
}

fn fast_travel_menu() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            right: px(16),
            width: px(200),
            padding: UiRect::all(px(10)),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            border_radius: BorderRadius::all(px(12)),
        }
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75))
        FastTravelMenu
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Text("Fast Travel")
            TextFont {
                font_size: px(16),
            }
            TextColor(Color::WHITE)
            --
            @fast_travel_button(ZoneKind::Shop, "Shop")
            --
            @fast_travel_button(ZoneKind::Guild, "Guild")
        ]
    }
}

fn fast_travel_button(kind: ZoneKind, label: &'static str) -> impl Scene {
    bsn! {
        Button
        Node {
            padding: UiRect::axes(px(8), px(4)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(Color::srgb(0.25, 0.25, 0.25))
        Visibility::Hidden
        FastTravelButton(kind)
        on(move |_activate: On<Activate>, mut request: ResMut<FastTravelRequest>| {
            request.0 = Some(kind);
        })
        Children [
            Text(label)
            TextFont {
                font_size: px(14),
            }
            TextColor(Color::WHITE)
        ]
    }
}

fn minimap() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            bottom: px(16),
            right: px(16),
            width: px(120),
            height: px(120),
            border_radius: BorderRadius::all(px(12)),
        }
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5))
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Node {
                position_type: PositionType::Absolute,
                left: px(56),
                top: px(56),
                width: px(8),
                height: px(8),
            }
            BackgroundColor(Color::WHITE)
            --
            @minimap_zone_dot(ZoneKind::Shop, Color::srgb(0.2, 0.6, 0.9))
            --
            @minimap_zone_dot(ZoneKind::Guild, Color::srgb(0.9, 0.7, 0.2))
        ]
    }
}

fn minimap_zone_dot(kind: ZoneKind, color: Color) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(6),
            height: px(6),
        }
        BackgroundColor(color)
        Visibility::Hidden
        MinimapZoneDot(kind)
    }
}

fn inventory_scene() -> impl Scene {
    bsn! {
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
        }
        BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.95))
        InventoryRoot
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            @inventory_character()
            --
            @inventory_items()
            --
            @inventory_top_bar()
            --
            @inventory_menu_modal()
        ]
    }
}

fn inventory_top_bar() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: px(32),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8),
            padding: UiRect::horizontal(px(8)),
        }
        Children [
            Button
            Node {
                padding: UiRect::axes(px(12), px(6)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
            BackgroundColor(Color::srgb(0.3, 0.3, 0.35))
            on(|_activate: On<Activate>, mut ui: ResMut<UiStack>| {
                if ui.inventory_menu_open() {
                    ui.remove(UiLayer::InventoryMenu);
                } else {
                    ui.push(UiLayer::InventoryMenu);
                }
            })
            Children [
                Text("Menu")
                TextFont {
                    font_size: px(14),
                }
                TextColor(Color::WHITE)
            ]
        ]
    }
}

/// Sibling of the top bar and inventory contents, not a child of the top bar.
/// This gives the 600px overlay the correct containing block.
fn inventory_menu_modal() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6))
        bevy::ui::GlobalZIndex(20)
        InventoryMenuModal
        Visibility::Hidden
        Children [
            Node {
                width: px(260),
                padding: UiRect::all(px(16)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(px(16)),
            }
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2))
            Children [
                Text("Menu")
                TextFont {
                    font_size: px(24),
                }
                TextColor(Color::WHITE)
                --
                @inventory_menu_action("Continue", InventoryMenuAction::Continue)
                --
                @inventory_menu_action("Exit to Main", InventoryMenuAction::ExitToMain)
                --
                @inventory_menu_action("Exit to Desktop", InventoryMenuAction::ExitToDesktop)
                --
                @inventory_menu_action("Cancel", InventoryMenuAction::Continue)
            ]
        ]
    }
}

#[derive(Clone, Copy)]
enum InventoryMenuAction {
    Continue,
    ExitToMain,
    ExitToDesktop,
}

fn inventory_menu_action(label: &'static str, action: InventoryMenuAction) -> impl Scene {
    bsn! {
        Button
        Node {
            width: px(200),
            padding: UiRect::axes(px(12), px(8)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(8)),
        }
        BackgroundColor(Color::srgb(0.25, 0.25, 0.3))
        on(move |_activate: On<Activate>, mut ui: ResMut<UiStack>, mut next: ResMut<NextState<AppState>>, mut exit: MessageWriter<AppExit>| {
            ui.remove(UiLayer::InventoryMenu);
            match action {
                InventoryMenuAction::Continue => {}
                InventoryMenuAction::ExitToMain => {
                    ui.reset();
                    next.set(AppState::MainMenu);
                }
                InventoryMenuAction::ExitToDesktop => {
                    exit.write(AppExit::Success);
                }
            }
        })
        Children [
            Text(label)
            TextFont {
                font_size: px(16),
            }
            TextColor(Color::WHITE)
        ]
    }
}

fn inventory_character() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Relative,
            width: px(360),
            height: px(560),
        }
        Children [
            Node {
                position_type: PositionType::Absolute,
                top: px(0),
                left: px(0),
                width: percent(100),
                height: percent(100),
            }
            template(|ctx| {
                Ok(ImageNode::new(
                    ctx.resource::<AssetServer>().load("character/portrait.png"),
                ))
            })
            --
            Node {
                position_type: PositionType::Absolute,
                top: px(8),
                left: px(8),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                align_items: AlignItems::Center,
            }
            Children [
                @gear_slot("Head")
                --
                @gear_slot("Neck")
                --
                @gear_row_three()
                --
                @gear_row_two()
                --
                @gear_slot("Legs")
                --
                @gear_slot("Feet")
            ]
        ]
    }
}

fn gear_slot(label: &'static str) -> impl Scene {
    bsn! {
        Node {
            width: px(72),
            height: px(36),
            padding: UiRect::all(px(4)),
            flex_direction: FlexDirection::Row,
            column_gap: px(4),
            align_items: AlignItems::Center,
        }
        BackgroundColor(Color::srgba(0.25, 0.25, 0.3, 0.85))
        Children [
            Node {
                width: px(28),
                height: px(28),
            }
            BackgroundColor(Color::srgba(0.4, 0.4, 0.45, 0.8))
            --
            Text(label)
            TextFont {
                font_size: px(10),
            }
            TextColor(Color::WHITE)
        ]
    }
}

fn gear_row_three() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(8),
        }
        Children [
            @gear_slot("L Shldr")
            --
            @gear_slot("Torso")
            --
            @gear_slot("R Shldr")
        ]
    }
}

fn gear_row_two() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(8),
        }
        Children [
            @gear_slot("L Hand")
            --
            @gear_slot("R Hand")
        ]
    }
}

fn inventory_items() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            flex_grow: 1.0,
            padding: UiRect::all(px(8)),
        }
        BackgroundColor(Color::srgba(0.12, 0.12, 0.16, 0.9))
        Children [
            Node {
                flex_wrap: FlexWrap::Wrap,
                row_gap: px(4),
                column_gap: px(4),
            }
            Children [
                @filled_item_slot(evil_pup_icon())
                --
                @filled_item_slot(maurice_icon())
                --
                @filled_item_slot(salad_icon())
                --
                @filled_item_slot(scooby_icon())
                --
                @filled_item_slot(theo_icon())
                --
                @empty_item_slot()
                --
                @empty_item_slot()
                --
                @empty_item_slot()
                --
                @empty_item_slot()
                --
                @empty_item_slot()
                --
                @empty_item_slot()
                --
                @empty_item_slot()
            ]
            --
            Node {
                padding: UiRect::all(px(8)),
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
            }
            BackgroundColor(Color::srgb(0.12, 0.12, 0.16))
            Children [
                Text("Resources")
                TextFont {
                    font_size: px(14),
                }
                TextColor(Color::WHITE)
                --
                Text("Gold: 0")
                TextFont {
                    font_size: px(12),
                }
                TextColor(Color::srgb(0.8, 0.8, 0.8))
                CurrencyLabel::Gold
                --
                Text("Wood: 0")
                TextFont {
                    font_size: px(12),
                }
                TextColor(Color::srgb(0.8, 0.8, 0.8))
                CurrencyLabel::Wood
                --
                Text("Stone: 0")
                TextFont {
                    font_size: px(12),
                }
                TextColor(Color::srgb(0.8, 0.8, 0.8))
                CurrencyLabel::Stone
            ]
        ]
    }
}

fn filled_item_slot(icon: impl Scene) -> impl Scene {
    bsn! {
        Node {
            width: px(64),
            height: px(32),
            padding: UiRect::all(px(2)),
            flex_direction: FlexDirection::Row,
            column_gap: px(2),
        }
        BackgroundColor(Color::srgb(0.18, 0.18, 0.22))
        Children [
            @{icon}
            --
            Text("A12 D5")
            TextFont {
                font_size: px(8),
            }
            TextColor(Color::srgb(0.85, 0.85, 0.85))
        ]
    }
}

fn item_icon(path: &'static str) -> impl Scene {
    bsn! {
        Node {
            width: px(32),
            height: px(32),
        }
        template(|ctx| {
            Ok(ImageNode::new(ctx.resource::<AssetServer>().load(path)))
        })
    }
}

fn evil_pup_icon() -> impl Scene {
    item_icon("items/evil-pup.png")
}

fn maurice_icon() -> impl Scene {
    item_icon("items/maurice.webp")
}

fn salad_icon() -> impl Scene {
    item_icon("items/salad.jpg")
}

fn scooby_icon() -> impl Scene {
    item_icon("items/scooby.webp")
}

fn theo_icon() -> impl Scene {
    item_icon("items/theo.jpg")
}

fn empty_item_slot() -> impl Scene {
    bsn! {
        Node {
            width: px(64),
            height: px(32),
            padding: UiRect::all(px(2)),
            flex_direction: FlexDirection::Row,
            column_gap: px(2),
        }
        BackgroundColor(Color::srgb(0.18, 0.18, 0.22))
        Children [
            Node {
                width: px(32),
                height: px(32),
            }
            BackgroundColor(Color::srgba(0.3, 0.3, 0.35, 0.6))
            --
            Text(" ")
            TextFont {
                font_size: px(8),
            }
            TextColor(Color::srgb(0.85, 0.85, 0.85))
        ]
    }
}

fn pause_overlay() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6))
        PauseOverlay
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Text("PAUSED  (P to resume)")
            TextFont {
                font_size: px(48),
            }
            TextColor(Color::WHITE)
        ]
    }
}

pub fn handle_ui_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    nearby: Res<NearbyZone>,
    mut ui: ResMut<UiStack>,
    mut confirm_request: ResMut<PopupConfirmRequested>,
) {
    let confirm = keys.just_pressed(KeyCode::KeyE) || confirm_request.0;
    confirm_request.0 = false;

    if confirm && ui.active_panel().is_none() && !ui.inventory_open() {
        if let Some(kind) = nearby.0 {
            ui.push(UiLayer::Panel(kind));
        }
    }

    if keys.just_pressed(KeyCode::Escape) && ui.pop_top().is_none() {
        ui.release_cursor();
    }

    if mouse.just_pressed(MouseButton::Right) {
        if ui.is_empty() {
            if ui.cursor_capture_requested() {
                ui.push(UiLayer::Inventory);
            } else {
                ui.capture_cursor();
            }
        } else {
            ui.pop_top();
        }
    }

    if keys.just_pressed(KeyCode::KeyP) {
        ui.toggle_pause();
    }

    if let Some(active) = ui.active_panel() {
        if nearby.0 != Some(active) {
            ui.remove(UiLayer::Panel(active));
            if ui.is_empty() {
                ui.capture_cursor();
            }
        }
    }
}

fn panel_visibility_system(
    ui: Res<UiStack>,
    mut panels: Query<(&PanelRoot, &mut Visibility)>,
) {
    let active = ui.active_panel();
    for (panel, mut visibility) in &mut panels {
        *visibility = if active == Some(panel.0) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn popup_visibility_system(
    nearby: Res<NearbyZone>,
    ui: Res<UiStack>,
    mut popups: Query<&mut Visibility, With<PopupRoot>>,
) {
    let visible = nearby.0.is_some()
        && ui.active_panel().is_none()
        && !ui.inventory_open()
        && !ui.paused();

    for mut visibility in &mut popups {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn fast_travel_menu_visibility_system(
    ui: Res<UiStack>,
    mut menus: Query<&mut Visibility, With<FastTravelMenu>>,
) {
    for mut visibility in &mut menus {
        *visibility = if ui.inventory_open() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn fast_travel_button_visibility_system(
    ui: Res<UiStack>,
    visited: Res<VisitedZones>,
    mut buttons: Query<(&FastTravelButton, &mut Visibility)>,
) {
    for (button, mut visibility) in &mut buttons {
        *visibility = if ui.inventory_open() && visited.0.contains(&button.0) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn inventory_visibility_system(
    ui: Res<UiStack>,
    mut roots: Query<&mut Visibility, With<InventoryRoot>>,
) {
    for mut visibility in &mut roots {
        *visibility = if ui.inventory_open() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn inventory_menu_visibility_system(
    ui: Res<UiStack>,
    mut menus: Query<&mut Visibility, With<InventoryMenuModal>>,
) {
    for mut visibility in &mut menus {
        *visibility = if ui.inventory_menu_open() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn pause_overlay_visibility_system(
    ui: Res<UiStack>,
    mut overlays: Query<&mut Visibility, With<PauseOverlay>>,
) {
    for mut visibility in &mut overlays {
        *visibility = if ui.paused() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn sync_cursor(ui: Res<UiStack>, mut cursor: Single<&mut CursorOptions>) {
    let grabbed = ui.cursor_grabbed();
    cursor.visible = !grabbed;
    cursor.grab_mode = if grabbed {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
}

fn update_minimap(
    player: Query<&Transform, With<crate::player::Player>>,
    zones: Query<(&Transform, &crate::zone::TriggerZone)>,
    mut dots: Query<(&MinimapZoneDot, &mut Node, &mut Visibility)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };

    let player_position = player_transform.translation;
    let zone_positions: Vec<(ZoneKind, Vec3)> = zones
        .iter()
        .map(|(transform, zone)| (zone.kind, transform.translation))
        .collect();

    for (dot, mut node, mut visibility) in &mut dots {
        let Some((_, position)) = zone_positions.iter().find(|(kind, _)| *kind == dot.0) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let relative = *position - player_position;
        let x = 60.0 + relative.x * 4.0 - 3.0;
        let y = 60.0 - relative.z * 4.0 - 3.0;

        if !(0.0..=114.0).contains(&x) || !(0.0..=114.0).contains(&y) {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
            node.left = px(x);
            node.top = px(y);
        }
    }
}
