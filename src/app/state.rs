use bevy::state::state::States;

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    Load,
    Settings,
    Credits,
    Loading,
    LoadingPreview,
    InGame,
}
