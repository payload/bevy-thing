use bevy::{prelude::*, reflect::TypeUuid};

use crate::assets::TexAtlases;

#[derive(Default, Debug)]
pub struct Inventory {
    pub items: Vec<ItemKind>,
}

impl Inventory {
    pub fn has(&self, item: &ItemKind) -> bool {
        self.pos(item).is_some()
    }

    pub fn take(&mut self, item: &ItemKind) {
        for pos in self.pos(item) {
            self.items.remove(pos);
        }
    }

    pub fn put(&mut self, item: &ItemKind) {
        self.items.push(item.clone());
    }

    pub fn count(&self, item: &ItemKind) -> usize {
        self.items.iter().filter(|it| *it == item).count()
    }

    fn pos(&self, item: &ItemKind) -> Option<usize> {
        self.items.iter().position(|it| it == item)
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, TypeUuid)]
#[uuid = "39cadc56-aa9c-4543-8640-a118b74b5052"]
pub struct Item {
    pub name: &'static str,
    pub tex_atlas: Handle<TextureAtlas>,
    pub tex_sprite: u32,
}

impl Item {
    pub fn sprite_sheet_bundle(&self) -> SpriteSheetBundle {
        SpriteSheetBundle {
            texture_atlas: self.tex_atlas.clone(),
            sprite: TextureAtlasSprite::new(self.tex_sprite),
            ..Default::default()
        }
    }
}

pub struct Items {
    pub fish: ItemKind,
    pub baked_fish: ItemKind,
}

impl Items {
    pub fn new(assets: &mut Assets<Item>, atlases: &TexAtlases) -> Self {
        Self {
            fish: assets.add(Item {
                name: "fish",
                tex_atlas: atlases.oven_atlas.clone(),
                tex_sprite: 10,
            }),
            baked_fish: assets.add(Item {
                name: "baked_fish",
                tex_atlas: atlases.oven_atlas.clone(),
                tex_sprite: 11,
            }),
        }
    }
}

pub type ItemKind = Handle<Item>;
