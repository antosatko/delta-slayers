use bevy::prelude::*;

use crate::app::state::AppState;

/// Shown while gameplay loads the Blender-authored world.
pub(in crate::ui) fn scene() -> impl SceneList {
    bsn_list! {
        Camera2d
        Camera {
            order: 10,
        }
        IsDefaultUiCamera
        DespawnOnExit::<AppState>(AppState::Loading)
        --
        @loading_content()
    }
}

fn loading_content() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(18),
        }
        BackgroundColor(Color::srgb(0.035, 0.045, 0.065))
        DespawnOnExit::<AppState>(AppState::Loading)
        Children [
            Text("DELTA SLAYERS")
            TextFont {
                font_size: px(48),
            }
            TextColor(Color::WHITE)
            --
            Node {
                width: px(72),
                height: px(4),
            }
            BackgroundColor(Color::srgb(0.95, 0.55, 0.2))
            --
            Text("LOADING WORLD")
            TextFont {
                font_size: px(20),
            }
            TextColor(Color::srgb(0.95, 0.65, 0.4))
            --
            Text("Preparing the town...")
            TextFont {
                font_size: px(16),
            }
            TextColor(Color::srgb(0.72, 0.74, 0.78))
        ]
    }
}
