use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui_widgets::Button,
};

use crate::{
    SlateButtonTone, SlateStatus, SlateTheme,
    theme::{SlateButtonStyle, SlateHoverCard, SlateHoverDelay},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SlateIcon {
    #[default]
    Contracts, Inventory, Character, Journal, Map, Trade, Workshop, Settings,
    Search, Add, Close, Back, Confirm, Coin, Health, Shield, Stamina, Sword,
    Compass, Book, Flask, Anvil,
}

impl SlateIcon {
    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Contracts => "▤", Self::Inventory => "▦", Self::Character => "♙",
            Self::Journal => "▣", Self::Map => "⌖", Self::Trade => "⇄",
            Self::Workshop => "⚒", Self::Settings => "⚙", Self::Search => "⌕",
            Self::Add => "+", Self::Close => "×", Self::Back => "←",
            Self::Confirm => "✓", Self::Coin => "◈", Self::Health => "♥",
            Self::Shield => "⬡", Self::Stamina => "◉", Self::Sword => "⚔",
            Self::Compass => "⌖", Self::Book => "▣", Self::Flask => "◒",
            Self::Anvil => "⚒",
        }
    }
}

pub fn root() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Row,
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<SlateTheme>().page)))
    }
}

/// Quiet surface. Separation comes from spacing, surface tone, and subtle elevation.
pub fn panel(width: Val) -> impl Scene {
    bsn! {
        Node {
            width,
            padding: UiRect::all(px(16)),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            border_radius: BorderRadius::all(px(7)),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<SlateTheme>().panel)))
        BoxShadow([
            ShadowStyle {
                color: Color::srgba(0.0, 0.0, 0.0, 0.16),
                x_offset: px(0),
                y_offset: px(3),
                spread_radius: px(0),
                blur_radius: px(10),
            }
        ])
    }
}

pub fn page_header(eyebrow: &'static str, title: &'static str, subtitle: &'static str) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(5),
            padding: UiRect::bottom(px(16)),
        }
        Children [
            Text(eyebrow)
            TextFont { font_size: px(11) }
            template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().quiet)))
            --
            Text(title)
            TextFont { font_size: px(30) }
            template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().ink)))
            --
            Text(subtitle)
            TextFont { font_size: px(14) }
            template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().muted)))
        ]
    }
}

pub fn section_header(icon: SlateIcon, title: &'static str) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            min_height: px(26),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8),
        }
        Children [
            @glyph(icon, 17.0)
            --
            Text(title)
            TextFont { font_size: px(16) }
            template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().ink)))
        ]
    }
}

pub fn action_button(label: &'static str, icon: SlateIcon, tone: SlateButtonTone) -> impl Scene {
    bsn! {
        Button
        Hovered(false)
        SlateButtonStyle { tone, selected: false }
        Node {
            min_height: px(38),
            padding: UiRect::axes(px(13), px(7)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(8),
            border_radius: BorderRadius::all(px(5)),
        }
        template(move |ctx| Ok(BackgroundColor(ctx.resource::<SlateTheme>().button_background(tone, false, false))))
        Children [
            @glyph(icon, 16.0)
            --
            Text(label)
            TextFont { font_size: px(14) }
            template(move |ctx| Ok(TextColor(ctx.resource::<SlateTheme>().button_foreground(tone))))
        ]
    }
}

pub fn badge(label: &'static str, status: SlateStatus) -> impl Scene {
    bsn! {
        Node {
            min_height: px(22),
            padding: UiRect::axes(px(7), px(3)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(4)),
        }
        template(|ctx| Ok(BackgroundColor(ctx.resource::<SlateTheme>().panel_alt)))
        Text(label)
        TextFont { font_size: px(10) }
        template(move |ctx| Ok(TextColor(ctx.resource::<SlateTheme>().status_color(status))))
    }
}

pub fn stat_bar(
    icon: SlateIcon,
    label: &'static str,
    value: &'static str,
    fraction: f32,
    status: SlateStatus,
) -> impl Scene {
    let fraction = fraction.clamp(0.0, 1.0);
    bsn! {
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
        }
        Children [
            Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(7) }
            @glyph(icon, 14.0)
            --
            Text(label)
            TextFont { font_size: px(13) }
            template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().muted)))
            --
            Node { flex_grow: 1.0 }
            --
            Text(value)
            TextFont { font_size: px(13) }
            template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().ink)))
            --
            Node {
                width: percent(100),
                height: px(5),
                overflow: Overflow::clip(),
                border_radius: BorderRadius::all(px(3)),
            }
            template(|ctx| Ok(BackgroundColor(ctx.resource::<SlateTheme>().page)))
            Children [
                Node { width: percent(fraction * 100.0), height: percent(100), border_radius: BorderRadius::all(px(3)) }
                template(move |ctx| Ok(BackgroundColor(ctx.resource::<SlateTheme>().status_color(status))))
            ]
        ]
    }
}

pub fn item_tile(icon: SlateIcon, label: &'static str, quantity: &'static str, selected: bool) -> impl Scene {
    bsn! {
        Button
        Hovered(false)
        SlateButtonStyle { tone: SlateButtonTone::Secondary, selected }
        Node {
            width: px(112),
            min_height: px(104),
            padding: UiRect::all(px(9)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::SpaceBetween,
            row_gap: px(7),
            border_radius: BorderRadius::all(px(6)),
        }
        template(move |ctx| {
            let theme = ctx.resource::<SlateTheme>();
            Ok(BackgroundColor(if selected { theme.panel_raised } else { theme.panel_alt }))
        })
        Children [
            @glyph(icon, 23.0)
            --
            Node { width: percent(100), flex_direction: FlexDirection::Column, row_gap: px(3) }
            Children [
                Text(label)
                TextFont { font_size: px(12) }
                template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().ink)))
                --
                Text(quantity)
                TextFont { font_size: px(10) }
                template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().quiet)))
            ]
        ]
    }
}

pub fn reference_term(
    label: &'static str,
    title: &'static str,
    category: &'static str,
    description: &'static str,
) -> impl Scene {
    bsn! {
        Node { position_type: PositionType::Relative, flex_direction: FlexDirection::Row, align_items: AlignItems::Center }
        Hovered(false)
        SlateHoverDelay { delay_seconds: 0.6, elapsed_seconds: 0.0 }
        Children [
            Text(label)
            TextFont { font_size: px(14) }
            template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().accent)))
            --
            Node {
                position_type: PositionType::Absolute,
                top: percent(100),
                left: px(0),
                width: px(350),
                padding: UiRect::all(px(16)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                border_radius: BorderRadius::all(px(7)),
            }
            SlateHoverCard
            bevy::ui::GlobalZIndex(100)
            Visibility::Hidden
            template(|ctx| Ok(BackgroundColor(ctx.resource::<SlateTheme>().panel_raised)))
            BoxShadow([
                ShadowStyle {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.24),
                    x_offset: px(0),
                    y_offset: px(5),
                    spread_radius: px(0),
                    blur_radius: px(16),
                }
            ])
            Children [
                Text(category)
                TextFont { font_size: px(10) }
                template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().quiet)))
                --
                Text(title)
                TextFont { font_size: px(18) }
                template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().ink)))
                --
                Text(description)
                TextFont { font_size: px(13) }
                template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().muted)))
            ]
        ]
    }
}

fn glyph(icon: SlateIcon, size: f32) -> impl Scene {
    let glyph = icon.glyph();
    bsn! {
        Text(glyph)
        TextFont { font_size: px(size) }
        template(|ctx| Ok(TextColor(ctx.resource::<SlateTheme>().accent)))
    }
}
