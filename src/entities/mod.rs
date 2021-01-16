use bevy::prelude::*;

pub mod player;

pub struct OvenState {
    pub baking_timer: Timer,
    pub item: Option<&'static str>,
    pub baked_item: Option<&'static str>,
    pub on_fire: bool,
}

impl OvenState {
    pub fn animation(&self) -> &'static str {
        match (
            self.on_fire,
            self.baking_timer.finished(),
            self.item.is_some(),
        ) {
            (false, _, false) => "off",
            (false, false, true) => "off_fish",
            (false, true, true) => "off_bakedfish",
            (true, _, false) => "on",
            (true, false, true) => "on_fish",
            (true, true, true) => "on_bakedfish",
        }
    }

    pub fn interact(&mut self) -> Option<&'static str> {
        match (
            self.on_fire,
            self.baking_timer.finished(),
            self.item.is_some(),
        ) {
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
                self.item = Some("fish");
                self.baked_item = Some("bakedfish");
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
