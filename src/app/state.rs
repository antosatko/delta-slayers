use bevy::state::state::States;

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    Continue,
    Load,
    Settings,
    Credits,
    Loading,
    InGame,
}
