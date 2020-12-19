/*
    mages, trees, stones are nice but lets get proximity to the stones
    before picking them up.

    random movement on mages, but only pick up when near
    use rapier
*/

use bevy::prelude::*;

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;

use crate::bitpack::Bitpack;

use crate::level1::*;

pub struct Level3Plugin;

impl Plugin for Level3Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system());
    }
}

fn setup(commands: &mut Commands) {

}