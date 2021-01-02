use crate::{bevy_rapier_utils::*, commands_ext::CommandsExt, components::ProximitySet, systems::{context_map::*, steering::Steer, texture_atlas_utils::*}};
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
        .add_system(context_map_gizmo_system.system())
        .add_system(update_global_context_ai.system())
        .add_system(context_map_ai_gizmo_system.system())
        .add_system(movement_system.system());
    app
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut rapier: ResMut<RapierConfiguration>,
    mut clear_color: ResMut<ClearColor>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    rapier.gravity.y = 0.0;
    clear_color.0 = Color::rgb(0.133, 0.137, 0.137);

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
            Character::Mob,
            "Player".to_string(),
            Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),
            GlobalTransform::default(),
            ContextMapAI::new_random(),
            Gizmo::new(Color::WHITE, 8.0),
        ))
        .with_child(SpriteSheetBundle {
            texture_atlas: micro_roguelike_tex_atlas.clone(),
            sprite: TextureAtlasSprite::new(4),
            ..Default::default()
        });

    commands
        .spawn((
            Character::Mob,
            "Orc".to_string(),
            Transform::from_translation(Vec3::new(-20.0, 0.0, 0.0)),
            GlobalTransform::default(),
            ContextMapAI::new_random(),
            Gizmo::new(Color::WHITE, 8.0),
        ))
        .with_child(SpriteSheetBundle {
            texture_atlas: micro_roguelike_tex_atlas.clone(),
            sprite: TextureAtlasSprite::new(11),
            ..Default::default()
        });

    /*
    commands
        .spawn((
            "Skeleton".to_string(),
            Transform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
            GlobalTransform::default(),
            Faction::AggresiveMob,
            ContextMapAI::new_random(),
            Gizmo::new(Color::WHITE, 8.0),
        ))
        .with_child(SpriteSheetBundle {
            texture_atlas: micro_roguelike_tex_atlas.clone(),
            sprite: TextureAtlasSprite::new(10),
            ..Default::default()
        });
        */

    commands
        .spawn((
            Character::Chest,
            "Chest".to_string(),
            Transform::from_translation(Vec3::new(0.0, -20.0, 0.0)),
            GlobalTransform::default(),
        ))
        .with_child(SpriteSheetBundle {
            texture_atlas: micro_roguelike_tex_atlas.clone(),
            sprite: TextureAtlasSprite::new(51),
            ..Default::default()
        });
}

enum Character {
    Mob,
    Chest,
}

fn update_global_context_ai(
    mut this_query: Query<(Entity, &Transform, &Character, Mut<ContextMapAI>)>,
    others_query: Query<(Entity, &Transform, &Character)>,
) {
    let others: Vec<_> = others_query.iter().collect();

    for (this, trans, character, mut ai) in this_query.iter_mut() {
        ai.interests.weights *= 0.0; // 1.0 - 0.5 * time.delta_seconds();
        ai.dangers.weights *= 0.0; // 1.0 - 0.5 * time.delta_seconds();

        for (other, o_trans, o_character) in others.iter() {
            if &this != other {
                let diff: Vec2 = trans.translation.truncate() - o_trans.translation.truncate();
                let len = diff.length();

                match (character, o_character) {
                    (Character::Mob, Character::Chest) => {
                        let steer = Steer::new(trans.translation.truncate(), o_trans.translation.truncate());
                        let towards = steer.towards_if_between(10.0, 100.0);
                        ai.interests.add_map(towards, |w| (1.0 + w) / 2.0);
                    },
                    (Character::Mob, Character::Mob) => {
                        let steer = Steer::new(trans.translation.truncate(), o_trans.translation.truncate());
                        let separation = 0.9 * steer.separation(10.0);
                        ai.interests.add_map(separation, |w| 1.0 - w * w);
                    },
                    _ => {}
                }
            }
        }
    }
}

fn movement_system(time: Res<Time>, mut this_query: Query<(Mut<Transform>, &ContextMapAI)>) {
    for (mut trans, ai) in this_query.iter_mut() {
        let interest = ai.interests.max_as_vec2();
        let _danger = ai.dangers.max_as_vec2();

        // trans.translation += (interest * 1.0 / 60.0 * 50.0).extend(0.0);
    }
}
