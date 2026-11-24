mod billboard;
mod game;
mod inventory;
mod menu;
mod player;
mod state;
mod transitions;
mod ui;
mod ui_stack;
mod zone;

use bevy::prelude::*;

use crate::state::AppState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.53, 0.78, 0.98)))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 400.0,
            ..default()
        })
        .init_state::<AppState>()
        .add_plugins((
            menu::MenuPlugin,
            transitions::TransitionsPlugin,
            ui::UiPlugin,
            game::GamePlugin,
        ))
        .run();
}