use bevy::{prelude::*, ui::Pressed};

/// Slate palette. Replace this resource before spawning scenes to customise the
/// visual system without changing individual widgets.
#[derive(Resource, Clone, Debug)]
pub struct SlateTheme {
    pub page: Color,
    pub panel: Color,
    pub panel_alt: Color,
    pub panel_raised: Color,
    pub ink: Color,
    pub muted: Color,
    pub quiet: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub line: Color,
    pub line_strong: Color,
    pub positive: Color,
    pub warning: Color,
    pub danger: Color,
}

impl Default for SlateTheme {
    fn default() -> Self {
        Self {
            page: Color::srgb(0.106, 0.125, 0.141),
            panel: Color::srgb(0.141, 0.169, 0.188),
            panel_alt: Color::srgb(0.169, 0.204, 0.227),
            panel_raised: Color::srgb(0.200, 0.243, 0.271),
            ink: Color::srgb(0.953, 0.965, 0.969),
            muted: Color::srgb(0.816, 0.859, 0.878),
            quiet: Color::srgb(0.706, 0.765, 0.792),
            accent: Color::srgb(0.725, 0.820, 0.863),
            accent_hover: Color::srgb(0.816, 0.886, 0.914),
            line: Color::srgb(0.376, 0.451, 0.486),
            line_strong: Color::srgb(0.510, 0.596, 0.635),
            positive: Color::srgb(0.769, 0.867, 0.812),
            warning: Color::srgb(0.878, 0.831, 0.678),
            danger: Color::srgb(0.878, 0.749, 0.753),
        }
    }
}

impl SlateTheme {
    pub(crate) fn button_background(&self, tone: SlateButtonTone, hovered: bool, pressed: bool) -> Color {
        match (tone, pressed, hovered) {
            (SlateButtonTone::Primary, true, _) => self.accent_hover,
            (SlateButtonTone::Primary, false, true) => self.accent_hover,
            (SlateButtonTone::Primary, false, false) => self.accent,
            (SlateButtonTone::Secondary, true, _) => self.panel_raised,
            (SlateButtonTone::Secondary, false, true) => self.panel_raised,
            (SlateButtonTone::Secondary, false, false) => self.panel_alt,
            (SlateButtonTone::Ghost, true, _) => self.panel_raised,
            (SlateButtonTone::Ghost, false, true) => self.panel_alt,
            (SlateButtonTone::Ghost, false, false) => Color::NONE,
            (SlateButtonTone::Danger, true, _) => self.danger,
            (SlateButtonTone::Danger, false, true) => self.danger,
            (SlateButtonTone::Danger, false, false) => self.panel_alt,
        }
    }

    pub(crate) fn button_foreground(&self, tone: SlateButtonTone) -> Color {
        match tone {
            SlateButtonTone::Primary => Color::srgb(0.090, 0.125, 0.145),
            SlateButtonTone::Secondary | SlateButtonTone::Ghost | SlateButtonTone::Danger => self.ink,
        }
    }

    pub(crate) fn status_color(&self, status: SlateStatus) -> Color {
        match status {
            SlateStatus::Neutral => self.muted,
            SlateStatus::Positive => self.positive,
            SlateStatus::Warning => self.warning,
            SlateStatus::Danger => self.danger,
        }
    }
}

/// Visual treatment for action buttons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SlateButtonTone {
    Primary,
    #[default]
    Secondary,
    Ghost,
    Danger,
}

/// Semantic colour for badges and value indicators.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SlateStatus {
    #[default]
    Neutral,
    Positive,
    Warning,
    Danger,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct SlateButtonStyle {
    pub tone: SlateButtonTone,
}

/// Installs the palette and the small behaviours shared by Slate widgets.
pub struct SlatePlugin;

impl Plugin for SlatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SlateTheme>()
            .add_systems(Update, (update_button_styles, update_reference_cards));
    }
}

fn update_button_styles(
    theme: Res<SlateTheme>,
    mut buttons: Query<(
        &bevy::picking::hover::Hovered,
        Option<&Pressed>,
        &SlateButtonStyle,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    for (hovered, pressed, style, mut background, mut border) in &mut buttons {
        *background = BackgroundColor(theme.button_background(
            style.tone,
            hovered.get(),
            pressed.is_some(),
        ));
        *border = BorderColor::all(if hovered.get() {
            theme.accent
        } else {
            theme.line
        });
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct SlateHoverDelay {
    pub delay_seconds: f32,
    pub elapsed_seconds: f32,
}

#[derive(Component, Default, Clone, Copy, Debug)]
pub(crate) struct SlateHoverCard;

fn update_reference_cards(
    time: Res<Time>,
    mut triggers: Query<(
        &bevy::picking::hover::Hovered,
        &mut SlateHoverDelay,
        &Children,
    )>,
    mut cards: Query<&mut Visibility, With<SlateHoverCard>>,
) {
    for (hovered, mut delay, children) in &mut triggers {
        if hovered.get() {
            delay.elapsed_seconds =
                (delay.elapsed_seconds + time.delta_secs()).min(delay.delay_seconds);
        } else {
            delay.elapsed_seconds = 0.0;
        }

        let visibility = if hovered.get() && delay.elapsed_seconds >= delay.delay_seconds {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        for child in children {
            if let Ok(mut card_visibility) = cards.get_mut(*child) {
                *card_visibility = visibility;
            }
        }
    }
}
