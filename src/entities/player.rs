use bevy::prelude::*;

use crate::{
    bevy_rapier_utils::*,
    commands_ext::*,
    components::*,
    interactions::{GameInteraction, PushAway},
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Player {
    forward_sensor: Option<Entity>,
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerSpawn;

#[derive(Debug, Clone, Copy)]
pub enum PlayerEvent {
    Observe,
    Interact,
}

pub fn spawn_player(commands: &mut Commands, transform: Transform) {
    let entity = commands.entity((Marker::Player,));
    let Vec3 { x, y, .. } = transform.translation;

    let mut player = Player::default();

    commands
        .with(transform)
        .with(GlobalTransform::default())
        .with(Dress::Bitpack(25, Color::ORANGE))
        .with(
            RigidBodyBuilder::new_dynamic()
                .translation(x, y)
                .user_data(entity.to_user_data())
                .linear_damping(8.0)
                .lock_rotations(),
        )
        .with_child((ColliderBuilder::ball(8.0).user_data(entity.to_user_data()),))
        .with_a_child(|e| {
            player.forward_sensor = Some(e);
            (
                ProximitySet::default(),
                ColliderBuilder::ball(8.0)
                    .user_data(e.to_bits() as u128)
                    .translation(0.0, 8.0)
                    .sensor(true),
            )
        })
        .with(player);
}

pub fn player_spawn_system(
    commands: &mut Commands,
    query: Query<(Entity, &PlayerSpawn, &Transform)>,
) {
    for (entity, _spawn, trans) in query.iter() {
        spawn_player(commands, trans.clone());
        commands.despawn(entity);
    }
}

pub fn player_handle_input_events(
    _commands: &mut Commands,
    //
    mut reader: Local<EventReader<PlayerEvent>>,
    events: Res<Events<PlayerEvent>>,
    mut interactions: ResMut<Events<GameInteraction>>,
    //
    query: Query<(Entity, &Player)>,
    proximity: Query<&ProximitySet>,
    observe: Query<&Marker>,
) {
    for ev in reader.iter(&events) {
        match ev {
            PlayerEvent::Interact => {
                for (entity, player) in query.iter() {
                    player_interact(entity, player, &proximity, &mut interactions);
                }
            }
            PlayerEvent::Observe => {
                for (entity, player) in query.iter() {
                    player_observe(entity, player, &proximity, &observe);
                }
            }
        }
    }
}

fn player_interact(
    entity: Entity,
    player: &Player,
    proximity: &Query<&ProximitySet>,
    interactions: &mut Events<GameInteraction>,
) {
    for proximity in player.forward_sensor.and_then(|it| proximity.get(it).ok()) {
        for near_entity in proximity.iter() {
            interactions.send(GameInteraction::PushAway(PushAway {
                which: *near_entity,
                relative_to: entity,
                rel_impulse: 20.0,
            }));
        }
    }
}

fn player_observe(
    _entity: Entity,
    player: &Player,
    proximity: &Query<&ProximitySet>,
    observe: &Query<&Marker>,
) {
    for proximity in player.forward_sensor.and_then(|it| proximity.get(it).ok()) {
        let observations = proximity
            .iter()
            .filter_map(|e| observe.get(*e).ok())
            .collect::<Vec<_>>();

        if !observations.is_empty() {
            println!("{:?}", observations);
        }
    }
}
