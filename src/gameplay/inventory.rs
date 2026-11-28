use bevy::prelude::*;

#[derive(Reflect, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[reflect(Default)]
#[type_path = "delta_slayers::gameplay::inventory"]
pub enum InventoryItem {
    #[default]
    Gold,
    Wood,
    Stone,
}

pub const CAPACITY_SUBUNITS_PER_UNIT: u32 = 100;
pub const DEFAULT_MAX_CAPACITY: u32 = 100 * CAPACITY_SUBUNITS_PER_UNIT;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    Gold,
    Wood,
    Stone,
}

impl ResourceKind {
    pub const ALL: [Self; 3] = [Self::Gold, Self::Wood, Self::Stone];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Gold => "Gold",
            Self::Wood => "Wood",
            Self::Stone => "Stone",
        }
    }

    pub const fn save_key(self) -> &'static str {
        match self {
            Self::Gold => "gold",
            Self::Wood => "wood",
            Self::Stone => "stone",
        }
    }

    pub const fn capacity_subunits_per_unit(self) -> u32 {
        match self {
            // 100 coins occupy one capacity unit.
            Self::Gold => 1,
            Self::Wood => CAPACITY_SUBUNITS_PER_UNIT,
            Self::Stone => 150,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Gold => 0,
            Self::Wood => 1,
            Self::Stone => 2,
        }
    }
}

impl From<InventoryItem> for ResourceKind {
    fn from(item: InventoryItem) -> Self {
        match item {
            InventoryItem::Gold => Self::Gold,
            InventoryItem::Wood => Self::Wood,
            InventoryItem::Stone => Self::Stone,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemKind {
    EvilPup,
    Maurice,
    Salad,
    Scooby,
    Theo,
}

impl ItemKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::EvilPup => "Evil pup",
            Self::Maurice => "Maurice",
            Self::Salad => "Salad",
            Self::Scooby => "Scooby",
            Self::Theo => "Theo",
        }
    }

    pub const fn save_key(self) -> &'static str {
        match self {
            Self::EvilPup => "evil-pup",
            Self::Maurice => "maurice",
            Self::Salad => "salad",
            Self::Scooby => "scooby",
            Self::Theo => "theo",
        }
    }

    pub fn from_save_key(value: &str) -> Option<Self> {
        match value {
            "evil-pup" => Some(Self::EvilPup),
            "maurice" => Some(Self::Maurice),
            "salad" => Some(Self::Salad),
            "scooby" => Some(Self::Scooby),
            "theo" => Some(Self::Theo),
            _ => None,
        }
    }

    pub const fn capacity_subunits(self) -> u32 {
        match self {
            Self::EvilPup => 450,
            Self::Maurice => 300,
            Self::Salad => 100,
            Self::Scooby => 450,
            Self::Theo => 300,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemInstance {
    pub id: u64,
    pub kind: ItemKind,
}

/// Authoritative inventory data. Resources are quantity stacks; items are
/// individual instances with stable IDs so later systems can refer to them.
#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct InventoryModel {
    pub max_capacity_subunits: u32,
    resources: [u32; 3],
    items: Vec<ItemInstance>,
    next_item_id: u64,
}

impl Default for InventoryModel {
    fn default() -> Self {
        Self {
            max_capacity_subunits: DEFAULT_MAX_CAPACITY,
            resources: [0; 3],
            // Keep the five entries that were already shown by the placeholder
            // inventory screen as a small, immediately testable starter loadout.
            items: vec![
                ItemInstance { id: 1, kind: ItemKind::EvilPup },
                ItemInstance { id: 2, kind: ItemKind::Maurice },
                ItemInstance { id: 3, kind: ItemKind::Salad },
                ItemInstance { id: 4, kind: ItemKind::Scooby },
                ItemInstance { id: 5, kind: ItemKind::Theo },
            ],
            next_item_id: 6,
        }
    }
}

impl InventoryModel {
    pub fn resource_amount(&self, kind: ResourceKind) -> u32 {
        self.resources[kind.index()]
    }

    pub fn items(&self) -> &[ItemInstance] {
        &self.items
    }

    pub fn used_capacity_subunits(&self) -> u32 {
        let resource_capacity = ResourceKind::ALL.into_iter().fold(0_u64, |total, kind| {
            total.saturating_add(
                u64::from(self.resource_amount(kind))
                    .saturating_mul(u64::from(kind.capacity_subunits_per_unit())),
            )
        });
        let item_capacity = self.items.iter().fold(0_u64, |total, item| {
            total.saturating_add(u64::from(item.kind.capacity_subunits()))
        });

        resource_capacity
            .saturating_add(item_capacity)
            .min(u64::from(u32::MAX)) as u32
    }

    pub fn is_over_capacity(&self) -> bool {
        self.used_capacity_subunits() > self.max_capacity_subunits
    }

    pub fn can_move(&self) -> bool {
        !self.is_over_capacity()
    }

    pub fn can_engage_combat(&self) -> bool {
        !self.is_over_capacity()
    }

    pub fn can_use_portal(&self) -> bool {
        !self.is_over_capacity()
    }

    /// Manual inventory actions are intentionally unrestricted by capacity.
    pub fn add_resource_manually(&mut self, kind: ResourceKind, amount: u32) {
        let current = self.resource_amount(kind);
        self.resources[kind.index()] = current.saturating_add(amount);
    }

    /// Automatic pickups are accepted only when the resulting inventory fits.
    pub fn try_add_resource_automatically(&mut self, kind: ResourceKind, amount: u32) -> bool {
        let previous = self.resource_amount(kind);
        self.resources[kind.index()] = previous.saturating_add(amount);
        if self.is_over_capacity() {
            self.resources[kind.index()] = previous;
            false
        } else {
            true
        }
    }

    /// Manual item acquisition may create overflow; the player can resolve it
    /// through ordinary inventory actions.
    pub fn add_item_manually(&mut self, kind: ItemKind) -> u64 {
        let id = self.next_item_id;
        self.next_item_id = self.next_item_id.saturating_add(1);
        self.items.push(ItemInstance { id, kind });
        id
    }

    /// Automatic item pickups must not put the inventory over capacity.
    pub fn try_add_item_automatically(&mut self, kind: ItemKind) -> Option<u64> {
        let id = self.add_item_manually(kind);
        if self.is_over_capacity() {
            self.items.pop();
            self.next_item_id = id;
            None
        } else {
            Some(id)
        }
    }

    pub fn leave_resource(&mut self, kind: ResourceKind, amount: u32) -> u32 {
        let current = self.resource_amount(kind);
        let removed = current.min(amount);
        self.resources[kind.index()] = current - removed;
        removed
    }

    pub fn leave_item_at(&mut self, index: usize) -> Option<ItemInstance> {
        (index < self.items.len()).then(|| self.items.remove(index))
    }

    pub fn move_item(&mut self, index: usize, offset: isize) -> bool {
        let Some(destination) = index.checked_add_signed(offset) else {
            return false;
        };
        if index >= self.items.len() || destination >= self.items.len() {
            return false;
        }
        self.items.swap(index, destination);
        true
    }

    pub(crate) fn from_saved(
        max_capacity_subunits: u32,
        gold: u32,
        wood: u32,
        stone: u32,
        items: Vec<ItemInstance>,
    ) -> Self {
        let next_item_id = items
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
            .max(1);

        Self {
            max_capacity_subunits,
            resources: [gold, wood, stone],
            items,
            next_item_id,
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub enum InventoryAction {
    AddResource { kind: ResourceKind, amount: u32 },
    TryAddResourceAutomatically { kind: ResourceKind, amount: u32 },
    TryAddItemAutomatically(ItemKind),
    RemoveResource { kind: ResourceKind, amount: u32 },
    AddItem(ItemKind),
    LeaveLastItem,
    MoveLastItemToFront,
    ClearResources,
}

#[derive(Resource, Default)]
pub struct InventoryActionRequest(pub Option<InventoryAction>);

pub(crate) fn apply_inventory_action(
    mut request: ResMut<InventoryActionRequest>,
    mut inventory: ResMut<InventoryModel>,
) {
    let Some(action) = request.0.take() else {
        return;
    };

    match action {
        InventoryAction::AddResource { kind, amount } => {
            inventory.add_resource_manually(kind, amount);
        }
        InventoryAction::TryAddResourceAutomatically { kind, amount } => {
            inventory.try_add_resource_automatically(kind, amount);
        }
        InventoryAction::TryAddItemAutomatically(kind) => {
            let _ = inventory.try_add_item_automatically(kind);
        }
        InventoryAction::RemoveResource { kind, amount } => {
            inventory.leave_resource(kind, amount);
        }
        InventoryAction::AddItem(kind) => {
            inventory.add_item_manually(kind);
        }
        InventoryAction::LeaveLastItem => {
            let last = inventory.items().len().saturating_sub(1);
            let _ = inventory.leave_item_at(last);
        }
        InventoryAction::MoveLastItemToFront => {
            let last = inventory.items().len().saturating_sub(1);
            if last > 0 {
                inventory.move_item(last, -(last as isize));
            }
        }
        InventoryAction::ClearResources => {
            for kind in ResourceKind::ALL {
                let amount = inventory.resource_amount(kind);
                inventory.leave_resource(kind, amount);
            }
        }
    }
}

pub fn format_capacity(subunits: u32) -> String {
    format!(
        "{}.{:02}",
        subunits / CAPACITY_SUBUNITS_PER_UNIT,
        subunits % CAPACITY_SUBUNITS_PER_UNIT
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resources_and_items_both_contribute_to_capacity() {
        let mut inventory = InventoryModel::default();
        inventory.add_resource_manually(ResourceKind::Gold, 100);
        inventory.add_resource_manually(ResourceKind::Wood, 2);
        inventory.add_resource_manually(ResourceKind::Stone, 2);

        // Starter items use 16.00; resources add 1.00 + 2.00 + 3.00.
        assert_eq!(inventory.used_capacity_subunits(), 2200);
        assert_eq!(format_capacity(inventory.used_capacity_subunits()), "22.00");
    }

    #[test]
    fn exact_capacity_is_valid_but_any_overflow_blocks_restricted_actions() {
        let mut inventory = InventoryModel::default();
        inventory.max_capacity_subunits = inventory.used_capacity_subunits();

        assert!(!inventory.is_over_capacity());
        assert!(inventory.can_move());
        assert!(inventory.can_engage_combat());
        assert!(inventory.can_use_portal());

        inventory.add_resource_manually(ResourceKind::Gold, 1);

        assert!(inventory.is_over_capacity());
        assert!(!inventory.can_move());
        assert!(!inventory.can_engage_combat());
        assert!(!inventory.can_use_portal());
    }

    #[test]
    fn manual_additions_can_overfill_the_inventory() {
        let mut inventory = InventoryModel::default();
        inventory.add_resource_manually(ResourceKind::Stone, 100);

        assert!(inventory.is_over_capacity());
        assert_eq!(inventory.resource_amount(ResourceKind::Stone), 100);
        inventory.add_item_manually(ItemKind::Salad);
        assert_eq!(inventory.items().len(), 6);
    }

    #[test]
    fn automatic_pickups_are_rejected_when_the_result_would_overfill() {
        let mut inventory = InventoryModel::default();
        inventory.max_capacity_subunits = inventory.used_capacity_subunits() + 100;

        assert!(!inventory.try_add_resource_automatically(ResourceKind::Wood, 2));
        assert_eq!(inventory.resource_amount(ResourceKind::Wood), 0);
        assert!(inventory.try_add_item_automatically(ItemKind::EvilPup).is_none());
        assert_eq!(inventory.items().len(), 5);
    }

    #[test]
    fn automatic_pickups_that_fit_are_accepted() {
        let mut inventory = InventoryModel::default();
        inventory.max_capacity_subunits += 1000;

        assert!(inventory.try_add_resource_automatically(ResourceKind::Wood, 2));
        assert!(inventory.try_add_item_automatically(ItemKind::EvilPup).is_some());
        assert_eq!(inventory.resource_amount(ResourceKind::Wood), 2);
        assert_eq!(inventory.items().len(), 6);
    }

    #[test]
    fn leaving_loot_reduces_capacity_and_reordering_does_not_change_it() {
        let mut inventory = InventoryModel::default();
        let initial_capacity = inventory.used_capacity_subunits();

        assert!(inventory.move_item(0, 1));
        assert_eq!(inventory.used_capacity_subunits(), initial_capacity);

        let removed = inventory.leave_item_at(0).unwrap();
        assert_eq!(removed.kind, ItemKind::Maurice);
        assert_eq!(
            inventory.used_capacity_subunits(),
            initial_capacity - ItemKind::Maurice.capacity_subunits()
        );

        assert_eq!(inventory.leave_resource(ResourceKind::Gold, 5), 0);
    }

    #[test]
    fn saved_inventory_preserves_resources_items_and_capacity() {
        let mut inventory = InventoryModel::default();
        inventory.add_resource_manually(ResourceKind::Gold, 123);
        inventory.add_item_manually(ItemKind::Salad);
        inventory.max_capacity_subunits = 12_345;

        let restored = InventoryModel::from_saved(
            inventory.max_capacity_subunits,
            inventory.resource_amount(ResourceKind::Gold),
            inventory.resource_amount(ResourceKind::Wood),
            inventory.resource_amount(ResourceKind::Stone),
            inventory.items().to_vec(),
        );

        assert_eq!(restored, inventory);
    }
}
