use bevy::prelude::*;

use crate::gameplay::zones::ZoneKind;

/// Overlay navigation is the single source of truth for in-game UI state.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiLayer {
    Pause,
    Panel(ZoneKind),
    Inventory,
    ExplorationSetup,
    GameMenu,
    Settings,
}

#[derive(Resource, Debug, Default)]
pub struct UiStack {
    layers: Vec<UiLayer>,
}

impl UiStack {
    pub fn contains(&self, layer: UiLayer) -> bool {
        self.layers.contains(&layer)
    }

    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    pub fn inventory_open(&self) -> bool {
        self.contains(UiLayer::Inventory)
    }

    pub fn exploration_setup_open(&self) -> bool {
        self.contains(UiLayer::ExplorationSetup)
    }

    pub fn game_menu_open(&self) -> bool {
        self.contains(UiLayer::GameMenu)
    }

    pub fn paused(&self) -> bool {
        self.contains(UiLayer::Pause)
    }

    pub fn active_panel(&self) -> Option<ZoneKind> {
        self.layers.iter().rev().find_map(|layer| match layer {
            UiLayer::Panel(kind) => Some(*kind),
            _ => None,
        })
    }

    pub fn push(&mut self, layer: UiLayer) {
        if !self.contains(layer) {
            self.layers.push(layer);
        }
    }

    pub fn pop_top(&mut self) -> Option<UiLayer> {
        self.layers.pop()
    }

    pub fn remove(&mut self, layer: UiLayer) -> bool {
        let Some(index) = self.layers.iter().rposition(|current| *current == layer) else {
            return false;
        };
        self.layers.remove(index);
        true
    }

    pub fn reset(&mut self) {
        self.layers.clear();
    }

    pub fn toggle_pause(&mut self) {
        if self.paused() {
            self.remove(UiLayer::Pause);
        } else {
            self.push(UiLayer::Pause);
        }
    }
}
