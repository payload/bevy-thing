/*
    mages, trees, stones are nice but lets get proximity to the stones
    before picking them up.

    random movement on mages, but only pick up when near
    use rapier
*/

use bevy::prelude::*;
use bevy_rapier2d::{
    na::Point2,
    physics::{JointBuilderComponent, RapierPhysicsPlugin},
    rapier::{
        dynamics::{BallJoint, RigidBodyBuilder},
        geometry::ColliderBuilder,
    },
    render::RapierRenderPlugin,
};

pub struct Level3Plugin;

impl Plugin for Level3Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app /**/
            .add_plugin(RapierPhysicsPlugin)
            .add_plugin(RapierRenderPlugin)
            .add_startup_system(setup.system())
            .add_system(print_events.system())
            /* end */;
    }
}

trait With<T> {
    fn with<F: FnOnce(&mut T) -> ()>(self, func: F) -> T;
}

impl<T> With<T> for T {
    fn with<F: FnOnce(&mut T) -> ()>(mut self, func: F) -> T {
        func(&mut self);
        self
    }
}

fn setup(commands: &mut Commands) {
    commands.spawn(Camera2dBundle::default());

    let body1 = RigidBodyBuilder::new_static();
    let collider1 = ColliderBuilder::cuboid(100.0, 10.0);

    let body2 = RigidBodyBuilder::new_dynamic().translation(0.0, 50.0);
    let collider2 = ColliderBuilder::ball(10.0);

    let a_body1 = commands.spawn((body1, collider1)).current_entity().unwrap();
    let a_body2 = commands.spawn((body2, collider2)).current_entity().unwrap();

    let joint_params = BallJoint::new(Point2::origin(), Point2::new(5.0, -50.0));
    let joint = JointBuilderComponent::new(joint_params, a_body1, a_body2);
    commands.spawn((joint,));
}

fn print_events(events: Res<bevy_rapier2d::physics::EventQueue>) {
    while let Ok(proximity_event) = events.proximity_events.pop() {
        println!("{:?}", proximity_event);
    }

    while let Ok(contact_event) = events.contact_events.pop() {
        println!("{:?}", contact_event);
    }
}
