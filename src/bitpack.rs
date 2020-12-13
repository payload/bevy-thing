use bevy::prelude::*;

pub const MAGICIAN1: usize = 24;
pub const TREE1: usize = 51;

pub struct BitpackPlugin;

pub struct Bitpack {
    pub atlas_handle: Handle<TextureAtlas>,
}

impl Plugin for BitpackPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(load_bitpack);
    }
}

fn load_bitpack(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("bit-pack/Tilesheet/monochrome_transparent.png");
    let atlas_handle = atlases.add(TextureAtlas::from_grid_with_padding(
        texture_handle,
        Vec2::new(16.0, 16.0),
        768 / 16,
        352 / 16,
        Vec2::new(1.0, 1.0),
    ));
    commands.insert_resource(Bitpack { atlas_handle });
}
