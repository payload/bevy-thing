use bevy::prelude::*;

use crate::{bevy_rapier_utils::*, commands_ext::*, components::*};

pub struct Player;
pub struct PlayerSpawn;

pub fn spawn_player(commands: &mut Commands, transform: Transform) {
    let entity = commands.entity((Player,));
    let Vec3 { x, y, .. } = transform.translation;

    commands
        .with(transform)
        .with(GlobalTransform::default())
        .with(Dress::Bitpack(25, Color::ORANGE));
    commands
        .with_bundle((RigidBodyBuilder::new_dynamic()
            .translation(transform.translation.x, transform.translation.y)
            .user_data(entity.to_user_data()),))
        .with_child((
            ColliderBuilder::ball(4.0).user_data(entity.to_user_data()), // .collision_groups(InteractionGroups::new(0x0002, 0xffff)),
        ));

    #[cfg(any())]
    commands
        .with(
            RigidBodyBuilder::new_dynamic()
                .translation(x, y)
                .user_data(entity.to_user_data())
                .linear_damping(8.0)
                .lock_rotations(),
        )
        .with_child((
            ColliderBuilder::ball(4.0)
                .user_data(entity.to_user_data())
                .sensor(true), // .collision_groups(InteractionGroups::new(0x0001, 0xffff)),
        ))
        .with_a_child(|e| {
            (
                "player forward sensor".to_string(),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                ColliderBuilder::ball(4.0)
                    .user_data(e.to_bits() as u128)
                    .translation(0.0, 8.0)
                    .sensor(true),
            )
        });
}

pub fn player_spawn_system(
    commands: &mut Commands,
    query: Query<(Entity, &PlayerSpawn, &Transform)>,
) {
    for (entity, _spawn, trans) in query.iter() {
        println!("player {:?}", trans.clone().translation);
        spawn_player(commands, trans.clone());
        commands.despawn(entity);
    }
}
