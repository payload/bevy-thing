#![feature(bool_to_option)]

mod bevy_rapier_utils;
mod bitpack;
mod bitpack_map;
mod bundle_utils;
mod commands_ext;
mod map_asset;
mod utils;
mod rapier_debug_render;
mod entities;
mod components;
mod interactions;

mod level1;
mod level2;
mod level3;
mod level4;

use bevy::prelude::*;
use bitpack::BitpackPlugin;
use level1::Level1Plugin;
use level2::Level2Plugin;
use level3::Level3Plugin;

fn main() {
    if let Some(command) = std::env::args().nth(1) {
        match command.as_str() {
            "level1" => level1(),
            "level2" => level2(),
            "level3" => level3(),
            "level4" => level4::app().run(),
            _ => (),
        }
    }
}

fn level1() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(BitpackPlugin)
        .add_plugin(Level1Plugin)
        .run();
}

fn level2() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(BitpackPlugin)
        .add_plugin(Level2Plugin)
        .run();
}

fn level3() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(BitpackPlugin)
        .add_plugin(Level3Plugin)
        .run();
}
