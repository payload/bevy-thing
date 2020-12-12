pub mod cursor;

use bevy::prelude::*;
use rand::prelude::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(demo_assets_bit_pack)
        .add_system(atlas_tinyview_hover)
        .run();
}

fn setup(commands: &mut Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn demo_assets_bit_pack(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("bit-pack/Tilesheet/monochrome_transparent.png");
    let size = Vec2::new(16.0, 16.0);
    let padding = Vec2::new(1.0, 1.0);
    let cols = 768 / 16;
    let rows = 352 / 16;
    let atlas = TextureAtlas::from_grid_with_padding(texture_handle, size, cols, rows, padding);
    let atlas_handle = atlases.add(atlas);
    let atlas = atlases.get(&atlas_handle).unwrap();
    let spacing_factor = 1.5;

    for index in 0..atlas.len() {
        let rect = atlas.textures.get(index).unwrap();
        let sprite = TextureAtlasSprite {
            index: index as u32,
            color: Color::GREEN,
        };
        commands.spawn(SpriteSheetBundle {
            texture_atlas: atlases.get_handle(&atlas_handle),
            sprite,
            transform: Transform {
                translation: Vec3::new(
                    spacing_factor * (rect.min.x - atlas.size.x * 0.5),
                    -spacing_factor * (rect.min.y - atlas.size.y * 0.5),
                    0.0,
                ),
                scale: Vec3::splat(1.0),
                rotation: Quat::default(),
            },
            ..Default::default()
        });
    }
}

fn atlas_tinyview_hover(mut query: Query<(&GlobalTransform, &mut TextureAtlasSprite)>) {
    let mut rng = rand::thread_rng();
    for (transform, mut sprite) in query.iter_mut() {
        if rng.gen::<f32>() < 0.1f32 {
            sprite.color = Color::RED;
        } else {
            sprite.color = Color::WHITE;
        }
    }
}

fn random_angle() -> f32 {
    2.0 * 3.14159 * (-0.5 + rand::random::<f32>())
}
