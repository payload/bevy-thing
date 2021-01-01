use crate::{
    bevy_rapier_utils::*,
    commands_ext::CommandsExt,
    components::ProximitySet,
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
            "Player".to_string(),
            Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),
            GlobalTransform::default(),
            Faction::NiceMob,
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
            "Orc".to_string(),
            Transform::from_translation(Vec3::new(-50.0, 0.0, 0.0)),
            GlobalTransform::default(),
            ProximitySet::default(),
            ContextMapAI::new_random(),
            Gizmo::new(Color::WHITE, 8.0),
            Faction::AggresiveMob,
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
            Faction::AggresiveMob,
            ContextMapAI::new_random(),
            Gizmo::new(Color::WHITE, 8.0),
        ))
        .with_child(SpriteSheetBundle {
            texture_atlas: micro_roguelike_tex_atlas.clone(),
            sprite: TextureAtlasSprite::new(10),
            ..Default::default()
        });

    commands
        .spawn((
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

enum Faction {
    NiceMob,
    AggresiveMob,
}

fn _update_proximity_context_ai(
    comp_query: Query<(Option<&Transform>, Option<&Faction>)>,
    mut ai_query: Query<(Entity, &ProximitySet, Mut<ContextMapAI>)>,
) {
    for (entity, proximity, mut ai) in ai_query.iter_mut() {
        ai.interests.weights *= 0.0; // 1.0 - 0.5 * time.delta_seconds();
        ai.dangers.weights *= 0.0; // 1.0 - 0.5 * time.delta_seconds();

        let (e_trans, e_faction) = comp_query.get(entity).unwrap();
        for &other in proximity.iter() {
            let (o_trans, o_faction) = comp_query.get(other).unwrap();

            match (e_trans, e_faction, o_trans, o_faction) {
                (Some(e_trans), Some(e_faction), Some(o_trans), Some(o_faction)) => {
                    update_context_ai_factions(
                        o_trans.translation.truncate() - e_trans.translation.truncate(),
                        e_faction,
                        o_faction,
                        &mut ai,
                    );
                }
                _ => {}
            }
        }
    }
}

fn update_global_context_ai(
    mut this_query: Query<(Entity, &Transform, &Faction, Mut<ContextMapAI>)>,
    others_query: Query<(Entity, &Transform, &Faction)>,
) {
    for (this, trans, faction, mut ai) in this_query.iter_mut() {
        ai.interests.weights *= 0.0; // 1.0 - 0.5 * time.delta_seconds();
        ai.dangers.weights *= 0.0; // 1.0 - 0.5 * time.delta_seconds();

        for (other, o_trans, o_faction) in others_query.iter() {
            if this != other {
                let diff: Vec2 = trans.translation.truncate() - o_trans.translation.truncate();
                let len = diff.length();
                // 110 0   100 0.5   0 1   10 1
                let vec = vec![
                    (0.0, 1.0),
                    (10.0, 1.0),
                    (100.0, 0.5),
                    (110.0, 0.0),
                    (111.0, 0.0),
                ];
                let spline = makima_spline::Spline::from_vec(vec);
                let sample = spline.sample(len as f64) as f32;
                let diff = diff * 0.9 * (sample / len);
                update_context_ai_factions(diff, faction, o_faction, &mut ai);

                if len < 20.0 {
                    ai.interests.add(diff * -(20.0 / len))
                }
            }
        }
    }
}

fn update_context_ai_factions(
    diff: Vec2,
    e_faction: &Faction,
    o_faction: &Faction,
    ai: &mut ContextMapAI,
) {
    use Faction::*;
    match (e_faction, o_faction) {
        (NiceMob, NiceMob) => ai.interests.add(diff),
        (NiceMob, AggresiveMob) => ai.dangers.add(diff),
        (AggresiveMob, NiceMob) => ai.interests.add(diff),
        (AggresiveMob, AggresiveMob) => {}
    }
}

fn movement_system(time: Res<Time>, mut this_query: Query<(Mut<Transform>, &ContextMapAI)>) {
    for (mut trans, ai) in this_query.iter_mut() {
        let interest = ai.interests.max_as_vec2();
        let _danger = ai.dangers.max_as_vec2();

        trans.translation += (interest * time.delta_seconds() * 50.0).extend(0.0);
    }
}
