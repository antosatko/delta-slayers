pub mod state;

use bevy::{
    feathers::{dark_theme::create_dark_theme, theme::UiTheme, FeathersPlugins},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_skein::SkeinPlugin;
use delta_slayers_ui::CarbonPlugin;

use crate::{gameplay::GameplayPlugin, ui::UiPlugin};

/// Builds and runs the Bevy application.
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, maximize_primary_window)
        .add_plugins(SkeinPlugin::default())
        .insert_resource(ClearColor(Color::srgb(0.53, 0.78, 0.98)))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 400.0,
            ..default()
        })
        .init_state::<state::AppState>()
        .insert_resource(UiTheme(create_dark_theme()))
        .add_plugins((FeathersPlugins, CarbonPlugin, UiPlugin, GameplayPlugin))
        .run();
}

fn maximize_primary_window(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    window.set_maximized(true);
}
