use bevy::prelude::*;

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct CursorGrabbed(pub bool);

#[derive(Resource, Default)]
pub struct Paused(pub bool);

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    Continue,
    Load,
    Settings,
    Credits,
    InGame,
}