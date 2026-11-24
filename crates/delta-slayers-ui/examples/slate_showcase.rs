use bevy::{
    prelude::*,
    ui_widgets::Activate,
};
use delta_slayers_ui::{
    SlateButtonTone, SlateIcon, SlatePlugin, SlateStatus, SlateTheme, action_button, badge,
    item_tile, page_header, panel, reference_term, root, section_header, stat_bar,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SlatePlugin)
        .add_systems(Startup, scene().spawn())
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
            @sidebar()
            --
            Node {
                flex_grow: 1.0,
                min_width: px(0),
                height: percent(100),
                padding: UiRect::axes(px(24), px(20)),
                flex_direction: FlexDirection::Column,
                row_gap: px(14),
            }
            Children [
                @page_header(
                    "GREYBRIDGE CHAPTER",
                    "The bell beneath the mill",
                    "A compact UI showcase built from reusable Slate scenes."
                )
                --
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexStart,
                    column_gap: px(14),
                }
                Children [
                    Node {
                        flex_grow: 1.0,
                        min_width: px(0),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(12),
                    }
                    Children [
                        @panel(percent(100))
                        Children [
                            @section_header(SlateIcon::Contracts, "Contract status")
                            --
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(8),
                            }
                            @badge("AVAILABLE", SlateStatus::Positive)
                            --
                            Text("Investigate the mill's night bell and locate the missing miller.")
                            TextFont { font_size: px(14) }
                            template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().muted)))
                            --
                            @reference_term(
                                "brine-wight",
                                "Brine-wight",
                                "LOCAL FOLKLORE · UNCONFIRMED",
                                "A coastal name for something believed to return from the water. The surviving reports do not establish whether it describes a creature, a condition of the watercourse, or an old debt given a shape."
                            )
                            --
                            @action_button(
                                "Accept contract",
                                SlateIcon::Confirm,
                                SlateButtonTone::Primary
                            )
                            on(|_activate: On<Activate>| {
                                info!("Contract accepted (showcase only)");
                            })
                        ]
                        --
                        @panel(percent(100))
                        Children [
                            @section_header(SlateIcon::Character, "Party condition")
                            --
                            @stat_bar(SlateIcon::Health, "Health", "86 / 100", 0.86, SlateStatus::Positive)
                            --
                            @stat_bar(SlateIcon::Stamina, "Stamina", "62 / 80", 0.775, SlateStatus::Warning)
                            --
                            @stat_bar(SlateIcon::Shield, "Armour", "18%", 0.18, SlateStatus::Neutral)
                        ]
                    ]
                    --
                    Node {
                        width: px(280),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(12),
                    }
                    Children [
                        @panel(percent(100))
                        Children [
                            @section_header(SlateIcon::Inventory, "Selected item")
                            --
                            @item_tile(
                                SlateIcon::Sword,
                                "Iron longsword",
                                "1 · EQUIPPED",
                                true
                            )
                            --
                            @badge("GOOD CONDITION", SlateStatus::Positive)
                            --
                            Text("A serviceable guild issue blade. The edge has been sharpened recently.")
                            TextFont { font_size: px(12) }
                            template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().muted)))
                            --
                            @action_button(
                                "Unequip",
                                SlateIcon::Close,
                                SlateButtonTone::Secondary
                            )
                            on(|_activate: On<Activate>| {
                                info!("Unequip clicked (showcase only)");
                            })
                        ]
                        --
                        @panel(percent(100))
                        Children [
                            @section_header(SlateIcon::Coin, "Purse")
                            --
                            Text("312 crowns")
                            TextFont { font_size: px(24) }
                            template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().ink)))
                            --
                            @badge("GREYBRIDGE", SlateStatus::Neutral)
                        ]
                    ]
                ]
            ]
        ]
    }
}

fn sidebar() -> impl Scene {
    bsn! {
        Node {
            width: px(164),
            height: percent(100),
            padding: UiRect::all(px(10)),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            border: UiRect::right(px(1)),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<SlateTheme>().panel_alt)))
        template(|ctx| Ok(BorderColor::all(ctx.resource::<SlateTheme>().line)))
        Children [
            Text("DELTA SLAYERS")
            TextFont { font_size: px(15) }
            template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().ink)))
            --
            @nav_item(SlateIcon::Contracts, "Contracts", true)
            --
            @nav_item(SlateIcon::Inventory, "Inventory", false)
            --
            @nav_item(SlateIcon::Character, "Character", false)
            --
            @nav_item(SlateIcon::Journal, "Journal", false)
            --
            @nav_item(SlateIcon::Map, "Map", false)
            --
            @nav_item(SlateIcon::Trade, "Trade", false)
            --
            @nav_item(SlateIcon::Workshop, "Workshop", false)
        ]
    }
}

fn nav_item(icon: SlateIcon, label: &'static str, active: bool) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            min_height: px(36),
            padding: UiRect::axes(px(8), px(6)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8),
            border: UiRect::all(if active { px(1) } else { px(0) }),
        }
        template(|ctx| {
            let theme = ctx.resource::<SlateTheme>();
            Ok(BackgroundColor(if active { theme.panel } else { Color::NONE }))
        })
        template(|ctx| Ok(BorderColor::all(ctx.resource::<SlateTheme>().line)))
        Text(icon.glyph())
        TextFont { font_size: px(17) }
        template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().accent)))
        --
        Text(label)
        TextFont { font_size: px(13) }
        template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().muted)))
    }
}
