use bevy::{prelude::*, ui_widgets::Activate};
use delta_slayers_ui::{
    CarbonButtonTone, CarbonIcon, CarbonTheme, action_button, equipment_slot, eyebrow_text,
    item_art_tile, key_hint, muted_text, page_header, page_title, panel, section_header,
};

use crate::{
    app::state::AppState,
    gameplay::zones::{FastTravelRequest, ZoneKind},
    ui::{
        components::{
            FastTravelButton, FastTravelMenu, GameMenuModal, MinimapZoneDot, PanelRoot,
            PauseOverlay, PopupConfirmRequested, PopupRoot,
        },
        inventory::{CurrencyLabel, InventoryRoot},
        stack::{UiLayer, UiStack},
    },
};

/// All in-game UI is declared as BSN scenes. Gameplay setup must not spawn UI.
pub(in crate::ui) fn scene() -> impl SceneList {
    bsn_list! {
        Camera2d
        Camera {
            order: 1,
        }
        IsDefaultUiCamera
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
        @game_menu_modal()
        --
        @pause_overlay()
    }
}

fn zone_panel(
    kind: ZoneKind,
    title: &'static str,
    icon: CarbonIcon,
    section: &'static str,
    description: &'static str,
) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            right: px(16),
            width: px(390),
            max_width: percent(100),
            min_width: px(0),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().overlay)))
        bevy::ui::GlobalZIndex(10)
        PanelRoot(kind)
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            @panel(percent(100))
            Children [
                @page_title(title)
                --
                @section_header(icon, section)
                --
                @eyebrow_text("NOT AVAILABLE YET")
                --
                @muted_text(description, 13.0)
                --
                @key_hint("ESC", "Close this window")
            ]
        ]
    }
}

fn shop_panel() -> impl Scene {
    zone_panel(
        ZoneKind::Shop,
        "Shop",
        CarbonIcon::Trade,
        "Merchant stock",
        "Trading is not connected yet. This panel will hold merchant offers and purchase details.",
    )
}

fn guild_panel() -> impl Scene {
    zone_panel(
        ZoneKind::Guild,
        "Guild",
        CarbonIcon::Contracts,
        "Guild board",
        "Guild contracts and party services will be shown here when those systems are implemented.",
    )
}

fn interaction_prompt() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            bottom: px(28),
            width: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        bevy::ui::GlobalZIndex(8)
        PopupRoot
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Node {
                width: px(330),
                max_width: percent(90),
                min_width: px(0),
                padding: UiRect::all(px(10)),
                border_radius: BorderRadius::all(px(8)),
            }
            template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().panel)))
            Children [
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: px(12),
                    row_gap: px(8),
                }
                Children [
                    @key_hint("E", "Interact")
                    --
                    @action_button("Open", CarbonIcon::Confirm, CarbonButtonTone::Primary)
                    on(|_activate: On<Activate>, mut request: ResMut<PopupConfirmRequested>| {
                        request.0 = true;
                    })
                ]
            ]
        ]
    }
}

fn fast_travel_menu() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            right: px(12),
            width: px(224),
            max_width: percent(60),
            min_width: px(0),
        }
        bevy::ui::GlobalZIndex(18)
        FastTravelMenu
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            @panel(percent(100))
            Children [
                @section_header(CarbonIcon::Map, "Fast travel")
                --
                @muted_text("Travel to places you have already visited.", 11.0)
                --
                @fast_travel_button(ZoneKind::Shop, "Shop", CarbonIcon::Trade)
                --
                @fast_travel_button(ZoneKind::Guild, "Guild", CarbonIcon::Contracts)
            ]
        ]
    }
}

fn fast_travel_button(kind: ZoneKind, label: &'static str, icon: CarbonIcon) -> impl Scene {
    bsn! {
        @action_button(label, icon, CarbonButtonTone::Secondary)
        FastTravelButton(kind)
        Visibility::Hidden
        on(move |_activate: On<Activate>, mut request: ResMut<FastTravelRequest>| {
            request.0 = Some(kind);
        })
    }
}

fn minimap() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            bottom: px(16),
            right: px(16),
            width: px(144),
            height: px(144),
            padding: UiRect::all(px(8)),
            border_radius: BorderRadius::all(px(9)),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().panel)))
        bevy::ui::GlobalZIndex(2)
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Node {
                position_type: PositionType::Relative,
                width: px(120),
                height: px(120),
                overflow: Overflow::clip(),
                border_radius: BorderRadius::all(px(5)),
            }
            template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().surface_deep)))
            Children [
                Node {
                    position_type: PositionType::Absolute,
                    left: px(56),
                    top: px(56),
                    width: px(8),
                    height: px(8),
                    border_radius: BorderRadius::all(px(4)),
                }
                template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().ink)))
                --
                @minimap_zone_dot(ZoneKind::Shop)
                --
                @minimap_zone_dot(ZoneKind::Guild)
            ]
        ]
    }
}

fn minimap_zone_dot(kind: ZoneKind) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(7),
            height: px(7),
            border_radius: BorderRadius::all(px(4)),
        }
        MinimapZoneDot(kind)
        Visibility::Hidden
        template(move |ctx| {
            let theme = ctx.resource::<CarbonTheme>();
            let color = match kind {
                ZoneKind::Shop => theme.accent,
                ZoneKind::Guild => theme.warning,
            };
            Ok(BackgroundColor(color))
        })
    }
}

fn inventory_scene() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(20)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().overlay)))
        bevy::ui::GlobalZIndex(12)
        InventoryRoot
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Node {
                width: percent(100),
                height: percent(100),
                max_width: px(1160),
                max_height: px(780),
                min_width: px(0),
                min_height: px(0),
                overflow: Overflow::clip(),
                padding: UiRect::all(px(14)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: px(14),
                border_radius: BorderRadius::all(px(9)),
            }
            template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().panel)))
            Children [
                @page_header(
                    "Inventory",
                    "Party inventory",
                    "Review carried items, equipment, and party resources. Item tiles are visual inventory entries."
                )
                --
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: px(12),
                    row_gap: px(8),
                }
                Children [
                    @key_hint("RMB", "Close inventory")
                    --
                    Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        align_items: AlignItems::Center,
                        column_gap: px(8),
                    }
                    Children [
                        @action_button("Game menu", CarbonIcon::Settings, CarbonButtonTone::Ghost)
                        on(|_activate: On<Activate>, mut ui: ResMut<UiStack>| {
                            ui.push(UiLayer::GameMenu);
                        })
                        --
                        @action_button("Close", CarbonIcon::Close, CarbonButtonTone::Secondary)
                        on(|_activate: On<Activate>, mut ui: ResMut<UiStack>| {
                            ui.remove(UiLayer::Inventory);
                        })
                    ]
                ]
                --
                Node {
                    width: percent(100),
                    flex_grow: 1.0,
                    min_width: px(0),
                    min_height: px(0),
                    overflow: Overflow::scroll(),
                    flex_direction: FlexDirection::Column,
                }
                Children [
                    Node {
                        width: percent(100),
                        min_width: px(0),
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        align_content: AlignContent::Start,
                        align_items: AlignItems::Stretch,
                        column_gap: px(14),
                        row_gap: px(14),
                    }
                    Children [
                        Node {
                            width: px(250),
                            flex_basis: px(240),
                            min_width: px(190),
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            flex_direction: FlexDirection::Column,
                        row_gap: px(12),
                    }
                    Children [
                        @character_panel()
                        --
                        @resources_panel()
                    ]
                    --
                        Node {
                            flex_grow: 1.0,
                            flex_basis: px(420),
                            min_width: px(190),
                            flex_shrink: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(12),
                        }
                        Children [
                            @inventory_items()
                        ]
                    ]
                ]
            ]
        ]
    }
}

fn character_panel() -> impl Scene {
    bsn! {
        @panel(percent(100))
        Children [
            @section_header(CarbonIcon::Character, "Character")
            --
            Node {
                width: percent(100),
                height: px(190),
                min_width: px(0),
                border_radius: BorderRadius::all(px(6)),
                overflow: Overflow::clip(),
            }
            template(|ctx| {
                Ok(ImageNode::new(
                    ctx.resource::<AssetServer>().load("character/portrait.png"),
                ))
            })
            --
            @section_header(CarbonIcon::Shield, "Equipment")
            --
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::Start,
                column_gap: px(7),
                row_gap: px(7),
            }
            Children [
                @equipment_slot(CarbonIcon::Shield)
                --
                @equipment_slot(CarbonIcon::Book)
                --
                @equipment_slot(CarbonIcon::Sword)
                --
                @equipment_slot(CarbonIcon::Sword)
                --
                @equipment_slot(CarbonIcon::Flask)
                --
                @equipment_slot(CarbonIcon::Shield)
                --
                @equipment_slot(CarbonIcon::Compass)
            ]
        ]
    }
}

fn resources_panel() -> impl Scene {
    bsn! {
        @panel(percent(100))
        Children [
            @section_header(CarbonIcon::Coin, "Resources")
            --
            Text("Gold: 0")
            CurrencyLabel::Gold
            TextFont { font_size: px(12) }
            template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
            --
            Text("Wood: 0")
            CurrencyLabel::Wood
            TextFont { font_size: px(12) }
            template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
            --
            Text("Stone: 0")
            CurrencyLabel::Stone
            TextFont { font_size: px(12) }
            template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
        ]
    }
}

fn inventory_items() -> impl Scene {
    bsn! {
        @panel(percent(100))
        Children [
            @section_header(CarbonIcon::Inventory, "Carried items")
            --
            Node {
                width: percent(100),
                min_width: px(0),
                min_height: px(112),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::Start,
                align_items: AlignItems::Start,
                column_gap: px(8),
                row_gap: px(8),
            }
            Children [
                @item_art_tile("items/evil-pup.png", "Evil pup", None)
                --
                @item_art_tile("items/maurice.webp", "Maurice", None)
                --
                @item_art_tile("items/salad.jpg", "Salad", None)
                --
                @item_art_tile("items/scooby.webp", "Scooby", None)
                --
                @item_art_tile("items/theo.jpg", "Theo", None)
            ]
        ]
    }
}

fn game_menu_modal() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(20)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().overlay)))
        bevy::ui::GlobalZIndex(30)
        GameMenuModal
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Node {
                width: px(360),
                max_width: percent(92),
                min_width: px(0),
                max_height: percent(90),
                overflow: Overflow::scroll(),
            }
            Children [
                @panel(percent(100))
                Children [
                    @page_title("Game menu")
                    --
                    @muted_text("The current session is suspended while this menu is open.", 12.0)
                    --
                    @game_menu_action("Continue", CarbonIcon::Back, GameMenuAction::Continue, CarbonButtonTone::Primary)
                    --
                    @game_menu_action("Exit to main menu", CarbonIcon::Compass, GameMenuAction::ExitToMain, CarbonButtonTone::Secondary)
                    --
                    @game_menu_action("Exit to desktop", CarbonIcon::Close, GameMenuAction::ExitToDesktop, CarbonButtonTone::Danger)
                ]
            ]
        ]
    }
}

#[derive(Clone, Copy)]
enum GameMenuAction {
    Continue,
    ExitToMain,
    ExitToDesktop,
}

fn game_menu_action(
    label: &'static str,
    icon: CarbonIcon,
    action: GameMenuAction,
    tone: CarbonButtonTone,
) -> impl Scene {
    bsn! {
        @action_button(label, icon, tone)
        on(move |_activate: On<Activate>,
            mut ui: ResMut<UiStack>,
            mut next: ResMut<NextState<AppState>>,
            mut exit: MessageWriter<AppExit>| {
            ui.remove(UiLayer::GameMenu);
            match action {
                GameMenuAction::Continue => {}
                GameMenuAction::ExitToMain => {
                    ui.reset();
                    next.set(AppState::MainMenu);
                }
                GameMenuAction::ExitToDesktop => {
                    exit.write(AppExit::Success);
                }
            }
        })
    }
}

fn pause_overlay() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(20)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().overlay)))
        bevy::ui::GlobalZIndex(25)
        PauseOverlay
        Visibility::Hidden
        DespawnOnExit::<AppState>(AppState::InGame)
        Children [
            Node {
                width: px(340),
                max_width: percent(92),
                min_width: px(0),
                max_height: percent(90),
                overflow: Overflow::scroll(),
            }
            Children [
                @panel(percent(100))
                Children [
                    @page_title("Paused")
                    --
                    @muted_text("Press P to resume or use the actions below.", 12.0)
                    --
                    @action_button("Resume", CarbonIcon::Back, CarbonButtonTone::Primary)
                    on(|_activate: On<Activate>, mut ui: ResMut<UiStack>| {
                        ui.remove(UiLayer::Pause);
                    })
                    --
                    @action_button("Open game menu", CarbonIcon::Settings, CarbonButtonTone::Secondary)
                    on(|_activate: On<Activate>, mut ui: ResMut<UiStack>| {
                        ui.push(UiLayer::GameMenu);
                    })
                ]
            ]
        ]
    }
}
