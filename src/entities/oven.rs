use bevy::prelude::*;

use crate::systems::inventory::{ItemKind, Items};

pub struct OvenState {
    pub baking_timer: Timer,
    pub item: Option<ItemKind>,
    pub baked_item: Option<ItemKind>,
    pub on_fire: bool,
}

pub fn oven_update(time: Res<Time>, mut query: Query<Mut<OvenState>>) {
    let delta = time.delta_seconds();
    for mut oven in query.iter_mut() {
        oven.baking_timer.tick(delta);
    }
}

impl OvenState {
    pub fn animation(&self) -> &'static str {
        let fire = self.on_fire;
        let baked = self.baking_timer.finished();
        let item = self.item.is_some();

        match (fire, baked, item) {
            (false, _, false) => "off",
            (false, false, true) => "off_fish",
            (false, true, true) => "off_bakedfish",
            (true, _, false) => "on",
            (true, false, true) => "on_fish",
            (true, true, true) => "on_bakedfish",
        }
    }

    pub fn interact(&mut self, items: &Items) -> Option<ItemKind> {
        let fire = self.on_fire;
        let baked = self.baking_timer.finished();
        let item = self.item.is_some();

        match (fire, baked, item) {
            (false, _, false) => {
                self.on_fire = true;
                None
            }
            (false, false, true) => self.item.take(),
            (false, true, true) => {
                self.item = None;
                self.baked_item.take()
            }
            (true, _, false) => {
                self.baking_timer.reset();
                self.item = Some(items.fish.clone());
                self.baked_item = Some(items.baked_fish.clone());
                None
            }
            (true, false, true) => {
                self.on_fire = false;
                self.baked_item = None;
                self.item.take()
            }
            (true, true, true) => {
                self.on_fire = false;
                self.item = None;
                self.baked_item.take()
            }
        }
    }
}
