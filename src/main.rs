#![feature(bool_to_option)]

mod bevy_rapier_utils;
mod bitpack;
mod bitpack_map;
mod bundle_utils;
mod commands_ext;
mod components;
mod entities;
mod interactions;
mod levels;
mod map_asset;
mod rapier_debug_render;
mod systems;
mod utils;

use levels::*;

fn main() {
    if let Some(command) = std::env::args().nth(1) {
        match command.as_str() {
            "level1" => level1::app().run(),
            "level2" => level2::app().run(),
            "level3" => level3::app().run(),
            "level4" => level4::app().run(),
            "steering-arcade" => systems::steering::arcade_example(),
            "steering-rapier" => systems::steering::rapier_example(),
            "context-map" => systems::context_map::example(),
            "jabber" => systems::jabber::example(),
            "texture-atlas-utils" => systems::texture_atlas_utils::example(),
            _ => (),
        }
    }
}
