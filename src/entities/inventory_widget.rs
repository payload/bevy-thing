use bevy::prelude::*;

use crate::{
    commands_ext::CommandsExt,
    systems::inventory::{Inventory, Item},
};

#[derive(Default)]
pub struct InventoryWidget {
    pub slots: Vec<Entity>,
    pub items: Vec<Entity>,
    pub selection: Option<usize>,
    pub tex_atlas: Handle<TextureAtlas>,
    pub tex_unselected_index: u32,
    pub tex_selected_index: u32,
}

pub fn inventory_widget_added(
    commands: &mut Commands,
    mut query: Query<(Entity, Mut<InventoryWidget>), Added<InventoryWidget>>,
) {
    for (entity, mut widget) in query.iter_mut() {
        if widget.slots.is_empty() {
            for index in -4..4 {
                let slot = commands.entity(SpriteSheetBundle {
                    transform: Transform::from_xyz(index as f32 * 7.0, 0.0, 0.0),
                    texture_atlas: widget.tex_atlas.clone(),
                    sprite: TextureAtlasSprite::new(widget.tex_unselected_index),
                    ..Default::default()
                });

                widget.slots.push(slot);
            }

            commands.push_children(entity, &widget.slots);
        }
    }
}

pub fn inventory_widget_selection_control(
    keys: Res<Input<KeyCode>>,
    mut widget_query: Query<Mut<InventoryWidget>>,
) {
    let slot_selection = [
        KeyCode::Key1,
        KeyCode::Key2,
        KeyCode::Key3,
        KeyCode::Key4,
        KeyCode::Key5,
        KeyCode::Key6,
        KeyCode::Key7,
        KeyCode::Key8,
    ]
    .iter()
    .map(|c| keys.just_pressed(*c))
    .enumerate()
    .filter_map(|(index, pressed)| pressed.then_some(index))
    .last();

    let selection = if keys.just_pressed(KeyCode::Key0) {
        Some(None)
    } else {
        slot_selection.map(Some)
    };

    for selection in selection {
        for mut widget in widget_query.iter_mut() {
            if widget.selection != selection {
                widget.selection = selection;
            }
        }
    }
}

pub fn inventory_widget_selection_system(
    widget_query: Query<&InventoryWidget, Changed<InventoryWidget>>,
    mut sprite_query: Query<(Mut<TextureAtlasSprite>, Mut<Transform>)>,
) {
    for widget in widget_query.iter() {
        for (index, slot) in widget.slots.iter().enumerate() {
            let selected = Some(index) == widget.selection;
            let tex_index = if selected {
                widget.tex_selected_index
            } else {
                widget.tex_unselected_index
            };

            // TODO slot should keep a semantic state
            // such that sprite.index and translation.z can be derived from that
            // Model View distinction!

            for (mut sprite, mut trans) in sprite_query.get_mut(*slot) {
                if sprite.index != tex_index {
                    sprite.index = tex_index;
                    trans.translation.z += if selected { 0.1 } else { -0.1 };
                }
            }
        }
    }
}

pub fn inventory_widget_items_system(
    commands: &mut Commands,
    items: Res<Assets<Item>>,
    mut widget_query: Query<(Mut<InventoryWidget>, &Inventory), Changed<Inventory>>,
) {
    for (mut widget, inventory) in widget_query.iter_mut() {
        for item in widget.items.drain(0..) {
            commands.despawn_recursive(item);
        }

        for (index, item) in inventory.items.iter().enumerate() {
            for item in items.get(item) {
                let item_entity = commands
                    .spawn(item.sprite_sheet_bundle())
                    .with(Transform::from_xyz(0.0, 0.0, 0.1))
                    .unwrap_entity();

                widget.items.push(item_entity);
                commands.push_children(widget.slots[index], &[item_entity]);
            }
        }
    }
}
