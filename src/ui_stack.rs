use bevy::prelude::*;

use crate::zone::ZoneKind;

/// Overlay navigation is the single source of truth for in-game UI state.
///
/// The cursor preference is kept separately because closing the last overlay
/// should restore the gameplay cursor mode rather than infer it from input.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiLayer {
    Pause,
    Panel(ZoneKind),
    Inventory,
    InventoryMenu,
}

#[derive(Resource, Debug)]
pub struct UiStack {
    layers: Vec<UiLayer>,
    capture_requested: bool,
}

impl Default for UiStack {
    fn default() -> Self {
        Self {
            layers: Vec::new(),
            capture_requested: true,
        }
    }
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

    pub fn inventory_menu_open(&self) -> bool {
        self.contains(UiLayer::InventoryMenu)
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
        let popped = self.layers.pop();
        if self.layers.is_empty() && popped.is_some() {
            self.capture_requested = true;
        }
        popped
    }

    pub fn remove(&mut self, layer: UiLayer) -> bool {
        let Some(index) = self.layers.iter().rposition(|current| *current == layer) else {
            return false;
        };

        self.layers.remove(index);
        if self.layers.is_empty() {
            self.capture_requested = true;
        }
        true
    }

    pub fn reset(&mut self) {
        self.layers.clear();
        self.capture_requested = true;
    }

    pub fn cursor_capture_requested(&self) -> bool {
        self.capture_requested
    }

    pub fn cursor_grabbed(&self) -> bool {
        self.layers.is_empty() && self.capture_requested
    }

    pub fn capture_cursor(&mut self) {
        self.capture_requested = true;
    }

    pub fn release_cursor(&mut self) {
        self.capture_requested = false;
    }

    pub fn toggle_pause(&mut self) {
        if self.paused() {
            self.remove(UiLayer::Pause);
        } else {
            self.push(UiLayer::Pause);
        }
    }
}
