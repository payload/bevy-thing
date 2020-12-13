use bevy::prelude::*;

pub const MAGICIAN1: usize = 24;
pub const TREE1: usize = 51;

pub fn load_bitpack(
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) -> Handle<TextureAtlas> {
    let texture_handle = asset_server.load("bit-pack/Tilesheet/monochrome_transparent.png");
    atlases.add(TextureAtlas::from_grid_with_padding(
        texture_handle,
        Vec2::new(16.0, 16.0),
        768 / 16,
        352 / 16,
        Vec2::new(1.0, 1.0),
    ))
}
