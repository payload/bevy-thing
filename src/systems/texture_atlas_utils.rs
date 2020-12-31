use bevy::{input::system::exit_on_esc_system, prelude::*};

pub fn texture_atlas_grid(
    texture: Handle<Texture>,
    size: Vec2,
    padding: Vec2,
    atlases: &mut ResMut<Assets<TextureAtlas>>,
    commands: &mut Commands,
) -> Handle<TextureAtlas> {
    let atlas = atlases.add(TextureAtlas::new_empty(texture, size.clone()));
    commands.spawn((RecreateTextureAtlas(atlas.clone(), size, padding),));
    atlas
}

pub struct TextureAtlasUtilsPlugin;
impl Plugin for TextureAtlasUtilsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(recreate_texture_atlas_system.system());
    }
}

pub struct RecreateTextureAtlas(Handle<TextureAtlas>, Vec2, Vec2);

pub fn recreate_texture_atlas_system(
    commands: &mut Commands,
    textures: Res<Assets<Texture>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    query: Query<(Entity, &RecreateTextureAtlas)>,
) {
    for (entity, RecreateTextureAtlas(handle, size, padding)) in query.iter() {
        for atlas in atlases.get_mut(handle) {
            for tex in textures.get(atlas.texture.clone()) {
                let tile = *size + *padding;
                let cols = (tex.size.width as f32 / tile.x).ceil() as u32;
                let rows = (tex.size.height as f32 / tile.y).ceil() as u32;

                commands.despawn(entity);
                *atlas = TextureAtlas::from_grid_with_padding(
                    atlas.texture.clone(),
                    *size,
                    cols as usize,
                    rows as usize,
                    *padding,
                );
            }
        }
    }
}

pub fn example() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_system(exit_on_esc_system.system())
        .add_startup_system(example_setup.system())
        .add_system(recreate_texture_atlas_system.system())
        .run();
}

fn example_setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let micro_roguelike_tex = asset_server.load("micro-roguelike/Tilemap/colored_tilemap.png");
    let micro_roguelike_tex_atlas = texture_atlas_grid(
        micro_roguelike_tex.clone(),
        Vec2::new(8.0, 8.0),
        Vec2::new(1.0, 1.0),
        &mut atlases,
        commands,
    );

    commands.spawn({
        let mut cam = Camera2dBundle::default();
        cam.transform.scale.x = 0.25;
        cam.transform.scale.y = 0.25;
        cam
    });

    commands.spawn(SpriteBundle {
        material: materials.add(ColorMaterial::texture(micro_roguelike_tex.clone())),
        ..Default::default()
    });

    commands.spawn(SpriteSheetBundle {
        transform: Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        texture_atlas: micro_roguelike_tex_atlas.clone(),
        sprite: TextureAtlasSprite {
            color: Color::WHITE,
            index: 4,
        },
        ..Default::default()
    });
}
