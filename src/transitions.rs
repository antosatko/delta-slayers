use bevy::{
    prelude::*,
    ui_widgets::{Activate, Button},
};

use crate::state::AppState;

pub struct TransitionsPlugin;

impl Plugin for TransitionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Continue), setup_continue)
            .add_systems(OnEnter(AppState::Load), setup_load)
            .add_systems(OnEnter(AppState::Settings), setup_settings)
            .add_systems(OnEnter(AppState::Credits), setup_credits);
    }
}

fn setup_continue(mut commands: Commands) {
    spawn_screen(&mut commands, "Continue", AppState::Continue);
}

fn setup_load(mut commands: Commands) {
    spawn_screen(&mut commands, "Load Game", AppState::Load);
}

fn setup_settings(mut commands: Commands) {
    spawn_screen(&mut commands, "Settings", AppState::Settings);
}

fn setup_credits(mut commands: Commands) {
    spawn_screen(&mut commands, "Credits", AppState::Credits);
}

fn spawn_screen(commands: &mut Commands, title: &'static str, state: AppState) {
    commands.spawn((Camera2d, DespawnOnExit::<AppState>(state)));
    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(16),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.2, 0.35)),
            DespawnOnExit::<AppState>(state),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: px(48).into(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Placeholder screen. Back to return."),
                TextFont {
                    font_size: px(18).into(),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(px(20), px(10)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.95, 0.55, 0.2)),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Back"),
                        TextFont {
                            font_size: px(20).into(),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                })
                .observe(|_a: On<Activate>, mut next: ResMut<NextState<AppState>>| {
                    next.set(AppState::MainMenu);
                });
        });
}