/*
    a stone, you can pick it up, drop it again, throw it with some power, it clonks on the ground, it bings on the wall
    a mage, can pick things up, drop them, throw them, move

    random controling can move randomly, may pick up things if near, may drop, may throw randomly

    Stone, Mage are markers
    CanBeItemBasics and CanItemBasics deals with picking, dropping, throwing
    SoundOnContact, ContactType, SoundType describes which sound to play when
    Kinematics deals with physical movement of things
    MovementAbility deals with the ability for physical movement
    ControlRandomMovement for controling MovementAbility things
    ControlRandomItemBasics for controling CanItemBasics things
*/

use bevy::{ecs::DynamicBundle, prelude::*};
use rand::prelude::*;

use crate::bitpack::Bitpack;

pub struct Level1Plugin;

impl Plugin for Level1Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup)
            .add_startup_system(add_camera)
            .add_system(kinematic_system)
            .add_system(control_random_movement_system)
            .add_system(control_random_item_basics_system)
            .add_system(carry_system)
            .add_system(throw_system);
    }
}

pub struct Stone;
pub struct Mage;

pub struct CanBeItemBasics {
    pick_up: bool,
    drop: bool,
    throw: bool,
}

pub struct CanItemBasics {
    pick_up: bool,
    drop: bool,
    throw: bool,
    picked_up: Option<Entity>,
}

pub struct SoundOnContact {
    _sound_map: Vec<(ContactType, SoundType)>,
}

impl SoundOnContact {
    fn new(sound_map: Vec<(ContactType, SoundType)>) -> Self {
        Self {
            _sound_map: sound_map,
        }
    }
}

pub enum ContactType {
    Ground,
    Wall,
}
pub enum SoundType {
    Clonk,
    Bling,
}

pub struct Kinematics {
    vel: Vec3,
    drag: f32,
}

pub struct MovementAbility {
    top_speed: f32,
}

pub struct ControlRandomMovement {
    timer: Timer,
}

pub struct ControlRandomItemBasics {
    timer: Timer,
}

pub struct Carried {
    owner: Entity,
    offset: Transform,
}

pub struct Thrown {
    vel: Vec3,
}

impl Thrown {
    pub fn new(vel: Vec3) -> Self {
        Self { vel }
    }
}

pub fn setup(commands: &mut Commands, bitpack: Res<Bitpack>) {
    use ContactType::*;
    use SoundType::*;

    let stone = (
        Stone,
        CanBeItemBasics {
            pick_up: true,
            drop: true,
            throw: true,
        },
        Kinematics {
            vel: Vec3::zero(),
            drag: 0.97,
        },
        Transform::from_translation(Vec3::new(-32.0, 0.0, 0.0)),
        GlobalTransform::default(),
        SoundOnContact::new(vec![(Ground, Clonk), (Wall, Bling)]),
    );

    let mage = (
        Mage,
        CanItemBasics {
            pick_up: true,
            drop: true,
            throw: false,
            picked_up: None,
        },
        Kinematics {
            vel: Vec3::zero(),
            drag: 0.97,
        },
        MovementAbility { top_speed: 20.0 },
        Transform::default(),
        GlobalTransform::default(),
        ControlRandomMovement {
            timer: Timer::from_seconds(1.0, true),
        },
        ControlRandomItemBasics {
            timer: Timer::from_seconds(1.1, true),
        },
    );

    let stone_dress = dress_stone(bitpack.atlas_handle.clone());
    let mage_dress = dress_mage(bitpack.atlas_handle.clone());

    commands
        .spawn(stone)
        .with_children(|child| {
            child.spawn(stone_dress);
        })
        .spawn(mage)
        .with_children(|child| {
            child.spawn(mage_dress);
        });
}

fn dress_stone(atlas: Handle<TextureAtlas>) -> impl DynamicBundle {
    SpriteSheetBundle {
        texture_atlas: atlas,
        sprite: TextureAtlasSprite {
            index: 3,
            color: Color::GRAY,
        },
        ..Default::default()
    }
}

fn dress_mage(atlas: Handle<TextureAtlas>) -> impl DynamicBundle {
    SpriteSheetBundle {
        texture_atlas: atlas,
        sprite: TextureAtlasSprite {
            index: 24,
            color: Color::BLACK,
        },
        ..Default::default()
    }
}

pub fn kinematic_system(time: Res<Time>, mut query: Query<(Mut<Kinematics>, Mut<Transform>)>) {
    let dt = time.delta_seconds();
    for (mut kin, mut trans) in query.iter_mut() {
        let drag = kin.drag;
        kin.vel *= 1.0 - drag * dt;
        trans.translation += kin.vel * dt;
    }
}

pub fn control_random_movement_system(
    time: Res<Time>,
    mut query: Query<(
        Mut<ControlRandomMovement>,
        Mut<Kinematics>,
        &MovementAbility,
    )>,
) {
    let dt = time.delta_seconds();
    let mut rng = rand::thread_rng();
    for (mut control, mut kin, movement) in query.iter_mut() {
        if control.timer.tick(dt).finished() {
            let top_speed = movement.top_speed;
            let rand_vec = rng.random_vec2d() * top_speed * 0.8;
            kin.vel = rand_vec;
        }
    }
}

pub fn control_random_item_basics_system(
    commands: &mut Commands,
    time: Res<Time>,
    mut active_query: Query<(Entity, Mut<ControlRandomItemBasics>, Mut<CanItemBasics>)>,
    mut item_query: Query<(Entity, Mut<CanBeItemBasics>)>,
) {
    let dt = time.delta_seconds();

    for (active, mut control, can) in active_query.iter_mut() {
        if control.timer.tick(dt).finished() {
            if let Some(item) = can.picked_up {
                control_drop_or_throw_item(commands, can, item, &mut item_query);
            } else if can.pick_up {
                control_pick_up_item(commands, active, can, &mut item_query);
            }
        }
    }
}

fn control_drop_or_throw_item(
    commands: &mut Commands,
    mut can: Mut<CanItemBasics>,
    item: Entity,
    item_query: &mut Query<(Entity, Mut<CanBeItemBasics>)>,
) {
    let mut rng = rand::thread_rng();
    let mut can_be = item_query
        .get_component_mut::<CanBeItemBasics>(item)
        .unwrap();
    let drop = can.drop && can_be.drop;
    let throw = can.throw && can_be.throw;

    if (drop && throw && rng.gen()) || (drop && !throw) {
        can.picked_up = None;
        can_be.pick_up = true;
        commands.remove_one::<Carried>(item);
    } else if throw {
        can.picked_up = None;
        can_be.pick_up = true;
        commands.insert_one(item, Thrown::new(rng.random_vec2d() * 40.0));
        commands.remove_one::<Carried>(item);
    }
}

fn control_pick_up_item(
    commands: &mut Commands,
    owner: Entity,
    mut can: Mut<CanItemBasics>,
    item_query: &mut Query<(Entity, Mut<CanBeItemBasics>)>,
) {
    let pick_up = item_query.iter_mut().filter(|(_, c)| c.pick_up).next();
    let offset = Transform::from_translation(Vec3::new(0.0, 6.0, 0.0));

    if let Some((item, mut can_be)) = pick_up {
        can.picked_up = Some(item.clone());
        can_be.pick_up = false;
        commands.insert_one(item, Carried { owner, offset });
    }
}

pub fn carry_system(
    mut query: Query<(&Carried, Mut<Transform>)>,
    transform: Query<&Transform, Without<Carried>>,
) {
    for (carried, mut trans) in query.iter_mut() {
        let owner_trans = transform.get(carried.owner).expect("owner has transform");
        trans.translation = owner_trans.translation + carried.offset.translation;
    }
}

pub fn throw_system(commands: &mut Commands, mut query: Query<(Entity, &Thrown, Mut<Kinematics>)>) {
    for (entity, thrown, mut kin) in query.iter_mut() {
        kin.vel += thrown.vel;
        commands.remove_one::<Thrown>(entity);
    }
}

pub fn add_camera(commands: &mut Commands) {
    commands
        .spawn(Camera2dBundle {
            transform: Transform::from_scale(Vec3::splat(0.25)),
            ..Default::default()
        })
        .current_entity()
        .unwrap();
}

trait RandomVec {
    fn random_vec2d(&mut self) -> Vec3;
}

impl RandomVec for rand::ThreadRng {
    fn random_vec2d(&mut self) -> Vec3 {
        Vec3::new(-0.5 + self.gen::<f32>(), -0.5 + self.gen::<f32>(), 0.0).normalize()
    }
}
