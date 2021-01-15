#![allow(dead_code)]

/// based on John Peels flock-rs, MIT licensed https://github.com/JohnPeel/flock-rs
use bevy::{
    prelude::*,
    render::{pipeline::RenderPipeline, render_graph::base::MainPass},
    sprite::{QUAD_HANDLE, SPRITE_PIPELINE_HANDLE},
};
use bevy_rapier2d::{
    physics::{RapierConfiguration, RapierPhysicsPlugin, RigidBodyHandleComponent},
    rapier::{
        dynamics::{RigidBodyBuilder, RigidBodySet},
        geometry::ColliderBuilder,
        math::Isometry,
    },
};
use rand::Rng;

use crate::bevy_rapier_utils::IntoVector2;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Boid {
    id: usize,
    flock_id: usize,
    velocity: Vec2,
    max_speed: f32,
    safe_radius: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct FlockParameters {
    id: usize,
    boid_count: usize,
    color: Color,
    radius: f32,
    alignment_strength: f32,
    cohesion_strength: f32,
    separation_strength: f32,
}

#[derive(Default)]
struct FlockAverages {
    position: Vec2,
    forward: Vec2,
    current_count: usize,
    boids: Vec<(Boid, Vec2)>,
}

impl FlockAverages {
    fn add(&mut self, boid: &Boid, boid_pos: Vec2) {
        if self.current_count == 0 {
            self.position = boid_pos;
            self.forward = boid.velocity;
        } else {
            self.position = 0.5 * (self.position + boid_pos);
            self.forward = 0.5 * (self.forward + boid.velocity);
        }

        self.current_count += 1;
        self.boids[boid.id] = (*boid, boid_pos);
    }
}

pub type Flocks = Vec<FlockParameters>;

fn spawn_flocks(commands: &mut Commands, flocks: &Flocks) {
    let mut rng = rand::thread_rng();

    for flock in flocks.iter() {
        for index in 0..flock.boid_count {
            let pos = Vec2::new(rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0));
            let pos = pos * flock.radius;
            let vel = -pos.normalize();

            commands.spawn((
                Transform::from_translation(pos.extend(0.0)),
                Boid {
                    id: index,
                    flock_id: flock.id,
                    velocity: vel * 50.0,
                    max_speed: 50.0,
                    safe_radius: 25.0,
                },
            ));
        }
    }
}

fn calculate_averages(
    flocks: &Flocks,
    averages: &mut Vec<FlockAverages>,
    iter: &mut dyn Iterator<Item = (Mut<Boid>, Mut<Transform>)>,
) {
    let mut result = Vec::<FlockAverages>::with_capacity(flocks.len());
    result.resize_with(result.capacity(), FlockAverages::default);

    for flock in flocks.iter() {
        let mut boids = Vec::with_capacity(flock.boid_count);
        boids.resize(flock.boid_count, Default::default());

        result[flock.id] = FlockAverages {
            position: Vec2::zero(),
            forward: Vec2::zero(),
            current_count: 0,
            boids,
        };
    }

    for (boid, transform) in iter {
        result[boid.flock_id].add(&boid, transform.translation.truncate());
    }

    *averages = result;
}

fn flocks_update_system(
    time: Res<Time>,
    mut averages: Local<Vec<FlockAverages>>,
    flocks: ResMut<Flocks>,
    mut boids_q: Query<(Mut<Boid>, Mut<Transform>)>,
) {
    calculate_averages(
        &flocks,
        &mut averages,
        &mut boids_q.iter_mut(),
    );

    for average in averages.iter_mut() {
        let boids = average.boids.clone();

        for (boid, boid_pos) in boids.iter() {
            let flock = flocks[boid.flock_id];

            let mut separation = Vec2::zero();

            for (other, other_pos) in boids.iter() {
                if boid.id != other.id {
                    let steer = Steer::new(*boid_pos, *other_pos);
                    separation += steer.separation(boid.safe_radius + other.safe_radius);
                }
            }

            if separation.length_squared() > 1.0 {
                separation = separation.normalize();
            }

            let cohesion = Steer::new(*boid_pos, average.position).cohesion(flock.radius);
            let alignment = Steer::alignment(boid.max_speed, average.forward);
            let to_center = Steer::new(*boid_pos, Vec2::zero()).keep_close(2000.0);

            let weighted: Vec2 = flock.alignment_strength * alignment
                + flock.cohesion_strength * cohesion
                + flock.separation_strength * separation
                + to_center;
            let scaled = weighted * boid.max_speed * time.delta_seconds();
            let mut new_velocity = boid.velocity + scaled;

            if new_velocity.length_squared() > boid.max_speed * boid.max_speed {
                new_velocity = new_velocity.normalize() * boid.max_speed;
            }

            average.boids[boid.id].0.velocity = new_velocity;
        }
    }

    for (mut boid, mut trans) in boids_q.iter_mut() {
        boid.velocity = averages[boid.flock_id].boids[boid.id].0.velocity;
        trans.translation = averages[boid.flock_id].boids[boid.id].1.extend(0.0);
    }
}

fn calculate_alignment(max_speed: f32, average_forward: Vec2) -> Vec2 {
    let alignment = average_forward / max_speed;

    if alignment.length_squared() > 1.0 {
        alignment.normalize()
    } else {
        alignment
    }
}

fn calculate_separation(boid: &Boid, position: &Vec2, boids: &[(Boid, Vec2)]) -> Vec2 {
    let mut separation = Vec2::zero();

    for (other, other_pos) in boids.iter() {
        if boid.id != other.id {
            let difference = *position - *other_pos;
            let distance_squared = difference.length_squared();
            let minimum_distance = boid.safe_radius + other.safe_radius;

            if distance_squared < minimum_distance * minimum_distance {
                separation += difference.normalize() * (minimum_distance - distance_squared.sqrt())
                    / minimum_distance;
            }
        }
    }

    if separation.length_squared() > 1.0 {
        separation.normalize()
    } else {
        separation
    }
}

fn calculate_cohesion(&position: &Vec2, &average_position: &Vec2, flock_radius: f32) -> Vec2 {
    let cohesion = average_position - position;

    if cohesion.length_squared() < flock_radius * flock_radius {
        cohesion / flock_radius
    } else {
        cohesion.normalize()
    }
}

pub fn boid_arcade_update_system(time: Res<Time>, mut boids_q: Query<(&Boid, Mut<Transform>)>) {
    for (boid, mut trans) in boids_q.iter_mut() {
        let vel = (boid.velocity * time.delta_seconds()).extend(0.0);
        trans.translation += vel;
        trans.rotation = Quat::from_rotation_z(boid.velocity.y.atan2(boid.velocity.x));
    }
}

fn boid_example_sprite_system(
    commands: &mut Commands,
    query: Query<(Entity, &Boid), Without<Sprite>>,
    //
    ass: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut material: Local<Option<Handle<ColorMaterial>>>,
) {
    if material.is_none() {
        let texture = ass.load("micro-roguelike/Tiles/Colored/tile_0087.png");
        *material = Some(materials.add(texture.into()));
    }

    let material = (*material).clone().unwrap();

    for (entity, _) in query.iter() {
        commands.insert(
            entity,
            (
                material.clone(),
                //
                QUAD_HANDLE.typed() as Handle<Mesh>,
                RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                    SPRITE_PIPELINE_HANDLE.typed(),
                )]),
                Visible {
                    is_transparent: true,
                    ..Default::default()
                },
                MainPass,
                Sprite::default(),
                Draw::default(),
                GlobalTransform::default(),
            ),
        );
    }
}

pub fn arcade_example() {
    App::build()
        .init_resource::<Flocks>()
        .add_plugins(DefaultPlugins)
        .add_startup_system(example_setup.system())
        .add_system(flocks_update_system.system())
        .add_system(boid_arcade_update_system.system())
        .add_system(boid_example_sprite_system.system())
        .run();
}

fn example_setup(cmds: &mut Commands) {
    cmds.spawn({
        let mut bundle = Camera2dBundle::default();
        bundle.transform.scale = Vec3::new(0.5, 0.5, 1.0);
        bundle
    });

    let mut flocks = Flocks::default();
    flocks.push(FlockParameters {
        id: 0,
        boid_count: 10,
        alignment_strength: 1.0,
        cohesion_strength: 1.0,
        separation_strength: 1.0,
        color: Color::WHITE,
        radius: 50.0,
    });

    spawn_flocks(cmds, &flocks);
    cmds.insert_resource(flocks);
}

fn rapier_config(mut config: ResMut<RapierConfiguration>) {
    config.gravity.y = 0.0;
}

///////////////////////

pub fn rapier_example() {
    App::build()
        .init_resource::<Flocks>()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin)
        .add_startup_system(example_setup.system())
        .add_startup_system(rapier_config.system())
        .add_system(flocks_update_system.system())
        .add_system(boid_rapier_update_system.system())
        .add_system(boid_example_sprite_system.system())
        .add_system(boid_rapier_body_system.system())
        .run();
}

pub fn boid_rapier_update_system(
    mut bodies: ResMut<RigidBodySet>,
    mut query: Query<(&Boid, &RigidBodyHandleComponent)>,
) {
    for (boid, body_component) in query.iter_mut() {
        for body in bodies.get_mut(body_component.handle()) {
            let angel = boid.velocity.y.atan2(boid.velocity.x);
            let pos = body.position();
            let pos = Isometry::new(pos.translation.vector, angel);
            body.set_position(pos, true);

            let speed = body.linvel().magnitude_squared();
            let max_speed = boid.max_speed * boid.max_speed;
            body.linear_damping = 1.25 * speed / max_speed;

            body.apply_impulse(boid.velocity.into_vector2(), true);
        }
    }
}

pub fn boid_rapier_body_system(
    commands: &mut Commands,
    query: Query<
        (Entity, &Transform, &Boid),
        (Without<RigidBodyBuilder>, Without<RigidBodyHandleComponent>),
    >,
) {
    for (entity, transform, _) in query.iter() {
        commands.insert(
            entity,
            (
                RigidBodyBuilder::new_dynamic()
                    .translation(transform.translation.x, transform.translation.y)
                    .rotation(transform.rotation.z)
                    .lock_rotations()
                    .linear_damping(1.0),
                ColliderBuilder::ball(4.0).restitution(1.0),
            ),
        );
    }
}

//

pub struct Steer {
    a: Vec2,
    b: Vec2,
    diff: Vec2,
    distance: f32,
    norm: Vec2,
}

impl Steer {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        let diff: Vec2 = b - a;
        let distance = diff.length();
        let norm = diff.normalize();
        Self {
            a,
            b,
            diff,
            distance,
            norm,
        }
    }

    pub fn towards_if_between(&self, min: f32, max: f32) -> Vec2 {
        if self.distance >= min && self.distance <= max {
            self.norm
        } else {
            Vec2::zero()
        }
    }

    pub fn separation(&self, safe_distance: f32) -> Vec2 {
        let overlap = self.distance - safe_distance;
        if overlap < 0.0 {
            self.norm * overlap / safe_distance
        } else {
            Vec2::zero()
        }
    }

    pub fn cohesion(&self, radius: f32) -> Vec2 {
        if self.distance < radius {
            self.diff / radius
        } else {
            self.norm
        }
    }

    pub fn keep_close(&self, max_radius: f32) -> Vec2 {
        self.diff * (self.diff / max_radius).length_squared()
    }

    pub fn alignment(max_speed: f32, average_forward: Vec2) -> Vec2 {
        let alignment = average_forward / max_speed;

        if alignment.length_squared() > 1.0 {
            alignment.normalize()
        } else {
            alignment
        }
    }
}
