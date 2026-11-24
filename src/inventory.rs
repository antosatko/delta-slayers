use bevy::prelude::*;

use crate::ui_stack::UiStack;

#[derive(Component)]
pub struct InventoryRoot;

pub fn inventory_visibility_system(
    ui_stack: Res<UiStack>,
    mut roots: Query<&mut Visibility, With<InventoryRoot>>,
) {
    for mut visibility in &mut roots {
        *visibility = if ui_stack.inventory_open() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}