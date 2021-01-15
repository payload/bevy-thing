use crate::{
    bevy_rapier_utils::*,
    commands_ext::CommandsExt,
    components::ProximitySet,
    systems::{context_map::*, steering::Steer, texture_atlas_utils::*},
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

    let layer0 = 500.0;
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
            GameEntity::Mob,
            "Player".to_string(),
            Transform::from_translation(Vec3::new(0.0, 32.0, layer0)),
            GlobalTransform::default(),
            ContextMapAI::new_random(),
            Gizmo::new(Color::WHITE, 8.0, 16.0),
        ))
        .with_child(SpriteSheetBundle {
            texture_atlas: micro_roguelike_tex_atlas.clone(),
            sprite: TextureAtlasSprite::new(4),
            ..Default::default()
        });

    commands
        .spawn((
            GameEntity::Mob,
            "Orc".to_string(),
            Transform::from_translation(Vec3::new(-32.0, 0.0, layer0)),
            GlobalTransform::default(),
            ContextMapAI::new_random(),
            Gizmo::new(Color::WHITE, 8.0, 16.0),
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
            GameEntity::Chest,
            "Chest".to_string(),
            Transform::from_translation(Vec3::new(0.0, -16.0, layer0)),
            GlobalTransform::default(),
        ))
        .with_child(SpriteSheetBundle {
            texture_atlas: micro_roguelike_tex_atlas.clone(),
            sprite: TextureAtlasSprite::new(51),
            ..Default::default()
        });

    let mut wall = |x, y| {
        commands
            .spawn((
                GameEntity::Wall,
                Transform::from_translation(Vec3::new(x, y, layer0)),
                GlobalTransform::default(),
            ))
            .with_child(SpriteSheetBundle {
                texture_atlas: micro_roguelike_tex_atlas.clone(),
                sprite: TextureAtlasSprite::new(1),
                ..Default::default()
            });
    };

    for y in -3..=3 {
        wall(-16.0, y as f32 * 8.0)
    }
}

enum GameEntity {
    Mob,
    Chest,
    Wall,
}

fn update_global_context_ai(
    mut this_query: Query<(Entity, &Transform, &GameEntity, Mut<ContextMapAI>)>,
    others_query: Query<(Entity, &Transform, &GameEntity)>,
) {
    let others: Vec<_> = others_query.iter().collect();

    for (this, trans, character, mut ai) in this_query.iter_mut() {
        ai.interests.weights *= 0.0; // 1.0 - 0.5 * time.delta_seconds();
        ai.dangers.weights *= 0.0; // 1.0 - 0.5 * time.delta_seconds();

        let mut separation = Vec2::zero();

        for (other, o_trans, o_character) in others.iter() {
            if &this != other {
                let o_pos = pos(o_trans);
                let pos = pos(trans);
                let diff: Vec2 = pos - o_pos;
                let len = diff.length();

                match (character, o_character) {
                    (GameEntity::Mob, GameEntity::Chest) => {
                        let steer = Steer::new(pos, o_pos);
                        let towards = steer.towards_if_between(10.0, 100.0);
                        ai.interests.add_map(towards, |w| (1.0 + w) / 2.0);
                    }
                    (GameEntity::Mob, GameEntity::Mob) => {}
                    (GameEntity::Mob, GameEntity::Wall) => {
                        if len < 24.0 {
                            // TODO set danger map hard, other w function maybe
                            ai.dangers.add_map((o_pos - pos).normalize(), |w| {
                                if w > 0.9 {
                                    1.0
                                } else {
                                    0.0
                                }
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        if separation.length_squared() > 1.0 {
            separation = separation.normalize();
        }

        // ai.interests.add_map(separation, |w| 1.0 - w * w);
    }
}

fn movement_system(time: Res<Time>, mut this_query: Query<(Mut<Transform>, &ContextMapAI)>) {
    for (mut trans, ai) in this_query.iter_mut() {
        let mut interests = ai.interests.clone();

        // TODO maybe leave lowest danger untouched or use a threshold
        for i in 0..ai.dangers.weights.len() {
            if ai.dangers.weights[i] > 0.0 {
                interests.weights[i] = 0.0;
            }
        }

        let dir = (interests.direction() + ai.dangers.direction().perp()).normalize();
        let movement = dir * 1.0 / 60.0 * 5.0;
        trans.translation += movement.extend(0.0);
    }
}

fn pos(trans: &Transform) -> Vec2 {
    trans.translation.truncate()
}
