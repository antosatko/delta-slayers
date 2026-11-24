use bevy::prelude::*;

use crate::zone::ZoneKind;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiLayer {
    Pause,
    Panel(ZoneKind),
    Inventory,
}

#[derive(Resource, Default)]
pub struct UiStack(pub Vec<UiLayer>);

impl UiStack {
    pub fn contains(&self, layer: UiLayer) -> bool {
        self.0.contains(&layer)
    }

    pub fn inventory_open(&self) -> bool {
        self.contains(UiLayer::Inventory)
    }

    pub fn paused(&self) -> bool {
        self.contains(UiLayer::Pause)
    }

    pub fn active_panel(&self) -> Option<ZoneKind> {
        self.0.iter().rev().find_map(|l| match l {
            UiLayer::Panel(k) => Some(*k),
            _ => None,
        })
    }

    pub fn push(&mut self, layer: UiLayer) {
        if !self.contains(layer) {
            self.0.push(layer);
        }
    }

    pub fn pop(&mut self) -> Option<UiLayer> {
        self.0.pop()
    }

    pub fn pop_top(&mut self) {
        self.0.pop();
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}