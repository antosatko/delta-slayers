use bevy::prelude::*;

use crate::gameplay::zones::ZoneKind;

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
