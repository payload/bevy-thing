#![feature(bool_to_option)]

mod input;
mod stuff;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(stuff::setup)
        .add_startup_system(stuff::demo_assets_bit_pack)
        .add_system(stuff::atlas_tinyview_hover)
        .add_system(stuff::cursor_system)
        .add_resource(input::MyInputState::default())
        .add_system_to_stage(stage::PRE_UPDATE, input::my_input_system)
        .add_system(input::system)
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        // .add_system(|diagnostics: Res<Diagnostics>| println!("{:?}", diagnostics.get(FrameTimeDiagnosticsPlugin::FPS).unwrap().average()))
        // .add_system_to_stage(stage::LAST, |mut ev: ResMut<Events<AppExit>>| ev.send(AppExit))
        .run();
}
