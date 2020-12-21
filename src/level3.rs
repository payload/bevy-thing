/*
    mages, trees, stones are nice but lets get proximity to the stones
    before picking them up.

    random movement on mages, but only pick up when near
    use rapier
*/

use bevy::prelude::*;
use bevy_rapier2d::{
    na::Point2,
    physics::*,
    rapier::{dynamics::*, geometry::*},
    render::*,
};
use level1::{CanBeItemBasics, ContactType, Kinematics, SoundOnContact, SoundType, Stone};
use level2::{MageBundle, TileBundle, TileMap, TileMapLoader, TileMapSpawnEvent, TileMapSpawner};

use crate::{bitpack::Bitpack, level1, level2};

pub struct Level3Plugin;

impl Plugin for Level3Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app /**/
            .add_plugin(RapierPhysicsPlugin)
            .add_plugin(RapierRenderPlugin)
            //
            .add_startup_system(level1::add_camera.system())
            .add_startup_system(add_physics_example.system())
            .add_startup_system(add_tilemap.system())
            //
            .add_system(print_events.system())
            .add_system(spawn_from_tilemap.system())
            // TODO tilemap plugin
            .add_system(level2::sync_tilemap_spawner_system.system())
            .add_asset::<TileMap>()
            .init_asset_loader::<TileMapLoader>()
            .add_event::<TileMapSpawnEvent>()
            //
            .add_system(level1::control_random_movement_system.system())
            .add_system(level1::kinematic_system.system())
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

fn spawn_from_tilemap(
    commands: &mut Commands,
    bitpack: Res<Bitpack>,
    mut event_reader: Local<EventReader<TileMapSpawnEvent>>,
    events: Res<Events<TileMapSpawnEvent>>,
) {
    for event in event_reader.iter(&events) {
        match event {
            TileMapSpawnEvent::Spawn(bundle) => tilebundle_spawn(*bundle, commands, &bitpack),
            TileMapSpawnEvent::Despawn(a_tile) => tilebundle_despawn(*a_tile, commands),
        };
    }
}

fn tilebundle_spawn(bundle: TileBundle, commands: &mut Commands, bitpack: &Res<Bitpack>) {
    match bundle.0 .0 as char {
        'M' => tilebundle_spawn_mage(bundle, commands, bitpack),
        '.' => tilebundle_spawn_stone(bundle, commands, bitpack),
        'A' => tilebundle_spawn_sprite(bundle, commands, bitpack, 49, Color::DARK_GREEN),
        'a' => tilebundle_spawn_sprite(bundle, commands, bitpack, 48, Color::DARK_GREEN),
        _ => (),
    }
}

fn tilebundle_despawn(a_tile: Entity, commands: &mut Commands) {
    commands.despawn_recursive(a_tile);
}

fn tilebundle_spawn_sprite(
    bundle: TileBundle,
    commands: &mut Commands,
    bitpack: &Res<Bitpack>,
    index: u32,
    color: Color,
) {
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: bitpack.atlas_handle.clone(),
            sprite: TextureAtlasSprite { index, color },
            ..Default::default()
        })
        .with_bundle(bundle);
}

fn tilebundle_spawn_mage(bundle: TileBundle, commands: &mut Commands, bitpack: &Res<Bitpack>) {
    commands
        .spawn(bundle)
        .with_bundle(MageBundle::new())
        .with_children(|child| {
            child.spawn(level1::dress_mage(bitpack.atlas_handle.clone()));
        });
}

fn tilebundle_spawn_stone(bundle: TileBundle, commands: &mut Commands, bitpack: &Res<Bitpack>) {
    use ContactType::*;
    use SoundType::*;

    commands
        .spawn(bundle)
        .with_bundle((
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
        ))
        .with_children(|child| {
            child.spawn(level1::dress_stone(bitpack.atlas_handle.clone()));
        });
}

fn add_physics_example(commands: &mut Commands, mut _config: ResMut<RapierConfiguration>) {
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
