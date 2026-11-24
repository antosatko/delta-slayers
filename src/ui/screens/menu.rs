use bevy::{prelude::*, ui_widgets::{Activate, Button}};

use crate::app::state::AppState;

pub(in crate::ui) fn scene() -> impl SceneList {
    bsn_list! {
        Camera2d
        DespawnOnExit::<AppState>(AppState::MainMenu)
        --
        @main_menu()
    }
}

fn main_menu() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }

        DespawnOnExit::<AppState>(AppState::MainMenu)

        BackgroundColor(Color::srgb(0.15, 0.2, 0.35))

        Children [
            Text("DELTA SLAYERS")
            TextFont {
                font_size: px(80),
            }
            TextColor(Color::WHITE)

            --

            @menu_button("Play", AppState::Loading)

            --

            @menu_button("Continue", AppState::Continue)

            --

            @menu_button("Load", AppState::Load)

            --

            @menu_button("Settings", AppState::Settings)

            --

            @menu_button("Credits", AppState::Credits)

            --

            @menu_button("Exit", AppState::MainMenu)
            on(|_activate: On<Activate>, mut app_exit: MessageWriter<AppExit>| {
                app_exit.write(AppExit::Success);
            })
        ]
    }
}

fn menu_button(label: &'static str, target: AppState) -> impl Scene {
    bsn! {
        Button

        Node {
            width: px(260),
            margin: UiRect::all(px(6)),
            padding: UiRect::all(px(16)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }

        BackgroundColor(Color::srgb(0.95, 0.55, 0.2))

        on(move |_activate: On<Activate>, mut next_state: ResMut<NextState<AppState>>| {
            info!("Transition: Moving to {:?}", target);
            next_state.set(target);
        })

        Children [
            Text(label)
            TextFont {
                font_size: px(30),
            }
            TextColor(Color::WHITE)
        ]
    }
}
