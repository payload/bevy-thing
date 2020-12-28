/// based on John Peels flock-rs, MIT licensed https://github.com/JohnPeel/flock-rs
use bevy::{
    prelude::*,
    render::{pipeline::RenderPipeline, render_graph::base::MainPass},
    sprite::{QUAD_HANDLE, SPRITE_PIPELINE_HANDLE},
};
use rand::Rng;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
struct Boid {
    id: usize,
    flock_id: usize,
    velocity: Vec2,
    max_speed: f32,
    safe_radius: f32,
}

#[derive(Debug, Copy, Clone)]
struct FlockParameters {
    id: usize,
    boid_count: usize,
    color: Color,
    flock_radius: f32,
    alignment_strength: f32,
    cohesion_strength: f32,
    separation_strength: f32,
}

#[derive(Default)]
struct FlockAverages {
    average_position: Vec2,
    average_forward: Vec2,
    current_count: usize,
    boids: Vec<(Boid, Vec2)>,
}

type Flocks = Vec<FlockParameters>;

fn spawn_flocks(commands: &mut Commands, flocks: &Flocks) {
    let mut rng = rand::thread_rng();

    for flock in flocks.iter() {
        for index in 0..flock.boid_count {
            let pos = Vec2::new(rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0));
            let pos = pos * flock.flock_radius;
            let vel = -pos.normalize();

            commands.spawn((
                Transform::from_translation(pos.extend(0.0)),
                Boid {
                    id: index,
                    flock_id: flock.id,
                    velocity: vel * 50.0,
                    max_speed: 50.0,
                    safe_radius: 50.0,
                },
            ));
        }
    }
}

fn calculate_averages(
    flocks: &Flocks,
    averages: &mut Vec<FlockAverages>,
    query: &mut Query<(Mut<Boid>, Mut<Transform>)>,
) {
    let mut result = Vec::<FlockAverages>::with_capacity(flocks.len());
    result.resize_with(result.capacity(), FlockAverages::default);

    for flock in flocks.iter() {
        let mut boids = Vec::with_capacity(flock.boid_count);
        boids.resize(flock.boid_count, Default::default());

        result[flock.id] = FlockAverages {
            average_position: Vec2::zero(),
            average_forward: Vec2::zero(),
            current_count: 0,
            boids,
        };
    }

    for (boid, transform) in query.iter_mut() {
        let average = &mut result[boid.flock_id];
        let boid_pos = transform.translation.truncate();

        if average.current_count == 0 {
            average.average_position = boid_pos;
            average.average_forward = boid.velocity;
        } else {
            average.average_position = 0.5 * (average.average_position + boid_pos);
            average.average_forward = 0.5 * (average.average_forward + boid.velocity);
        }

        average.current_count += 1;
        average.boids[boid.id] = (*boid, boid_pos);
    }

    *averages = result;
}

fn update_flocks(
    time: Res<Time>,
    mut averages: Local<Vec<FlockAverages>>,
    flocks: ResMut<Flocks>,
    mut boids_q: Query<(Mut<Boid>, Mut<Transform>)>,
) {
    calculate_averages(&flocks, &mut averages, &mut boids_q);

    for average in averages.iter_mut() {
        let boids = average.boids.clone();

        for (boid, boid_pos) in boids.iter() {
            let separation = calculate_separation(&boid, &boid_pos, &boids);
            let alignment = calculate_alignment(boid.max_speed, average.average_forward);
            let cohesion = calculate_cohesion(
                &boid_pos,
                &average.average_position,
                flocks[boid.flock_id].flock_radius,
            );
            let to_center = -*boid_pos * (*boid_pos / 2000.0).length_squared();

            let weighted: Vec2 = 0.5 * alignment + cohesion + 0.9 * separation + to_center;
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

fn update_boid(time: Res<Time>, mut boids_q: Query<(&Boid, Mut<Transform>)>) {
    for (boid, mut trans) in boids_q.iter_mut() {
        let vel = (boid.velocity * time.delta_seconds()).extend(0.0);
        trans.translation += vel;
        trans.rotation = Quat::from_rotation_z(boid.velocity.y.atan2(boid.velocity.x));
    }
}

fn add_boid_sprite(
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

pub fn example() {
    App::build()
        .init_resource::<Flocks>()
        .add_plugins(DefaultPlugins)
        .add_startup_system(example_setup.system())
        .add_system(update_flocks.system())
        .add_system(update_boid.system())
        .add_system(add_boid_sprite.system())
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
        flock_radius: 100.0,
    });

    spawn_flocks(cmds, &flocks);
    cmds.insert_resource(flocks);
}
