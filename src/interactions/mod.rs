use crate::bevy_rapier_utils::*;
use bevy::prelude::*;

pub enum GameInteraction {
    PushAway(PushAway),
}

pub struct PushAway {
    pub which: Entity,
    pub relative_to: Entity,
    pub rel_impulse: f32,
}

pub fn interactions_system(
    _commands: &mut Commands,
    mut reader: Local<EventReader<GameInteraction>>,
    events: Res<Events<GameInteraction>>,
    mut bodies: ResMut<RigidBodySet>,
    body: Query<&RigidBodyHandleComponent>,
) {
    for effect in reader.iter(&events) {
        match effect {
            GameInteraction::PushAway(push) => push_away(push, &mut bodies, &body),
        }
    }
}

pub fn push_away(
    push: &PushAway,
    bodies: &mut ResMut<RigidBodySet>,
    body: &Query<&RigidBodyHandleComponent>,
) {
    if let Some(body) = body
        .get(push.which)
        .ok()
        .and_then(|body| bodies.get_mut(body.handle()))
    {
        let dir = Vector2::new(push.rel_impulse, push.rel_impulse);
        body.set_linvel(dir, true);
    }
}
