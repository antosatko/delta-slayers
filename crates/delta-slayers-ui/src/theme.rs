use bevy::{prelude::*, ui::Pressed};

/// Carbon design tokens shared by every Carbon UI scene.
///
/// Use neutral surface steps for ordinary hierarchy. Azure is reserved for
/// selected controls, links, focus, and primary actions; status colours are
/// reserved for actual status information.
#[derive(Resource, Clone, Debug)]
pub struct CarbonTheme {
    pub page: Color,
    pub panel: Color,
    pub panel_alt: Color,
    pub panel_raised: Color,
    pub surface_deep: Color,
    pub ink: Color,
    pub muted: Color,
    pub quiet: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub line: Color,
    pub line_strong: Color,
    pub positive: Color,
    pub positive_background: Color,
    pub warning: Color,
    pub warning_background: Color,
    pub danger: Color,
    pub danger_background: Color,
    pub overlay: Color,
}

impl Default for CarbonTheme {
    fn default() -> Self {
        Self {
            page: Color::srgb(0.086, 0.098, 0.118),             // #16191e
            panel: Color::srgb(0.133, 0.157, 0.192),          // #222831
            panel_alt: Color::srgb(0.169, 0.204, 0.251),      // #2b3440
            panel_raised: Color::srgb(0.224, 0.271, 0.329),   // #394554
            surface_deep: Color::srgb(0.114, 0.145, 0.180),    // #1d252e
            ink: Color::srgb(0.945, 0.957, 0.969),             // #f1f4f7
            muted: Color::srgb(0.780, 0.820, 0.863),           // #c7d1dc
            quiet: Color::srgb(0.627, 0.686, 0.741),           // #a0afbd
            accent: Color::srgb(0.467, 0.718, 1.000),           // #77b7ff
            accent_hover: Color::srgb(0.639, 0.816, 1.000),     // #a3d0ff
            line: Color::srgb(0.227, 0.275, 0.329),             // #3a4654
            line_strong: Color::srgb(0.333, 0.396, 0.459),      // #556575
            positive: Color::srgb(0.416, 0.851, 0.639),         // #6ad9a3
            positive_background: Color::srgba(0.416, 0.851, 0.639, 0.10),
            warning: Color::srgb(1.000, 0.820, 0.400),          // #ffd166
            warning_background: Color::srgba(1.000, 0.820, 0.400, 0.10),
            danger: Color::srgb(1.000, 0.490, 0.533),            // #ff7d88
            danger_background: Color::srgba(1.000, 0.490, 0.533, 0.10),
            overlay: Color::srgba(0.035, 0.045, 0.065, 0.72),
        }
    }
}

impl CarbonTheme {
    pub(crate) fn button_background(
        &self,
        tone: CarbonButtonTone,
        hovered: bool,
        pressed: bool,
    ) -> Color {
        match tone {
            CarbonButtonTone::Primary if hovered || pressed => self.accent_hover,
            CarbonButtonTone::Primary => self.accent,
            CarbonButtonTone::Secondary if hovered || pressed => self.panel_raised,
            CarbonButtonTone::Secondary => self.panel_alt,
            CarbonButtonTone::Ghost if hovered || pressed => self.panel_alt,
            CarbonButtonTone::Ghost => Color::NONE,
            CarbonButtonTone::Danger if hovered || pressed => self.danger_background,
            CarbonButtonTone::Danger => Color::NONE,
        }
    }

    pub(crate) fn button_foreground(&self, tone: CarbonButtonTone) -> Color {
        match tone {
            CarbonButtonTone::Primary => Color::srgb(0.086, 0.098, 0.118),
            CarbonButtonTone::Secondary => self.ink,
            CarbonButtonTone::Ghost => self.muted,
            CarbonButtonTone::Danger => self.danger,
        }
    }

    pub(crate) fn status_color(&self, status: CarbonStatus) -> Color {
        match status {
            CarbonStatus::Neutral => self.muted,
            CarbonStatus::Positive => self.positive,
            CarbonStatus::Warning => self.warning,
            CarbonStatus::Danger => self.danger,
        }
    }

    pub(crate) fn status_background(&self, status: CarbonStatus) -> Color {
        match status {
            CarbonStatus::Neutral => self.panel_alt,
            CarbonStatus::Positive => self.positive_background,
            CarbonStatus::Warning => self.warning_background,
            CarbonStatus::Danger => self.danger_background,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarbonButtonTone {
    Primary,
    #[default]
    Secondary,
    Ghost,
    Danger,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarbonStatus {
    #[default]
    Neutral,
    Positive,
    Warning,
    Danger,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct CarbonButtonStyle {
    pub tone: CarbonButtonTone,
    pub selected: bool,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct CarbonHoverDelay {
    pub delay_seconds: f32,
    pub elapsed_seconds: f32,
}

#[derive(Component, Default, Clone, Copy, Debug)]
pub(crate) struct CarbonHoverCard;

pub struct CarbonPlugin;

impl Plugin for CarbonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CarbonTheme>()
            .add_systems(Update, (update_button_styles, update_reference_cards));
    }
}

fn update_button_styles(
    theme: Res<CarbonTheme>,
    mut buttons: Query<(
        &bevy::picking::hover::Hovered,
        Option<&Pressed>,
        &CarbonButtonStyle,
        &mut BackgroundColor,
    )>,
) {
    for (hovered, pressed, style, mut background) in &mut buttons {
        *background = BackgroundColor(if style.selected {
            theme.panel_raised
        } else {
            theme.button_background(style.tone, hovered.get(), pressed.is_some())
        });
    }
}

fn update_reference_cards(
    time: Res<Time>,
    mut triggers: Query<(
        &bevy::picking::hover::Hovered,
        &mut CarbonHoverDelay,
        &Children,
    )>,
    mut cards: Query<(&mut Visibility, &bevy::picking::hover::Hovered), With<CarbonHoverCard>>,
) {
    for (hovered, mut delay, children) in &mut triggers {
        let card_hovered = children.iter().any(|child| {
            cards
                .get(*child)
                .map_or(false, |(_, card_hover)| card_hover.get())
        });

        if hovered.get() {
            delay.elapsed_seconds =
                (delay.elapsed_seconds + time.delta_secs()).min(delay.delay_seconds);
        } else if !card_hovered {
            delay.elapsed_seconds = 0.0;
        }

        let visible = (hovered.get() && delay.elapsed_seconds >= delay.delay_seconds)
            || card_hovered;
        let visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        for child in children {
            if let Ok((mut card_visibility, _)) = cards.get_mut(*child) {
                *card_visibility = visibility;
            }
        }
    }
}
