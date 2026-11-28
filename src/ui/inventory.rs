use bevy::prelude::*;

use crate::gameplay::inventory::{
    format_capacity, InventoryModel, ResourceKind,
};

#[derive(Resource, Clone, Debug, Default)]
pub(super) struct InventoryViewModel {
    pub(super) capacity_label: String,
    pub(super) status_label: String,
    pub(super) resources_label: String,
}

#[derive(Component, Default, Clone)]
pub(super) struct InventoryRoot;

#[derive(Component, Clone, Copy, Default)]
pub(super) struct InventoryCapacityText;

#[derive(Component, Clone, Copy, Default)]
pub(super) struct InventoryStatusText;

#[derive(Component, Clone, Copy, Default)]
pub(super) struct InventoryResourcesText;

/// Projects the gameplay inventory into presentation-only labels.
pub(super) fn sync_inventory_view_model(
    model: Res<InventoryModel>,
    mut view_model: ResMut<InventoryViewModel>,
    mut capacity_labels: Query<&mut Text, With<InventoryCapacityText>>,
    mut status_labels: Query<&mut Text, (With<InventoryStatusText>, Without<InventoryCapacityText>)>,
    mut resource_labels: Query<&mut Text, (With<InventoryResourcesText>, Without<InventoryCapacityText>, Without<InventoryStatusText>)>,
) {
    let used = model.used_capacity_subunits();
    let max = model.max_capacity_subunits;
    view_model.capacity_label = format!("Capacity: {} / {}", format_capacity(used), format_capacity(max));
    view_model.status_label = if model.is_over_capacity() {
        format!("OVER CAPACITY by {}", format_capacity(used - max))
    } else {
        "Within capacity".to_owned()
    };
    view_model.resources_label = ResourceKind::ALL
        .into_iter()
        .map(|kind| format!("{}: {}", kind.name(), model.resource_amount(kind)))
        .collect::<Vec<_>>()
        .join("\n");
    for mut text in &mut capacity_labels {
        if text.0 != view_model.capacity_label {
            text.0.clone_from(&view_model.capacity_label);
        }
    }
    for mut text in &mut status_labels {
        if text.0 != view_model.status_label {
            text.0.clone_from(&view_model.status_label);
        }
    }
    for mut text in &mut resource_labels {
        if text.0 != view_model.resources_label {
            text.0.clone_from(&view_model.resources_label);
        }
    }

}

