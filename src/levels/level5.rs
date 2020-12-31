use crate::{
    bevy_rapier_utils::*,
    commands_ext::CommandsExt,
    systems::{context_map::*, texture_atlas_utils::*},
};
use bevy::{input::system::exit_on_esc_system, prelude::*};

pub fn app() -> AppBuilder {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        //
        .add_plugin(RapierPhysicsPlugin)
        .add_plugin(RapierRenderPlugin)
        .add_plugin(TextureAtlasUtilsPlugin)
        //
        .add_system(exit_on_esc_system.system())
        //
        .add_startup_system(setup.system())
        .add_system(context_map_gizmo_system.system());
    app
}

type StateResources<'a> = (Res<'a, Assets<TextureAtlas>>, Res<'a, Assets<Texture>>);
struct State {
    micro_roguelike_tex: Handle<Texture>,
    micro_roguelike_tex_atlas: Handle<TextureAtlas>,
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut rapier: ResMut<RapierConfiguration>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    rapier.gravity.y = 0.0;

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

    commands
        .spawn((
            "Player".to_string(),
            Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),
            GlobalTransform::default(),
        ))
        .with_child(SpriteSheetBundle {
            texture_atlas: micro_roguelike_tex_atlas.clone(),
            sprite: TextureAtlasSprite::new(4),
            ..Default::default()
        });

    commands
        .spawn((
            "Orc".to_string(),
            Transform::from_translation(Vec3::new(-50.0, 0.0, 0.0)),
            GlobalTransform::default(),
            ContextMap::new(Vector8::new_random()),
            Gizmo::new(Color::WHITE, 8.0),
        ))
        .with_child(SpriteSheetBundle {
            texture_atlas: micro_roguelike_tex_atlas.clone(),
            sprite: TextureAtlasSprite::new(11),
            ..Default::default()
        });

    commands
        .spawn((
            "Skeleton".to_string(),
            Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
            GlobalTransform::default(),
        ))
        .with_child(SpriteSheetBundle {
            texture_atlas: micro_roguelike_tex_atlas.clone(),
            sprite: TextureAtlasSprite::new(10),
            ..Default::default()
        });

    commands.insert_resource(State {
        micro_roguelike_tex,
        micro_roguelike_tex_atlas,
    });
}
