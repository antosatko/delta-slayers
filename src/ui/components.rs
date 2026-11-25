use bevy::prelude::*;

use crate::{
    gameplay::{exploration::ExplorationPortal, zones::ZoneKind},
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
        let authored = authored.sanitized();
        self.authored = authored;
        self.config = authored;
    }

    pub(super) fn reset(&mut self) {
        self.config = self.authored;
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
    VegetationDensity,
}

#[derive(Component, Default, Clone, Copy)]
pub(super) struct ExplorationSetupRoot;

#[derive(Component, Default, Clone, Copy)]
pub(super) struct ExplorationSetupTileCountText;
