use bevy::{prelude::*, ui_widgets::Activate};
use delta_slayers_ui::{
    CarbonButtonTone, CarbonIcon, CarbonPlugin, CarbonSelection, CarbonStatus, CarbonTab,
    CarbonTheme, action_button, badge, item_tile_optional, page_header, panel, reference_term,
    root, screen_tab, screen_tabs, section_header, stat_bar,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Carbon UI Showcase".into(),
                resolution: (1360.0, 900.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CarbonPlugin)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShowcasePage {
    Items,
    Equipment,
    Materials,
}

#[derive(Component, Clone, Copy)]
struct ShowcaseTab(ShowcasePage);

#[derive(Component, Clone, Copy)]
struct ShowcaseContent(ShowcasePage);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShowcaseItemKind {
    Sword,
    Shield,
    Draught,
    Crowns,
}

impl ShowcaseItemKind {
    const fn title(self) -> &'static str {
        match self {
            Self::Sword => "Iron longsword",
            Self::Shield => "Round shield",
            Self::Draught => "Healing draught",
            Self::Crowns => "Crowns",
        }
    }

    const fn category(self) -> &'static str {
        match self {
            Self::Sword => "WEAPON",
            Self::Shield => "ARMOUR",
            Self::Draught => "CONSUMABLE",
            Self::Crowns => "CURRENCY",
        }
    }

    const fn description(self) -> &'static str {
        match self {
            Self::Sword => "A dependable, well-balanced blade. Suitable for close combat.",
            Self::Shield => "A simple round shield with a reinforced wooden core.",
            Self::Draught => "A bitter herbal draught. Restores a moderate amount of health.",
            Self::Crowns => "The common currency used by merchants throughout the region.",
        }
    }

    const fn icon(self) -> CarbonIcon {
        match self {
            Self::Sword => CarbonIcon::Sword,
            Self::Shield => CarbonIcon::Shield,
            Self::Draught => CarbonIcon::Flask,
            Self::Crowns => CarbonIcon::Coin,
        }
    }

    const fn value(self) -> &'static str {
        match self {
            Self::Sword => "84 crowns",
            Self::Shield => "42 crowns",
            Self::Draught => "18 crowns",
            Self::Crowns => "312 crowns",
        }
    }
}

#[derive(Component, Clone, Copy)]
struct ShowcaseItem(ShowcaseItemKind);

#[derive(Component, Clone, Copy)]
enum ShowcaseDetailField {
    Icon,
    Title,
    Category,
    Description,
    Value,
}

#[derive(Component)]
struct ShowcaseFeedback;

fn showcase() -> impl Scene {
    bsn! {
        @root()
        Children [
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                min_height: px(0),
                max_width: px(1440),
                flex_grow: 1.0,
                padding: UiRect::axes(px(24), px(20)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: px(13),
            }
            Children [
                @page_header(
                    "Inventory",
                    "Carried items",
                    "Browse the party's inventory, inspect an item, and switch between inventory views."
                )
                --
                @screen_tabs()
                Children [
                    @showcase_tab("Items", ShowcasePage::Items, true)
                    --
                    @showcase_tab("Equipment", ShowcasePage::Equipment, false)
                    --
                    @showcase_tab("Materials", ShowcasePage::Materials, false)
                ]
                --
                @items_page()
                --
                @equipment_page()
                --
                @materials_page()
            ]
        ]
    }
}

fn showcase_tab(
    label: &'static str,
    page: ShowcasePage,
    selected: bool,
) -> impl Scene {
    bsn! {
        @screen_tab(label, selected)
        ShowcaseTab(page)
        on(move |_activate: On<Activate>,
            mut tabs: Query<(&ShowcaseTab, &mut CarbonTab)>,
            mut contents: Query<(&ShowcaseContent, &mut Node)>| {
            for (tab_page, mut tab) in &mut tabs {
                tab.selected = tab_page.0 == page;
            }

            for (content_page, mut node) in &mut contents {
                node.display = if content_page.0 == page {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        })
    }
}

fn items_page() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_width: px(0),
            min_height: px(0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Start,
            align_content: AlignContent::Start,
            column_gap: px(14),
            row_gap: px(14),
        }
        ShowcaseContent(ShowcasePage::Items)
        Children [
            Node {
                flex_grow: 1.0,
                flex_basis: px(500),
                min_width: px(340),
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
                        min_width: px(0),
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        align_content: AlignContent::Start,
                        column_gap: px(8),
                        row_gap: px(8),
                    }
                    Children [
                        @showcase_item(ShowcaseItemKind::Sword, CarbonIcon::Sword, None, false)
                        --
                        @showcase_item(ShowcaseItemKind::Shield, CarbonIcon::Shield, None, false)
                        --
                        @showcase_item(ShowcaseItemKind::Draught, CarbonIcon::Flask, Some("3"), true)
                        --
                        @showcase_item(ShowcaseItemKind::Crowns, CarbonIcon::Coin, Some("312"), false)
                    ]
                    --
                    Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(8),
                        row_gap: px(6),
                    }
                    Children [
                        @badge("8 ITEMS", CarbonStatus::Neutral)
                        --
                        @badge("1 NEW", CarbonStatus::Positive)
                    ]
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
            ]
            --
            Node {
                width: px(280),
                min_width: px(250),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(13),
            }
            Children [
                @panel(percent(100))
                Children [
                    @section_header(CarbonIcon::Flask, "Selected item")
                    --
                    Node {
                        width: px(112),
                        height: px(112),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        align_self: AlignSelf::Start,
                        border_radius: BorderRadius::all(px(7)),
                    }
                    template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().panel_alt)))
                    Children [
                        Text(CarbonIcon::Flask.glyph())
                        ShowcaseDetailField::Icon
                        TextFont { font_size: px(36) }
                        template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
                    ]
                    --
                    Text("CONSUMABLE")
                    ShowcaseDetailField::Category
                    TextFont { font_size: px(10) }
                    template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().quiet)))
                    --
                    Text("Healing draught")
                    ShowcaseDetailField::Title
                    TextFont { font_size: px(16) }
                    template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().ink)))
                    --
                    Node {
                        width: percent(100),
                        min_width: px(0),
                    }
                    Text("A bitter herbal draught. Restores a moderate amount of health.")
                    ShowcaseDetailField::Description
                    TextFont { font_size: px(12) }
                    template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
                    --
                    Node {
                        width: percent(100),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        column_gap: px(8),
                    }
                    Children [
                        Text("Value")
                        TextFont { font_size: px(12) }
                        template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().quiet)))
                        --
                        Text("18 crowns")
                        ShowcaseDetailField::Value
                        TextFont { font_size: px(12) }
                        template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().ink)))
                    ]
                    --
                    @action_button(
                        "Use item",
                        CarbonIcon::Confirm,
                        CarbonButtonTone::Primary
                    )
                    on(|_activate: On<Activate>,
                        items: Query<(&ShowcaseItem, &CarbonSelection)>,
                        mut feedback: Query<&mut Text, With<ShowcaseFeedback>>| {
                        let selected = items.iter()
                            .find(|(_, selection)| selection.selected)
                            .map(|(item, _)| item.0);

                        let message = match selected {
                            Some(ShowcaseItemKind::Draught) =>
                                "Used a healing draught. (Showcase only)",
                            Some(item) =>
                                match item {
                                    ShowcaseItemKind::Crowns => "Currency cannot be used.",
                                    _ => "This item cannot be used from the demo.",
                                },
                            None => "Select an item first.",
                        };

                        for mut text in &mut feedback {
                            text.0 = message.into();
                        }
                    })
                    --
                    @action_button("Drop", CarbonIcon::Close, CarbonButtonTone::Ghost)
                    on(|_activate: On<Activate>,
                        items: Query<(&ShowcaseItem, &CarbonSelection)>,
                        mut feedback: Query<&mut Text, With<ShowcaseFeedback>>| {
                        let selected = items.iter()
                            .find(|(_, selection)| selection.selected)
                            .map(|(item, _)| item.0);

                        let message = selected.map_or(
                            "Select an item first.".to_owned(),
                            |item| format!("Dropped {}. (Showcase only)", item.title()),
                        );

                        for mut text in &mut feedback {
                            text.0 = message.clone();
                        }
                    })
                    --
                    Text("Select an item to inspect it.")
                    ShowcaseFeedback
                    Node { width: percent(100), min_width: px(0) }
                    TextFont { font_size: px(11) }
                    template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().quiet)))
                ]
                --
                @panel(percent(100))
                Children [
                    @section_header(CarbonIcon::Book, "Reference")
                    --
                    Node {
                        width: percent(100),
                        min_width: px(0),
                    }
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
}

fn showcase_item(
    item: ShowcaseItemKind,
    icon: CarbonIcon,
    quantity: Option<&'static str>,
    selected: bool,
) -> impl Scene {
    bsn! {
        @item_tile_optional(icon, item.title(), quantity, selected)
        ShowcaseItem(item)
        on(move |_activate: On<Activate>,
            mut items: Query<(&ShowcaseItem, &mut CarbonSelection)>,
            mut details: Query<(&ShowcaseDetailField, &mut Text), Without<ShowcaseFeedback>>,
            mut feedback: Query<&mut Text, With<ShowcaseFeedback>>| {
            for (tile, mut selection) in &mut items {
                selection.selected = tile.0 == item;
            }

            for (field, mut text) in &mut details {
                text.0 = match field {
                    ShowcaseDetailField::Icon => item.icon().glyph(),
                    ShowcaseDetailField::Title => item.title(),
                    ShowcaseDetailField::Category => item.category(),
                    ShowcaseDetailField::Description => item.description(),
                    ShowcaseDetailField::Value => item.value(),
                }.into();
            }

            for mut text in &mut feedback {
                text.0 = format!("Selected {}.", item.title());
            }
        })
    }
}

fn equipment_page() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_width: px(0),
            min_height: px(0),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            row_gap: px(13),
        }
        ShowcaseContent(ShowcasePage::Equipment)
        Children [
            @panel(percent(100))
            Children [
                @section_header(CarbonIcon::Shield, "Equipped gear")
                --
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(8),
                    row_gap: px(8),
                }
                Children [
                    @item_tile_optional(CarbonIcon::Shield, "Round shield", None, false)
                    --
                    @item_tile_optional(CarbonIcon::Sword, "Iron longsword", None, false)
                    --
                    @item_tile_optional(CarbonIcon::Anvil, "Iron helmet", None, false)
                ]
            ]
            --
            @panel(percent(100))
            Children [
                @section_header(CarbonIcon::Health, "Character bonuses")
                --
                @stat_bar(
                    CarbonIcon::Shield,
                    "Damage reduction",
                    "18%",
                    0.18,
                    CarbonStatus::Neutral
                )
                --
                @stat_bar(
                    CarbonIcon::Health,
                    "Maximum health",
                    "+20",
                    0.2,
                    CarbonStatus::Positive
                )
                --
                @stat_bar(
                    CarbonIcon::Stamina,
                    "Stamina recovery",
                    "+8%",
                    0.08,
                    CarbonStatus::Positive
                )
            ]
        ]
    }
}

fn materials_page() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_width: px(0),
            min_height: px(0),
            display: Display::None,
            flex_direction: FlexDirection::Column,
            row_gap: px(13),
        }
        ShowcaseContent(ShowcasePage::Materials)
        Children [
            @panel(percent(100))
            Children [
                @section_header(CarbonIcon::Anvil, "Crafting materials")
                --
                Node {
                    width: percent(100),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(8),
                    row_gap: px(8),
                }
                Children [
                    @item_tile_optional(CarbonIcon::Anvil, "Iron ore", Some("12"), false)
                    --
                    @item_tile_optional(CarbonIcon::Shield, "Leather", Some("8"), false)
                    --
                    @item_tile_optional(CarbonIcon::Flask, "Herbs", Some("5"), false)
                    --
                    @item_tile_optional(CarbonIcon::Coin, "Crowns", Some("312"), false)
                ]
            ]
            --
            @panel(percent(100))
            Children [
                @section_header(CarbonIcon::Book, "Storage notes")
                --
                Node {
                    width: percent(100),
                    min_width: px(0),
                }
                Text("Materials are grouped by crafting use. Select a material in a full implementation to see recipes that require it.")
                TextFont { font_size: px(12) }
                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
            ]
        ]
    }
}
