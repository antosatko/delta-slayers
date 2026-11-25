use bevy::{
    prelude::*,
    ui_widgets::Activate,
};
use delta_slayers_ui::{
    CarbonButtonTone, CarbonIcon, CarbonPlugin, CarbonStatus, CarbonTheme, action_button, badge,
    item_tile_optional, page_header, panel, reference_term, root, screen_tab, screen_tabs,
    section_header, stat_bar,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CarbonPlugin)
        .add_systems(Startup, scene.spawn())
        .run();
}

fn scene() -> impl SceneList {
    bsn_list! {
        Camera2d
        --
        @showcase()
    }
}

fn showcase() -> impl Scene {
    bsn! {
        @root()
        Children [
            Node {
                flex_grow: 1.0,
                min_width: px(0),
                height: percent(100),
                padding: UiRect::axes(px(24), px(20)),
                flex_direction: FlexDirection::Column,
                row_gap: px(13),
            }
            Children [
                @page_header(
                    "Inventory",
                    "Carried items",
                    "Items currently carried by the party. Select one to inspect its details."
                )
                --
                @screen_tabs()
                Children [
                    @screen_tab("Items", true)
                    on(|_activate: On<Activate>| info!("Items tab selected (showcase only)"))
                    --
                    @screen_tab("Equipment", false)
                    on(|_activate: On<Activate>| info!("Equipment tab selected (showcase only)"))
                    --
                    @screen_tab("Materials", false)
                    on(|_activate: On<Activate>| info!("Materials tab selected (showcase only)"))
                ]
                --
                Node {
                    width: percent(100),
                    flex_grow: 1.0,
                    min_height: px(0),
                    flex_direction: FlexDirection::Row,
                    column_gap: px(14),
                }
                Children [
                    Node {
                        flex_grow: 1.0,
                        min_width: px(0),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(13),
                    }
                    Children [
                        @panel(percent(100))
                        Children [
                            @section_header(CarbonIcon::Inventory, "Carried items")
                            --
                            Node {
                                width: percent(100),
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: px(8),
                                row_gap: px(8),
                            }
                            Children [
                                @item_tile_optional(
                                    CarbonIcon::Sword,
                                    "Iron longsword",
                                    None,
                                    false
                                )
                                --
                                @item_tile_optional(
                                    CarbonIcon::Shield,
                                    "Round shield",
                                    None,
                                    false
                                )
                                --
                                @item_tile_optional(
                                    CarbonIcon::Flask,
                                    "Healing draught",
                                    Some("3"),
                                    true
                                )
                                --
                                @item_tile_optional(
                                    CarbonIcon::Coin,
                                    "Crowns",
                                    Some("312"),
                                    false
                                )
                            ]
                            --
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(8),
                            }
                            @badge("8 ITEMS", CarbonStatus::Neutral)
                            --
                            @badge("1 NEW", CarbonStatus::Positive)
                        ]
                        --
                        @panel(percent(100))
                        Children [
                            @section_header(CarbonIcon::Health, "Party condition")
                            --
                            @stat_bar(
                                CarbonIcon::Health,
                                "Health",
                                "86 / 100",
                                0.86,
                                CarbonStatus::Positive
                            )
                            --
                            @stat_bar(
                                CarbonIcon::Stamina,
                                "Stamina",
                                "62 / 80",
                                0.775,
                                CarbonStatus::Warning
                            )
                            --
                            @stat_bar(
                                CarbonIcon::Shield,
                                "Armour",
                                "18%",
                                0.18,
                                CarbonStatus::Neutral
                            )
                        ]
                    }
                    --
                    Node {
                        width: px(280),
                        min_width: px(250),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(13),
                    }
                    Children [
                        @panel(percent(100))
                        Children [
                            @section_header(CarbonIcon::Flask, "Selected item")
                            --
                            @item_tile_optional(
                                CarbonIcon::Flask,
                                "Healing draught",
                                Some("3"),
                                true
                            )
                            --
                            @badge("CONSUMABLE", CarbonStatus::Neutral)
                            --
                            Text("A bitter herbal draught. Restores a moderate amount of health.")
                            TextFont { font_size: px(12) }
                            template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
                            --
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::SpaceBetween,
                            }
                            Children [
                                Text("Value")
                                TextFont { font_size: px(12) }
                                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().quiet)))
                                --
                                Text("18 crowns")
                                TextFont { font_size: px(12) }
                                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().ink)))
                            ]
                            --
                            @action_button(
                                "Use item",
                                CarbonIcon::Confirm,
                                CarbonButtonTone::Primary
                            )
                            on(|_activate: On<Activate>| info!("Use item clicked (showcase only)"))
                            --
                            @action_button("Drop", CarbonIcon::Close, CarbonButtonTone::Ghost)
                            on(|_activate: On<Activate>| info!("Drop clicked (showcase only)"))
                        ]
                        --
                        @panel(percent(100))
                        Children [
                            @section_header(CarbonIcon::Book, "Reference")
                            --
                            Text("Some objects have additional context.")
                            TextFont { font_size: px(12) }
                            template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
                            --
                            @reference_term(
                                "Brine-wight",
                                "Brine-wight",
                                "LOCAL FOLKLORE · UNCONFIRMED",
                                concat!(
                                    "A coastal name for something believed to return from the water. ",
                                    "The surviving reports do not establish whether it describes ",
                                    "a creature, a condition of the watercourse, or an old debt."
                                )
                            )
                        ]
                    ]
                ]
            }
        ]
    }
}
