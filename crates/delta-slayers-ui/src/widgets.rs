use bevy::{picking::hover::Hovered, prelude::*, ui_widgets::Button};

use crate::{
    CarbonButtonTone, CarbonSelection, CarbonStatus, CarbonTheme,
    theme::{CarbonButtonStyle, CarbonHoverCard, CarbonHoverDelay, CarbonTab},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarbonIcon {
    #[default]
    Contracts,
    Inventory,
    Character,
    Journal,
    Map,
    Trade,
    Workshop,
    Settings,
    Search,
    Add,
    Close,
    Back,
    Confirm,
    Coin,
    Health,
    Shield,
    Stamina,
    Sword,
    Compass,
    Book,
    Flask,
    Anvil,
}

impl CarbonIcon {
    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Contracts => "▤",
            Self::Inventory => "▦",
            Self::Character => "♙",
            Self::Journal => "▣",
            Self::Map => "⌖",
            Self::Trade => "⇄",
            Self::Workshop => "⚒",
            Self::Settings => "⚙",
            Self::Search => "⌕",
            Self::Add => "+",
            Self::Close => "×",
            Self::Back => "←",
            Self::Confirm => "✓",
            Self::Coin => "◈",
            Self::Health => "♥",
            Self::Shield => "⬡",
            Self::Stamina => "◉",
            Self::Sword => "⚔",
            Self::Compass => "⌖",
            Self::Book => "▣",
            Self::Flask => "◒",
            Self::Anvil => "⚒",
        }
    }
}

#[derive(Clone, Copy)]
enum GlyphTone {
    Muted,
    Button(CarbonButtonTone),
    Item(bool),
}

/// Full-window layout root with the Carbon page surface.
pub fn root() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            min_width: px(0),
            min_height: px(0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            column_gap: px(16),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().page)))
    }
}

/// Raised content surface. Group content with spacing instead of outlining it.
pub fn panel(width: Val) -> impl Scene {
    bsn! {
        Node {
            width,
            min_width: px(0),
            padding: UiRect::all(px(15)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: px(12),
            border_radius: BorderRadius::all(px(8)),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().panel)))
    }
}

/// One-line screen title with a delayed help card. The optional second row is
/// composed by placing screen_tabs() and screen_tab() directly below it.
pub fn page_header(
    title: &'static str,
    help_title: &'static str,
    help_description: &'static str,
) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            padding: UiRect::bottom(px(8)),
        }
        Children [
            Node {
                width: percent(100),
                min_height: px(34),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(12),
            }
            Children [
                Text(title)
                Node {
                    flex_grow: 1.0,
                    min_width: px(0),
                }
                TextFont { font_size: px(28) }
                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().ink)))
                --
                Button
                Hovered(false)
                CarbonButtonStyle { tone: CarbonButtonTone::Secondary, selected: false }
                CarbonHoverDelay { delay_seconds: 0.6, elapsed_seconds: 0.0 }
                Node {
                    width: px(28),
                    height: px(28),
                    flex_shrink: 0.0,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(px(14)),
                }
                template(|ctx| {
                    let theme = ctx.resource::<CarbonTheme>();
                    Ok(BackgroundColor(theme.button_background(
                        CarbonButtonTone::Secondary,
                        false,
                        false,
                    )))
                })
                Children [
                    Text("?")
                    TextFont { font_size: px(14) }
                    template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
                    --
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(34),
                        right: px(0),
                        width: px(300),
                        padding: UiRect::all(px(14)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(7),
                        border_radius: BorderRadius::all(px(8)),
                    }
                    Hovered(false)
                    CarbonHoverCard
                    bevy::ui::GlobalZIndex(100)
                    Visibility::Hidden
                    template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().panel_raised)))
                    Children [
                        Text(help_title)
                        TextFont { font_size: px(15) }
                        template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().ink)))
                        --
                        Text(help_description)
                        TextFont { font_size: px(12) }
                        template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
                    ]
                ]
            ]
        ]
    }
}

/// Container for a screen's optional secondary navigation row.
pub fn screen_tabs() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            min_height: px(32),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(4),
        }
    }
}

/// A tab label for the second row below a page title. Attach the screen's
/// navigation observer to this scene at the call site.
pub fn screen_tab(label: &'static str, selected: bool) -> impl Scene {
    bsn! {
        Button
        Hovered(false)
        CarbonTab { selected }
        CarbonButtonStyle { tone: CarbonButtonTone::Ghost, selected: false }
        Node {
            min_height: px(31),
            padding: UiRect::axes(px(11), px(5)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(5)),
        }
        template(|ctx| {
            let theme = ctx.resource::<CarbonTheme>();
            Ok(BackgroundColor(theme.button_background(
                CarbonButtonTone::Ghost,
                false,
                false,
            )))
        })
        Children [
            Text(label)
            TextFont { font_size: px(12) }
            template(move |ctx| {
                let theme = ctx.resource::<CarbonTheme>();
                Ok(TextColor(if selected { theme.accent } else { theme.muted }))
            })
        ]
    }
}

/// Compact section title. Icons stay neutral until they communicate state.
pub fn section_header(icon: CarbonIcon, title: &'static str) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            min_height: px(24),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8),
        }
        Children [
            @glyph(icon, 15.0, GlyphTone::Muted)
            --
            Text(title)
            TextFont { font_size: px(15) }
            template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().ink)))
        ]
    }
}

/// Primary, secondary, quiet, and destructive actions share one control shape.
pub fn action_button(label: &'static str, icon: CarbonIcon, tone: CarbonButtonTone) -> impl Scene {
    bsn! {
        Button
        Hovered(false)
        CarbonButtonStyle { tone, selected: false }
        Node {
            min_height: px(36),
            padding: UiRect::axes(px(12), px(7)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(8),
            border_radius: BorderRadius::all(px(6)),
        }
        template(move |ctx| {
            let theme = ctx.resource::<CarbonTheme>();
            Ok(BackgroundColor(theme.button_background(tone, false, false)))
        })
        Children [
            @glyph(icon, 15.0, GlyphTone::Button(tone))
            --
            Text(label)
            TextFont { font_size: px(13) }
            template(move |ctx| {
                Ok(TextColor(
                    ctx.resource::<CarbonTheme>().button_foreground(tone),
                ))
            })
        ]
    }
}


/// Compact secondary copy for descriptions and helper notes.
pub fn muted_text(value: &'static str, size: f32) -> impl Scene {
    bsn! {
        Text(value)
        TextFont { font_size: px(size) }
        template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
    }
}

/// Small category/eyebrow label used above primary content.
pub fn eyebrow_text(value: &'static str) -> impl Scene {
    bsn! {
        Text(value)
        TextFont { font_size: px(10) }
        template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().quiet)))
    }
}

/// Keyboard hint with the key separated from its explanation.
pub fn key_hint(key: &'static str, label: &'static str) -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8),
        }
        Children [
            @badge(key, CarbonStatus::Neutral)
            --
            @muted_text(label, 12.0)
        ]
    }
}

/// A square image-first inventory tile. No popup is opened by selecting it.
pub fn item_art_tile(
    asset_path: &'static str,
    label: &'static str,
    quantity: Option<&'static str>,
) -> impl Scene {
    let quantity = quantity.unwrap_or("");
    bsn! {
        Node {
            width: px(112),
            height: px(112),
            min_width: px(0),
            position_type: PositionType::Relative,
            overflow: Overflow::clip(),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(7)),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().panel_alt)))
        Children [
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                min_height: px(0),
            }
            template(move |ctx| {
                Ok(ImageNode::new(ctx.resource::<AssetServer>().load(asset_path)))
            })
            --
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                bottom: px(0),
                width: percent(100),
                min_width: px(0),
                padding: UiRect::axes(px(7), px(6)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                row_gap: px(2),
            }
            template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().surface_deep)))
            Children [
                Text(label)
                TextFont { font_size: px(11) }
                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().ink)))
                --
                Text(quantity)
                TextFont { font_size: px(10) }
                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().quiet)))
            ]
        ]
    }
}

/// Empty equipment slot with a neutral icon, without a text caption.
pub fn equipment_slot(icon: CarbonIcon) -> impl Scene {
    bsn! {
        Node {
            width: px(58),
            height: px(58),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(7)),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().panel_alt)))
        Children [
            @glyph(icon, 22.0, GlyphTone::Muted)
        ]
    }
}

/// Small semantic label with a restrained tinted surface and no outline.
pub fn badge(label: &'static str, status: CarbonStatus) -> impl Scene {
    bsn! {
        Node {
            min_height: px(22),
            padding: UiRect::axes(px(8), px(3)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(5)),
        }
        template(move |ctx| {
            Ok(BackgroundColor(
                ctx.resource::<CarbonTheme>().status_background(status),
            ))
        })
        Children [
            Text(label)
            TextFont { font_size: px(10) }
            template(move |ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().status_color(status))))
        ]
    }
}

/// Label, current value, and semantic progress fill.
pub fn stat_bar(
    icon: CarbonIcon,
    label: &'static str,
    value: &'static str,
    fraction: f32,
    status: CarbonStatus,
) -> impl Scene {
    let fraction = fraction.clamp(0.0, 1.0);
    bsn! {
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
        }
        Children [
            Node {
                width: percent(100),
                min_width: px(0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(7),
            }
            Children [
                @glyph(icon, 13.0, GlyphTone::Muted)
                --
                Text(label)
                TextFont { font_size: px(12) }
                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
                --
                Node {
                    flex_grow: 1.0,
                    min_width: px(0),
                }
                --
                Text(value)
                TextFont { font_size: px(12) }
                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().ink)))
            ]
            --
            Node {
                width: percent(100),
                height: px(6),
                overflow: Overflow::clip(),
                border_radius: BorderRadius::all(px(3)),
            }
            template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().surface_deep)))
            Children [
                Node {
                    width: percent(fraction * 100.0),
                    height: percent(100),
                    border_radius: BorderRadius::all(px(3)),
                }
                template(move |ctx| {
                    Ok(BackgroundColor(
                        ctx.resource::<CarbonTheme>().status_color(status),
                    ))
                })
            ]
        ]
    }
}

/// Square, artwork-first item control. Pass None when the item is not stacked.
pub fn item_tile(
    icon: CarbonIcon,
    label: &'static str,
    quantity: &'static str,
    selected: bool,
) -> impl Scene {
    item_tile_scene(icon, label, quantity, selected)
}

/// Optional-quantity variant for items that may not be stackable.
pub fn item_tile_optional(
    icon: CarbonIcon,
    label: &'static str,
    quantity: Option<&'static str>,
    selected: bool,
) -> impl Scene {
    item_tile_scene(icon, label, quantity.unwrap_or(""), selected)
}

fn item_tile_scene(
    icon: CarbonIcon,
    label: &'static str,
    quantity: &'static str,
    selected: bool,
) -> impl Scene {
    bsn! {
        Button
        Hovered(false)
        CarbonButtonStyle { tone: CarbonButtonTone::Secondary, selected }
        CarbonSelection { selected }
        Node {
            width: px(112),
            height: px(112),
            min_width: px(0),
            position_type: PositionType::Relative,
            overflow: Overflow::clip(),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(7)),
        }
        template(move |ctx| Ok(BackgroundColor(if selected {
            ctx.resource::<CarbonTheme>().panel_raised
        } else {
            ctx.resource::<CarbonTheme>().panel_alt
        })))
        Children [
            @glyph(icon, 31.0, GlyphTone::Item(selected))
            --
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                bottom: px(0),
                width: percent(100),
                min_width: px(0),
                padding: UiRect::axes(px(7), px(6)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                row_gap: px(2),
            }
            template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().surface_deep)))
            Children [
                Text(label)
                TextFont { font_size: px(11) }
                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().ink)))
                --
                Text(quantity)
                TextFont { font_size: px(10) }
                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().quiet)))
            ]
        ]
    }
}

/// Inline linked term with a delayed information card. Moving onto the card
/// keeps it visible; leaving the trigger resets its dwell timer.
pub fn reference_term(
    label: &'static str,
    title: &'static str,
    category: &'static str,
    description: &'static str,
) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
        }
        Hovered(false)
        CarbonHoverDelay { delay_seconds: 0.6, elapsed_seconds: 0.0 }
        Children [
            Text(label)
            TextFont { font_size: px(13) }
            template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().accent)))
            --
            Node {
                position_type: PositionType::Absolute,
                top: percent(100),
                left: px(0),
                width: px(320),
                padding: UiRect::all(px(14)),
                flex_direction: FlexDirection::Column,
                row_gap: px(7),
                border_radius: BorderRadius::all(px(8)),
            }
            Hovered(false)
            CarbonHoverCard
            bevy::ui::GlobalZIndex(100)
            Visibility::Hidden
            template(|ctx| Ok(BackgroundColor(ctx.resource::<CarbonTheme>().panel_raised)))
            Children [
                Text(category)
                TextFont { font_size: px(10) }
                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().quiet)))
                --
                Text(title)
                TextFont { font_size: px(16) }
                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().ink)))
                --
                Text(description)
                TextFont { font_size: px(12) }
                template(|ctx| Ok(TextColor(ctx.resource::<CarbonTheme>().muted)))
            ]
        ]
    }
}

fn glyph(icon: CarbonIcon, size: f32, tone: GlyphTone) -> impl Scene {
    let glyph = icon.glyph();
    bsn! {
        Text(glyph)
        TextFont { font_size: px(size) }
        template(move |ctx| {
            let theme = ctx.resource::<CarbonTheme>();
            let color = match tone {
                GlyphTone::Muted => theme.quiet,
                GlyphTone::Button(button_tone) => theme.button_foreground(button_tone),
                GlyphTone::Item(selected) => {
                    if selected { theme.accent } else { theme.muted }
                }
            };
            Ok(TextColor(color))
        })
    }
}
