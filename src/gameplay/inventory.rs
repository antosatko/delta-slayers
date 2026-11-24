use bevy::prelude::*;

/// Authoritative inventory data, independent of its UI representation.
#[derive(Resource, Clone, Debug, Default)]
pub struct InventoryModel {
    pub gold: u32,
    pub wood: u32,
    pub stone: u32,
}
