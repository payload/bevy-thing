use bevy::prelude::*;

use crate::assets::TexAtlases;

#[derive(Default, Debug)]
pub struct Inventory {
    pub items: Vec<&'static str>,
}

impl Inventory {
    pub fn has(&self, item: &'static str) -> bool {
        self.pos(item).is_some()
    }

    pub fn take(&mut self, item: &'static str) {
        for pos in self.pos(item) {
            self.items.remove(pos);
        }
    }

    pub fn put(&mut self, item: &'static str) {
        self.items.push(item);
    }

    pub fn count(&self, item: &'static str) -> usize {
        self.items.iter().filter(|it| **it == item).count()
    }

    fn pos(&self, item: &'static str) -> Option<usize> {
        self.items.iter().position(|it| *it == item)
    }
}

pub struct Item {
    pub name: &'static str,
    pub tex_atlas: Handle<TextureAtlas>,
    pub tex_sprite: u32,
}

pub struct Items {
    pub fish: Item,
    pub baked_fish: Item,
}

impl Items {
    pub fn new(atlases: &TexAtlases) -> Self {
        Self {
            fish: Item {
                name: "fish",
                tex_atlas: atlases.oven_atlas.clone(),
                tex_sprite: 10,
            },
            baked_fish: Item {
                name: "baked_fish",
                tex_atlas: atlases.oven_atlas.clone(),
                tex_sprite: 11,
            }
        }
    }
}
