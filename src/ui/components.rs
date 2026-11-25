use bevy::prelude::*;

use crate::gameplay::{
    exploration::{ExplorationPortal, generate_seed},
    zones::ZoneKind,
};

#[derive(Component, Default, Clone)]
pub(super) struct PanelRoot(pub(super) ZoneKind);
#[derive(Component, Default, Clone)]
pub(super) struct PopupRoot;
#[derive(Component, Default, Clone)]
pub(super) struct FastTravelButton(pub(super) ZoneKind);
#[derive(Component, Default, Clone)]
pub(super) struct FastTravelMenu;
#[derive(Component, Default, Clone)]
pub(super) struct GameMenuModal;
#[derive(Component, Default, Clone)]
pub(super) struct PauseOverlay;
#[derive(Component, Default, Clone)]
pub(super) struct MinimapZoneDot(pub(super) ZoneKind);

#[derive(Component, Clone, Copy, Default)]
pub(super) struct MinimapExplorationDot(pub(super) i32, pub(super) i32);

#[derive(Resource, Default)]
pub(super) struct PopupConfirmRequested(pub(super) bool);

#[derive(Resource, Default)]
pub(super) struct ExplorationDebugPanelState {
    pub(super) visible: bool,
}

#[derive(Component, Default, Clone)]
pub(super) struct ExplorationDebugPanelRoot;

#[derive(Component, Default, Clone)]
pub(super) struct ExplorationDebugInfoText;

#[derive(Resource, Clone, Copy, Debug)]
pub(super) struct ExplorationSetupViewModel {
    pub(super) authored: ExplorationPortal,
    pub(super) config: ExplorationPortal,
}

impl Default for ExplorationSetupViewModel {
    fn default() -> Self {
        let config = ExplorationPortal::default();
        Self {
            authored: config,
            config,
        }
    }
}

impl ExplorationSetupViewModel {
    pub(super) fn open(&mut self, authored: ExplorationPortal) {
        let mut authored = authored.sanitized();
        authored.seed = authored
            .seed
            .map(|seed| seed.min(i64::MAX as u64))
            .filter(|seed| *seed != 0);
        self.authored = authored;
        self.config = authored;
    }

    pub(super) fn reset(&mut self) {
        self.config = self.authored;
    }

    pub(super) fn randomize_seed(&mut self) {
        self.config.seed = Some(generate_seed().clamp(1, i64::MAX as u64));
    }
}

#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ExplorationParameterControl {
    #[default]
    Seed,
    Radius,
    TileSize,
    HeightMultiplier,
    TerrainFrequency,
    BiomeScale,
    MaxJumpHeight,
}

#[derive(Component, Default, Clone, Copy)]
pub(super) struct ExplorationSetupRoot;

#[derive(Component, Default, Clone, Copy)]
pub(super) struct ExplorationSetupTileCountText;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exploration_settings_reset_to_the_authored_portal_values() {
        let mut view_model = ExplorationSetupViewModel::default();
        let mut authored = ExplorationPortal::default();
        authored.radius = 18;
        authored.seed = Some(42);
        view_model.open(authored);

        view_model.config.radius = 32;
        view_model.config.seed = Some(123);
        view_model.reset();

        assert_eq!(view_model.config.radius, 18);
        assert_eq!(view_model.config.seed, Some(42));
    }

    #[test]
    fn randomize_seed_selects_a_visible_reproducible_seed() {
        let mut view_model = ExplorationSetupViewModel::default();
        view_model.randomize_seed();

        assert!(
            view_model
                .config
                .seed
                .is_some_and(|seed| seed > 0 && seed <= i64::MAX as u64)
        );
    }

    #[test]
    fn zero_seed_is_treated_as_random() {
        let mut view_model = ExplorationSetupViewModel::default();
        let mut authored = ExplorationPortal::default();
        authored.seed = Some(0);
        view_model.open(authored);

        assert_eq!(view_model.config.seed, None);
    }
}
