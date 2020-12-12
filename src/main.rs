use bevy::prelude::*;
use rand::prelude::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(demo_assets_bit_pack)
        .add_system(atlas_tinyview_hover)
        .add_system(cursor_system)
        .run();
}

fn setup(commands: &mut Commands) {
    let camera = commands
        .spawn(Camera2dBundle::default())
        .current_entity()
        .unwrap();
    commands.insert_resource(MyCursorState {
        main_camera: camera,
        world_pos: None,
    });
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

fn atlas_tinyview_hover(
    cursor: Res<MyCursorState>,
    atlantes: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &GlobalTransform,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    if let Some(cursor_pos) = cursor.world_pos {
        for (transform, mut sprite, atlas_handle) in query.iter_mut() {
            let atlas = atlantes.get(atlas_handle).expect("get atlas");
            let rect = atlas
                .textures
                .get(sprite.index as usize)
                .expect("get atlas texture rect");
            let pos = Vec2::from(transform.translation);
            let size = Vec2::new(rect.width(), rect.height());
            let tl_pos = pos - 0.5 * size;
            let br_pos = pos + 0.5 * size;
            if tl_pos.x <= cursor_pos.x
                && tl_pos.y <= cursor_pos.y
                && br_pos.x >= cursor_pos.x
                && br_pos.y >= cursor_pos.y
            {
                if sprite.color != Color::RED {
                    sprite.color = Color::RED;
                }
            } else if sprite.color != Color::WHITE {
                sprite.color = Color::WHITE;
            }
        }
    } else {
        for (_transform, mut sprite, _atlas_handle) in query.iter_mut() {
            if sprite.color != Color::WHITE {
                sprite.color = Color::WHITE
            }
        }
    }
}

struct MyCursorState {
    main_camera: Entity,
    world_pos: Option<Vec2>,
}

fn cursor_system(
    mut state: ResMut<MyCursorState>,
    windows: Res<Windows>,
    q_camera: Query<&Transform>,
) {
    if let Some((window, cursor)) = windows.get_primary().and_then(|window| {
        window
            .cursor_position()
            .and_then(|cursor| Some((window, cursor)))
    }) {
        let camera_transform = q_camera.get(state.main_camera).unwrap();
        let window_size = Vec2::new(window.width() as f32, window.height() as f32);

        // the default orthographic projection is in pixels from the center;
        // just undo the translation
        let pos = cursor - window_size * 0.5;

        // apply the camera transform
        let world_pos = camera_transform.compute_matrix() * pos.extend(0.0).extend(1.0);
        state.world_pos = Some(Vec2::from(world_pos));
    } else {
        state.world_pos = None;
    }
}

fn random_angle() -> f32 {
    2.0 * 3.14159 * (-0.5 + rand::random::<f32>())
}