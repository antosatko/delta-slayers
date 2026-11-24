use bevy::{
    prelude::*,
    ui_widgets::{Activate, Button},
};

use crate::state::AppState;

pub struct TransitionsPlugin;

impl Plugin for TransitionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Continue),
            continue_screen.spawn(),
        )
        .add_systems(
            OnEnter(AppState::Load),
            load_screen.spawn(),
        )
        .add_systems(
            OnEnter(AppState::Settings),
            settings_screen.spawn(),
        )
        .add_systems(
            OnEnter(AppState::Credits),
            credits_screen.spawn(),
        );
    }
}

fn continue_screen() -> impl SceneList {
    placeholder_screen(AppState::Continue, "Continue")
}

fn load_screen() -> impl SceneList {
    placeholder_screen(AppState::Load, "Load Game")
}

fn settings_screen() -> impl SceneList {
    placeholder_screen(AppState::Settings, "Settings")
}

fn credits_screen() -> impl SceneList {
    placeholder_screen(AppState::Credits, "Credits")
}

fn placeholder_screen(state: AppState, title: &'static str) -> impl SceneList {
    bsn_list! {
        Camera2d
        DespawnOnExit::<AppState>(state)
        --
        @placeholder_content(title, state)
    }
}

fn placeholder_content(title: &'static str, state: AppState) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(16),
        }
        BackgroundColor(Color::srgb(0.15, 0.2, 0.35))
        DespawnOnExit::<AppState>(state)
        Children [
            Text(title)
            TextFont {
                font_size: px(48),
            }
            TextColor(Color::WHITE)
            --
            Text("Placeholder screen. Back to return.")
            TextFont {
                font_size: px(18),
            }
            TextColor(Color::srgb(0.7, 0.7, 0.7))
            --
            Button
            Node {
                padding: UiRect::axes(px(20), px(10)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            }
            BackgroundColor(Color::srgb(0.95, 0.55, 0.2))
            on(|_activate: On<Activate>, mut next: ResMut<NextState<AppState>>| {
                next.set(AppState::MainMenu);
            })
            Children [
                Text("Back")
                TextFont {
                    font_size: px(20),
                }
                TextColor(Color::WHITE)
            ]
        ]
    }
}
