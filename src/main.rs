mod bitpack;
mod bitpack_map;

mod level1;
mod level2;

mod map_asset;
mod stuff;

use bevy::prelude::*;
use bitpack::BitpackPlugin;
use level1::Level1Plugin;
use level2::Level2Plugin;

fn main() {
    if let Some(command) = std::env::args().nth(1) {
        match command.as_str() {
            "level1" => level1(),
            "level2" => level2(),
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
