use bevy::prelude::*;

#[derive(Resource, Clone, Debug, Default)]
pub struct InventoryModel {
    pub gold: u32,
    pub wood: u32,
    pub stone: u32,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct InventoryViewModel {
    pub gold_label: String,
    pub wood_label: String,
    pub stone_label: String,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub enum CurrencyLabel {
    #[default]
    Gold,
    Wood,
    Stone,
}

#[derive(Component, Default, Clone)]
pub struct InventoryRoot;

/// Projects domain data into presentation-ready labels. Text components are
/// updated only when their displayed value differs, so newly spawned BSN
/// entities receive the current model even if the model itself did not change
/// during the same frame.
pub fn sync_inventory_view_model(
    model: Res<InventoryModel>,
    mut view_model: ResMut<InventoryViewModel>,
    mut labels: Query<(&CurrencyLabel, &mut Text)>,
) {
    let gold = format!("Gold: {}", model.gold);
    let wood = format!("Wood: {}", model.wood);
    let stone = format!("Stone: {}", model.stone);

    view_model.gold_label.clone_from(&gold);
    view_model.wood_label.clone_from(&wood);
    view_model.stone_label.clone_from(&stone);

    for (label, mut text) in &mut labels {
        let value = match label {
            CurrencyLabel::Gold => &gold,
            CurrencyLabel::Wood => &wood,
            CurrencyLabel::Stone => &stone,
        };

        if text.0 != *value {
            text.0.clone_from(value);
        }
    }
}
