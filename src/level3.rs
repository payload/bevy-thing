/*
    mages, trees, stones are nice but lets get proximity to the stones
    before picking them up.

    random movement on mages, but only pick up when near
    use rapier
*/

use bevy::prelude::*;
use bevy_rapier2d::{
    na::*,
    physics::*,
    rapier::{dynamics::*, geometry::*},
    render::*,
};

use crate::level2::*;

pub struct Level3Plugin;

impl Plugin for Level3Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app /**/
            .add_plugin(RapierPhysicsPlugin)
            .add_plugin(RapierRenderPlugin)
            .add_startup_system(setup.system())
            .add_system(print_events.system())
            //
            .add_system(sync_tilemap_spawner_system.system())
            .add_asset::<TileMap>()
            .init_asset_loader::<TileMapLoader>()
            .add_event::<TileMapSpawnEvent>()
            /* end */;
    }
}

/*
    spawn plain entity

    body_build.user_data(player_entity.to_bits() as u128)
    collider_build
    insert entity (body, collider)

    insert entity render_stuff

    proximity event handler
    b1 = bodies.get(handle1).unwrap()
    Entity::from_bits(b1.user_data as u64)
*/

fn setup(commands: &mut Commands, mut config: ResMut<RapierConfiguration>) {
    commands.spawn(Camera2dBundle::default());

    // config.gravity = Vector2::new(0.0, 0.0)

    let a_body1 = {
        let entity = commands.spawn(()).current_entity().unwrap();

        let body = RigidBodyBuilder::new_static().user_data(entity.to_bits() as u128);
        let collider = ColliderBuilder::cuboid(100.0, 10.0).sensor(true);
        commands.insert(entity, (body, collider));
        entity
    };

    let a_body2 = {
        let entity = commands.spawn(()).current_entity().unwrap();

        let body = RigidBodyBuilder::new_dynamic().user_data(entity.to_bits() as u128);
        let collider = ColliderBuilder::ball(10.0);
        let color = RapierRenderColor(1.0, 0.0, 0.0);
        commands.insert(entity, (body, collider, color));
        entity
    };

    {
        let joint = BallJoint::new(Point2::origin(), Point2::new(5.0, -50.0));
        let joint_builder = JointBuilderComponent::new(joint, a_body1, a_body2);
        commands.spawn((joint_builder,));
    }
}

fn print_events(events: Res<EventQueue>, bodies: Res<RigidBodySet>) {
    while let Ok(event) = events.proximity_events.pop() {
        let body1 = bodies.get(event.collider1).unwrap();
        let body2 = bodies.get(event.collider2).unwrap();

        let a_collider1 = Entity::from_bits(body1.user_data as u64);
        let a_collider2 = Entity::from_bits(body2.user_data as u64);

        println!("{:?}\n\t{:?} {:?}", event, a_collider1, a_collider2);
    }

    while let Ok(contact_event) = events.contact_events.pop() {
        println!("{:?}", contact_event);
    }
}
