use bevy::prelude::*;

use crate::gameplay::inventory::InventoryModel;

#[derive(Resource, Clone, Debug, Default)]
pub(super) struct InventoryViewModel {
    gold_label: String,
    wood_label: String,
    stone_label: String,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub(super) enum CurrencyLabel {
    #[default]
    Gold,
    Wood,
    Stone,
}

#[derive(Component, Default, Clone)]
pub(super) struct InventoryRoot;

/// Projects authoritative inventory values into presentation-ready labels.
pub(super) fn sync_inventory_view_model(
    model: Res<InventoryModel>,
    mut view_model: ResMut<InventoryViewModel>,
    mut labels: Query<(&CurrencyLabel, &mut Text)>,
) {
    view_model.gold_label = format!("Gold: {}", model.gold);
    view_model.wood_label = format!("Wood: {}", model.wood);
    view_model.stone_label = format!("Stone: {}", model.stone);

    for (label, mut text) in &mut labels {
        let value = match label {
            CurrencyLabel::Gold => &view_model.gold_label,
            CurrencyLabel::Wood => &view_model.wood_label,
            CurrencyLabel::Stone => &view_model.stone_label,
        };
        if text.0 != *value {
            text.0.clone_from(value);
        }
    }
}
