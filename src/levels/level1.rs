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

use std::cmp::Ordering;

use bevy::{ecs::DynamicBundle, prelude::*};
use rand::prelude::*;

use crate::bitpack::{Bitpack, BitpackPlugin};

pub fn app() -> AppBuilder {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_plugin(Level1Plugin)
        .add_plugin(BitpackPlugin);
    app
}

pub struct Level1Plugin;

impl Plugin for Level1Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system())
            .add_startup_system(add_camera.system())
            .add_system(kinematic_system.system())
            .add_system(control_random_movement_system.system())
            .add_system(control_random_item_basics_system.system())
            .add_system(carry_system.system())
            .add_system(throw_system.system());
    }
}

pub struct Stone;
pub struct Mage;

pub struct CanBeItemBasics {
    pub pick_up: bool,
    pub drop: bool,
    pub throw: bool,
}

pub struct CanItemBasics {
    pub pick_up: bool,
    pub drop: bool,
    pub throw: bool,
    pub picked_up: Option<Entity>,
}

pub struct SoundOnContact {
    pub _sound_map: Vec<(ContactType, SoundType)>,
}

impl SoundOnContact {
    pub fn new(sound_map: Vec<(ContactType, SoundType)>) -> Self {
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
    pub vel: Vec3,
    pub drag: f32,
}

pub struct MovementAbility {
    pub top_speed: f32,
}

pub struct ControlRandomMovement {
    pub timer: Timer,
}

pub struct ControlRandomItemBasics {
    pub timer: Timer,
}

pub struct Carried {
    pub owner: Entity,
    pub offset: Transform,
}

pub struct Thrown {
    pub vel: Vec3,
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
            throw: true,
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

pub fn dress_stone(atlas: Handle<TextureAtlas>) -> impl DynamicBundle {
    SpriteSheetBundle {
        texture_atlas: atlas,
        sprite: TextureAtlasSprite {
            index: 3,
            color: Color::GRAY,
        },
        ..Default::default()
    }
}

pub fn dress_mage(atlas: Handle<TextureAtlas>) -> impl DynamicBundle {
    SpriteSheetBundle {
        texture_atlas: atlas,
        sprite: TextureAtlasSprite {
            index: 24 + 48,
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
    can_be_item_query: Query<&CanBeItemBasics>,
    not_carried_items: Query<(Entity, &CanBeItemBasics), Without<Carried>>,
) {
    let dt = time.delta_seconds();
    let mut pickups = None;

    for (owner, mut control, mut can) in active_query.iter_mut() {
        if control.timer.tick(dt).finished() {
            if let Some(item) = can.picked_up {
                if let Ok(can_be) = can_be_item_query.get(item) {
                    let mut rng = rand::thread_rng();
                    let drop = can.drop && can_be.drop;
                    let throw = can.throw && can_be.throw;

                    if (drop && throw && rng.gen()) || (drop && !throw) {
                        can.picked_up = None;
                        commands.remove_one::<Carried>(item);
                    } else if throw {
                        can.picked_up = None;
                        commands.insert_one(item, Thrown::new(rng.random_vec2d() * 40.0));
                        commands.remove_one::<Carried>(item);
                    }
                } else {
                    can.picked_up = None;
                }
            } else if can.pick_up {
                let pickups = pickups
                    .get_or_insert_with(|| shuffled_pickable_items(&mut not_carried_items.iter()));

                if let Some(item) = pickups.pop() {
                    let offset = Transform::from_translation(Vec3::new(0.0, 6.0, 0.0));
                    can.picked_up = Some(item.clone());
                    commands.insert_one(item, Carried { owner, offset });
                }
            }
        }
    }
}

fn shuffled_pickable_items(
    items: &mut dyn Iterator<Item = (Entity, &CanBeItemBasics)>,
) -> Vec<Entity> {
    let mut rng = rand::thread_rng();
    let mut pickups: Vec<Entity> = items.filter_map(|(e, c)| c.pick_up.then_some(e)).collect();
    pickups.sort_unstable_by(|_, _| {
        if rng.gen() {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
    pickups
}

pub fn carry_system(
    commands: &mut Commands,
    mut query: Query<(Entity, &Carried, Mut<Transform>)>,
    transform: Query<&Transform, Without<Carried>>,
) {
    for (item, carried, mut trans) in query.iter_mut() {
        if let Ok(owner_trans) = transform.get(carried.owner) {
            trans.translation = owner_trans.translation + carried.offset.translation;
        } else {
            commands.remove_one::<Carried>(item);
        }
    }
}

pub fn throw_system(commands: &mut Commands, mut query: Query<(Entity, &Thrown, Mut<Kinematics>)>) {
    for (entity, thrown, mut kin) in query.iter_mut() {
        kin.vel += thrown.vel;
        commands.remove_one::<Thrown>(entity);
    }
}

pub fn add_camera(commands: &mut Commands) {
    commands.spawn({
        let mut bundle = Camera2dBundle::default();
        bundle.transform.scale = Vec3::new(0.25, 0.25, 1.0);
        bundle
    });
}

pub trait RandomVec {
    fn random_vec2d(&mut self) -> Vec3;
}

impl RandomVec for rand::ThreadRng {
    fn random_vec2d(&mut self) -> Vec3 {
        Vec3::new(-0.5 + self.gen::<f32>(), -0.5 + self.gen::<f32>(), 0.0).normalize()
    }
}
