use bevy::prelude::*;

/// An item type currently supported by the inventory and pickup authoring API.
#[derive(Reflect, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[reflect(Default)]
#[type_path = "delta_slayers::gameplay::inventory"]
pub enum InventoryItem {
    #[default]
    Gold,
    Wood,
    Stone,
}

/// Authoritative inventory data, independent of its UI representation.
#[derive(Resource, Clone, Debug, Default)]
pub struct InventoryModel {
    pub gold: u32,
    pub wood: u32,
    pub stone: u32,
}

impl InventoryModel {
    /// Adds an amount to the selected inventory stack without overflowing.
    pub fn add_item(&mut self, item: InventoryItem, amount: u32) {
        let stack = match item {
            InventoryItem::Gold => &mut self.gold,
            InventoryItem::Wood => &mut self.wood,
            InventoryItem::Stone => &mut self.stone,
        };
        *stack = stack.saturating_add(amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_items_updates_the_matching_inventory_stack() {
        let mut inventory = InventoryModel::default();

        inventory.add_item(InventoryItem::Wood, 2);
        inventory.add_item(InventoryItem::Gold, 3);
        inventory.add_item(InventoryItem::Stone, 4);

        assert_eq!(inventory.wood, 2);
        assert_eq!(inventory.gold, 3);
        assert_eq!(inventory.stone, 4);
    }

    #[test]
    fn adding_items_saturates_instead_of_overflowing() {
        let mut inventory = InventoryModel {
            gold: u32::MAX,
            ..default()
        };

        inventory.add_item(InventoryItem::Gold, 1);

        assert_eq!(inventory.gold, u32::MAX);
    }
}
