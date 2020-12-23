/*
    mages, trees, stones are nice

    lets collide mages, stones and trees
    use physics to move and throw
    move towards stones

    I learned that in order to use rapier physics I need to manually put
    the entity int othe physics user_data to look it up again in the
    event handler.
*/

use bevy::{ecs::DynamicBundle, prelude::*};
use bevy_rapier2d::{
    na::{Isometry2, Vector2},
    physics::*,
    rapier::{dynamics::*, geometry::*},
    render::*,
};
use level1::{
    CanBeItemBasics, CanItemBasics, Carried, ContactType, ControlRandomMovement, Kinematics,
    MovementAbility, SoundOnContact, SoundType, Stone, Thrown,
};
use level2::{TileBundle, TileMap, TileMapLoader, TileMapSpawnEvent, TileMapSpawner};

use crate::{bitpack::Bitpack, bundle_utils::{sprite_bundle, static_tile_physics_bundle}, commands_ext::CommandsExt, level1::{self, RandomVec}, level2};

pub struct Level3Plugin;

impl Plugin for Level3Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app /**/
            .add_plugin(RapierPhysicsPlugin)
            //
            .add_startup_system(setup_physics.system())
            .add_startup_system(level1::add_camera.system())
            .add_startup_system(add_tilemap.system())
            //
            .add_system(spawn_from_tilemap.system())
            .add_system(control_random_movement_system.system())
            .add_system(level1::control_random_item_basics_system.system())
            .add_system(carry_system.system())
            .add_system(throw_system.system())
            // TODO this could be a tilemap plugin
            .add_system(level2::sync_tilemap_spawner_system.system())
            .add_asset::<TileMap>()
            .init_asset_loader::<TileMapLoader>()
            .add_event::<TileMapSpawnEvent>()
            /* end */;
    }
}

fn setup_physics(mut config: ResMut<RapierConfiguration>) {
    config.gravity = Vector2::new(0.0, 0.0);
}

trait IntoVector2 {
    fn into_vector2(self) -> Vector2<f32>;
}

impl IntoVector2 for Vec3 {
    fn into_vector2(self) -> Vector2<f32> {
        Vector2::new(self.x, self.y)
    }
}

fn throw_system(
    commands: &mut Commands,
    mut bodies: ResMut<RigidBodySet>,
    mut query: Query<(Entity, &Thrown, &RigidBodyHandleComponent)>,
) {
    for (entity, thrown, body_handle) in query.iter_mut() {
        if let Some(body) = bodies.get_mut(body_handle.handle()) {
            body.apply_impulse((thrown.vel * 50.0).into_vector2(), true);
            body.set_angvel(0.0, false);
        }
        commands.remove_one::<Thrown>(entity);
    }
}

fn carry_system(
    commands: &mut Commands,
    mut bodies: ResMut<RigidBodySet>,
    mut query: Query<(Entity, &Carried, &RigidBodyHandleComponent)>,
    transform: Query<&Transform, Without<Carried>>,
) {
    for (item, carried, handle) in query.iter_mut() {
        if let Ok(owner_trans) = transform.get(carried.owner) {
            if let Some(body) = bodies.get_mut(handle.handle()) {
                let pos = owner_trans.translation + carried.offset.translation;
                body.set_position(Isometry2::new(pos.into_vector2(), 0.0), false);
                body.set_angvel(0.0, false);
                body.set_linvel(Vector2::new(0.0, 0.0), false);
            }
        } else {
            commands.remove_one::<Carried>(item);
        }
    }
}

pub fn control_random_movement_system(
    time: Res<Time>,
    mut bodies: ResMut<RigidBodySet>,
    mut query: Query<(
        Mut<ControlRandomMovement>,
        Mut<RigidBodyHandleComponent>,
        &CanItemBasics,
        &MovementAbility,
    )>,
    items: Query<&Transform, With<CanBeItemBasics>>,
) {
    let mut iter = items.iter();
    let mut mid = iter.next().map(|it| it.translation).unwrap_or_default();
    for trans in items.iter() {
        mid = 0.5 * (mid + trans.translation);
    }

    let dt = time.delta_seconds();
    let mut rng = rand::thread_rng();
    for (mut control, body_handle, can, movement) in query.iter_mut() {
        if control.timer.tick(dt).finished() {
            if let Some(body) = bodies.get_mut(body_handle.handle()) {
                let top_speed = movement.top_speed;

                if can.picked_up.is_some() {
                    let vel = rng.random_vec2d() * top_speed * 0.8;
                    body.set_linvel(vel.into_vector2(), true);
                } else {
                    let trans = body.position().translation;
                    let dir = (mid - Vec3::new(trans.x, trans.y, 0.0)).normalize();
                    let vel = dir.lerp(rng.random_vec2d(), 0.4) * top_speed * 0.8;
                    body.set_linvel(vel.into_vector2(), true);
                }
            }
        }
    }
}

fn spawn_from_tilemap(
    commands: &mut Commands,
    bitpack: Res<Bitpack>,
    mut event_reader: Local<EventReader<TileMapSpawnEvent>>,
    events: Res<Events<TileMapSpawnEvent>>,
) {
    for event in event_reader.iter(&events) {
        match event {
            TileMapSpawnEvent::Spawn(bundle) => commands.tile_spawn(*bundle, &bitpack),
            TileMapSpawnEvent::Despawn(a_tile) => commands.despawn_recursive(*a_tile),
        };
    }
}

trait TileSpawn {
    fn tile_spawn(&mut self, tile_bundle: TileBundle, bitpack: &Res<Bitpack>) -> &mut Self;
}

impl TileSpawn for Commands {
    fn tile_spawn(&mut self, tile_bundle: TileBundle, bitpack: &Res<Bitpack>) -> &mut Self {
        let atlas = bitpack.atlas_handle.clone();
        let tile_char = tile_bundle.0 .0 as char;

        match tile_char {
            'M' => self
                .spawn(tile_bundle)
                .with_bundle(level2::mage_bundle())
                .entity_with_bundle(|e| mage_physics_bundle(e, tile_bundle.2))
                .with_child(level1::dress_mage(atlas)),
            '.' => self
                .spawn(tile_bundle)
                .with_bundle(stone_bundle())
                .entity_with_bundle(|e| stone_physics_bundle(e, tile_bundle.2))
                .with_child(level1::dress_stone(atlas)),
            'A' => self
                .spawn(sprite_bundle(atlas, 49, Color::DARK_GREEN))
                .with_bundle(tile_bundle)
                .entity_with_bundle(|e| static_tile_physics_bundle(e, tile_bundle.2)),
            'a' => self
                .spawn(sprite_bundle(atlas, 48, Color::DARK_GREEN))
                .with_bundle(tile_bundle)
                .entity_with_bundle(|e| static_tile_physics_bundle(e, tile_bundle.2)),
            _ => self,
        }
    }
}

fn mage_physics_bundle(entity: Entity, transform: Transform) -> impl DynamicBundle {
    (
        RigidBodyBuilder::new_dynamic()
            .translation(transform.translation.x, transform.translation.y)
            .lock_rotations()
            .user_data(entity.to_bits() as u128),
        ColliderBuilder::ball(4.0).collision_groups(InteractionGroups::new(0x0002, 0x0002)),
        RapierRenderColor(1.0, 0.0, 0.0),
    )
}

fn stone_physics_bundle(entity: Entity, transform: Transform) -> impl DynamicBundle {
    (
        RigidBodyBuilder::new_dynamic()
            .translation(transform.translation.x, transform.translation.y)
            .linear_damping(0.97)
            .user_data(entity.to_bits() as u128),
        ColliderBuilder::ball(3.0).collision_groups(InteractionGroups::new(0x0001, 0x0001)),
        RapierRenderColor(1.0, 0.0, 0.0),
    )
}

fn stone_bundle() -> impl DynamicBundle {
    use ContactType::*;
    use SoundType::*;

    (
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
        SoundOnContact::new(vec![(Ground, Clonk), (Wall, Bling)]),
    )
}

fn add_tilemap(asset_server: Res<AssetServer>, commands: &mut Commands) {
    asset_server.watch_for_changes().unwrap();

    let tilemap_handle: Handle<TileMap> = asset_server.load("level3.tilemap");

    let tilemap_bundle = (
        Transform::from_translation(Vec3::new(-64.0, 64.0, 0.0)),
        GlobalTransform::default(),
        TileMapSpawner::new(tilemap_handle),
        Children::default(),
    );

    commands.spawn(tilemap_bundle);
}
