/*
    Think about Z.
    Apply more like a fantasy setting to it.
    Micro manage actions of your underlings.
    But at the same time you need to
     take care of some Sim like needs of your people
     and some minor building tasks, like chairs and torches or something.
*/

use bevy::{
    ecs::DynamicBundle, input::system::exit_on_esc_system, math::Vec3Swizzles, prelude::*,
    render::camera::Camera,
};
use bevy_rapier2d::{
    na::{Isometry2, Point2, Vector2},
    physics::{
        EventQueue, JointBuilderComponent, RapierConfiguration, RapierPhysicsPlugin,
        RigidBodyHandleComponent,
    },
    rapier::{
        dynamics::{FixedJoint, RigidBodyBuilder, RigidBodyHandle, RigidBodySet},
        geometry::{ColliderBuilder, ColliderHandle, InteractionGroups, Proximity},
    },
};
use level2::TileMapSpawner;

use crate::{
    bevy_rapier_utils::IntoVector2,
    bitpack::{Bitpack, BitpackPlugin},
    bundle_utils::sprite_bundle,
    commands_ext::CommandsExt,
    level2::{self, TileBundle, TileMap, TileMapLoader, TileMapSpawnEvent},
    rapier_debug_render::rapier_debug_render,
    utils::*,
};

pub fn app() -> AppBuilder {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_plugin(BitpackPlugin)
        .add_plugin(RapierPhysicsPlugin)
        //
        .add_startup_system(setup.system())
        //
        .add_system(level2::sync_tilemap_spawner_system.system())
        .add_asset::<TileMap>()
        .init_asset_loader::<TileMapLoader>()
        .add_event::<TileMapSpawnEvent>()
        //
        .add_system_to_stage(stage::PRE_UPDATE, tilemap_spawn_events_handler.system())
        .add_system(spawn_dress.system())
        .add_system(spawn_physics.system())
        .add_system(camera_tracks_player.system())
        .add_system(player_input.system())
        .add_system(player_sense_area.system())
        .add_system(funny_kinematics.system())
        .add_system(rapier_debug_render.system())
        .add_system(sync_parent_body_transform.system())
        .add_system(exit_on_esc_system.system());
    app
}

fn setup(
    mut config: ResMut<RapierConfiguration>,
    mut clear_color: ResMut<ClearColor>,
    asset_server: Res<AssetServer>,
    commands: &mut Commands,
) {
    config.gravity = Vector2::new(0.0, 0.0);
    clear_color.0 = Color::rgb(0.278, 0.176, 0.235);

    asset_server.watch_for_changes().unwrap();

    let tilemap_handle: Handle<TileMap> = asset_server.load("level4.tilemap");

    let tilemap_bundle = (
        Transform::from_translation(Vec3::new(-64.0, 64.0, 0.0)),
        GlobalTransform::default(),
        TileMapSpawner::new(tilemap_handle),
        Children::default(),
    );

    commands.spawn(tilemap_bundle);

    let camera = commands.entity({
        let mut bundle = Camera2dBundle::default();
        bundle.transform.scale = Vec3::new(0.5, 0.5, 1.0);
        bundle
    });

    commands.insert_resource(PlayerCamera(camera));

    let font = asset_server.load("FiraSans-Bold.ttf");

    commands.spawn(CameraUiBundle::default());

    let text = commands.entity(TextBundle {
        text: Text {
            value: "Press E".into(),
            style: TextStyle {
                font_size: 32.0,
                ..Default::default()
            },
            font,
        },
        style: Style {
            align_self: AlignSelf::FlexEnd,
            ..Default::default()
        },
        ..Default::default()
    });

    commands.insert_resource(PlayerSensingUi(text));
}

struct PlayerCamera(Entity);
struct PlayerSensingUi(Entity);

#[derive(Copy, Clone, Debug)]
enum Marker {
    Wall,
    Chair,
    Table,
    Window,
    Door,
    Bookshelf,
    Mirror,
    Oven,
    Bed,
    Dirt,
    RandomTree,
    PlayerSpawn,
    Torch,
    DebugSensor,
}

const TILE_MARKER_MAP: &[(char, Marker)] = {
    use Marker::*;
    &[
        ('#', Wall),
        ('W', Window),
        ('D', Door),
        ('c', Chair),
        ('=', Table),
        ('.', Dirt),
        ('b', Bookshelf),
        ('t', Mirror),
        ('B', Bed),
        ('f', Torch),
        ('A', RandomTree),
        ('P', PlayerSpawn),
        ('o', Oven),
        ('o', DebugSensor),
    ]
};

trait Level4Commands {
    fn spawn_tile(&mut self, tile: TileBundle);
    fn spawn_marker(&mut self, marker: Marker, bundle: impl DynamicBundle + Send + Sync + 'static);
}

impl Level4Commands for Commands {
    fn spawn_tile(&mut self, tile: TileBundle) {
        for (_, marker) in TILE_MARKER_MAP
            .iter()
            .filter(|(char, _)| *char == tile.0 .0 as char)
        {
            println!("spawn {:?}", marker);
            self.spawn_marker(*marker, tile);
        }
    }

    fn spawn_marker(&mut self, marker: Marker, bundle: impl DynamicBundle + Send + Sync + 'static) {
        self.spawn(bundle).with(marker);

        let desc = PhysicalDesc {
            size: Vec2::new(16.0, 16.0),
        };

        match marker {
            Marker::Wall => self
                .with(Dress::Bitpack(826, Color::GRAY))
                .with(Physics::SolidTile(desc)),
            Marker::Chair => self
                .with(Dress::Bitpack(385, Color::SALMON))
                .with(Physics::DynamicBall(desc)),
            Marker::Table => self
                .with(Dress::Bitpack(386, Color::SALMON))
                .with(Physics::SolidTile(desc)),
            Marker::Window => self
                .with(Dress::Bitpack(827, Color::GRAY))
                .with(Physics::SolidTile(desc)),
            Marker::Door => self.with(Dress::Bitpack(9 * 48 + 6, Color::GRAY)),
            Marker::Bookshelf => self
                .with(Dress::Bitpack(7 * 48 + 3, Color::SALMON))
                .with(Physics::SolidTile(desc)),
            Marker::Mirror => self
                .with(Dress::Bitpack(8 * 48, Color::SALMON))
                .with(Physics::SolidTile(desc)),
            Marker::Oven => self
                .with(Dress::Bitpack(8 * 48 + 8, Color::SALMON))
                .with(Physics::SolidTile(desc)),
            Marker::Bed => self
                .with(Dress::Bitpack(8 * 48 + 5, Color::SALMON))
                .with(Physics::SolidTile(desc)),
            Marker::Dirt => self.with(Dress::Bitpack(3, Color::SALMON)),
            Marker::RandomTree => self
                .with(Dress::Bitpack(
                    [48, 49, 50, 51, 52, 53, 99, 100].random(),
                    Color::rgb(0.22, 0.851, 0.451),
                ))
                .with(Physics::SolidTile(desc)),
            Marker::PlayerSpawn => self
                .with(PlayerSpawn)
                .with(Dress::Bitpack(25, Color::ORANGE))
                .with(Physics::DynamicBallRotLocked(desc)),
            Marker::Torch => self.with(Dress::Bitpack(15 * 48 + 4, Color::YELLOW)),
            Marker::DebugSensor => self.with(Physics::StaticSensor(PhysicalDesc {
                size: Vec2::new(32.0, 32.0),
            })),
            //_ => self.despawn(entity),
        };
    }
}

#[derive(Copy, Clone, Debug)]
enum Dress {
    Bitpack(u32, Color),
}

#[derive(Copy, Clone, Debug)]
enum Physics {
    SolidTile(PhysicalDesc),
    DynamicBall(PhysicalDesc),
    DynamicBallRotLocked(PhysicalDesc),
    StaticSensor(PhysicalDesc),
}

#[derive(Copy, Clone, Debug)]
struct PhysicalDesc {
    pub size: Vec2,
}

fn spawn_dress(
    commands: &mut Commands,
    bitpack: Res<Bitpack>,
    query: Query<(Entity, &Dress, &Transform, &GlobalTransform), Changed<Dress>>,
) {
    use Dress::*;

    for (entity, &dress, trans, gtrans) in query.iter() {
        let atlas = bitpack.atlas_handle.clone();

        match dress {
            Bitpack(index, color) => {
                commands
                    .insert(entity, sprite_bundle(atlas, index, color))
                    .insert(entity, (*trans, *gtrans));
            }
        }
    }
}

fn spawn_physics(
    commands: &mut Commands,
    query: Query<(Entity, &Physics, &Transform), Added<Physics>>,
) {
    for (entity, physics, transform) in query.iter() {
        let user_data = entity.to_bits() as u128;
        commands.set_current_entity(entity);

        match physics {
            Physics::SolidTile(desc) => commands.with_bundle((
                RigidBodyBuilder::new_static()
                    .translation(transform.translation.x, transform.translation.y)
                    .user_data(user_data),
                ColliderBuilder::cuboid(desc.size.x * 0.5, desc.size.y * 0.5)
                    .collision_groups(InteractionGroups::new(0x0002, 0xffff)),
            )),
            Physics::DynamicBall(desc) => commands.with_bundle((
                RigidBodyBuilder::new_dynamic()
                    .translation(transform.translation.x, transform.translation.y)
                    .user_data(user_data)
                    .linear_damping(8.0)
                    .angular_damping(4.0),
                ColliderBuilder::ball(desc.size.x * 0.49)
                    .collision_groups(InteractionGroups::new(0x0002, 0xffff)),
            )),
            Physics::DynamicBallRotLocked(desc) => {
                commands.with_bundle((
                    RigidBodyBuilder::new_dynamic()
                        .translation(transform.translation.x, transform.translation.y)
                        .user_data(user_data)
                        .linear_damping(8.0)
                        .lock_rotations(),
                    ColliderBuilder::ball(desc.size.x * 0.5)
                        .collision_groups(InteractionGroups::new(0x0001, 0xffff)),
                )).spawn((
                    Transform::default(),
                    GlobalTransform::default(),
                    RigidBodyBuilder::new_kinematic()
                        .translation(transform.translation.x, transform.translation.y)
                        .user_data(user_data),
                    ColliderBuilder::cuboid(desc.size.x * 1.7, desc.size.y * 1.7).sensor(true),
                    BodyTrack { entity, despawn_when_dangling: true },
                    //.collision_groups(InteractionGroups::new(0x0100, 0xffff)),
                ))
            }
            Physics::StaticSensor(desc) => commands.with_bundle((
                RigidBodyBuilder::new_static()
                    .translation(transform.translation.x, transform.translation.y)
                    .user_data(user_data),
                ColliderBuilder::cuboid(desc.size.x * 0.5, desc.size.y * 0.5).sensor(true),
            )),
        };
    }
}

struct BodyTrack {
    entity: Entity,
    despawn_when_dangling: bool,
}

fn sync_parent_body_transform(
    commands: &mut Commands,
    mut bodies: ResMut<RigidBodySet>,
    query: Query<(Entity, &BodyTrack, &RigidBodyHandleComponent)>,
    transform: Query<&GlobalTransform>,
) {
    for (entity, track, body) in query.iter() {
        if let Ok(transform) = transform.get(track.entity) {
            if let Some(body) = bodies.get_mut(body.handle()) {
                let pos = Isometry2::translation(transform.translation.x, transform.translation.y);
                body.set_position(pos, true);
            }
        } else if track.despawn_when_dangling {
            commands.despawn_recursive(entity);
        }
    }
}

fn tilemap_spawn_events_handler(
    commands: &mut Commands,
    mut event_reader: Local<EventReader<TileMapSpawnEvent>>,
    events: Res<Events<TileMapSpawnEvent>>,
) {
    for event in event_reader.iter(&events) {
        match event {
            TileMapSpawnEvent::Spawn(bundle) => commands.spawn_tile(*bundle),
            TileMapSpawnEvent::Despawn(a_tile) => {
                commands.despawn_recursive(*a_tile);
            }
        };
    }
}

struct PlayerSpawn;

fn camera_tracks_player(
    camera: Res<PlayerCamera>,
    query: Query<&GlobalTransform, With<PlayerSpawn>>,
    mut transform: Query<Mut<Transform>, With<Camera>>,
) {
    if let Ok(mut camera_trans) = transform.get_mut(camera.0) {
        let mut mid = camera_trans.translation.xy();

        for trans in query.iter() {
            mid = 0.5 * (mid + trans.translation.xy());
        }

        if camera_trans.translation.xy() != mid {
            let vec = camera_trans.translation.xy().lerp(mid, 0.125).round();
            camera_trans.translation.x = vec.x;
            camera_trans.translation.y = vec.y;
        }
    }
}

fn player_input(
    keys: Res<Input<KeyCode>>,
    //
    mut bodies: ResMut<RigidBodySet>,
    query: Query<&RigidBodyHandleComponent, With<PlayerSpawn>>,
) {
    let mut cursor = Vec3::default();

    if keys.pressed(KeyCode::W) {
        cursor.y += 1.0;
    }
    if keys.pressed(KeyCode::A) {
        cursor.x -= 1.0;
    }
    if keys.pressed(KeyCode::S) {
        cursor.y -= 1.0;
    }
    if keys.pressed(KeyCode::D) {
        cursor.x += 1.0;
    }

    let cursor = if cursor != Vec3::zero() {
        (70.0 * cursor.normalize()).into_vector2()
    } else {
        Vector2::new(0.0, 0.0)
    };

    for body in query.iter() {
        if let Some(body) = bodies.get_mut(body.handle()) {
            body.set_linvel(cursor, true);
        }
    }
}

fn player_sense_area(
    keys: Res<Input<KeyCode>>,
    events: Res<EventQueue>,
    mut ui: ResMut<PlayerSensingUi>,
    mut text: Query<Mut<Text>>,
    marker: Query<&Marker>,
    bodies: Res<RigidBodySet>,
) {
    if let Ok(text) = text.get_mut(ui.0) {
        if keys.just_pressed(KeyCode::E) {}
        if keys.just_released(KeyCode::E) {}

        if events.contact_events.len() != 0 && events.proximity_events.len() != 0 {
            println!(
                "c{:?} p{:?}",
                events.contact_events.len(),
                events.proximity_events.len()
            );
        }

        while let Ok(event) = events.proximity_events.pop() {
            match event.new_status {
                Proximity::Intersecting => {
                    println!("intersect");
                    if let Some(entity) = bodies.get_entity(event.collider1) {
                        if let Ok(marker) = marker.get(entity) {
                            println!("1 {:?}", marker);
                        }
                    }

                    if let Some(entity) = bodies.get_entity(event.collider2) {
                        if let Ok(marker) = marker.get(entity) {
                            println!("2 {:?}", marker);
                        }
                    }
                }
                Proximity::WithinMargin => {}
                Proximity::Disjoint => {}
            }
        }
    }
}

trait RigidBodySetExt {
    fn get_entity(&self, handle: ColliderHandle) -> Option<Entity>;
}

impl RigidBodySetExt for RigidBodySet {
    fn get_entity(&self, handle: ColliderHandle) -> Option<Entity> {
        self.get(handle)
            .map(|body| Entity::from_bits(body.user_data as u64))
    }
}

struct DebugEntity;

fn funny_kinematics(keys: Res<Input<KeyCode>>, query: Query<Entity, With<DebugEntity>>) {
    if keys.just_pressed(KeyCode::T) {
        for entity in query.iter() {}
    }
}
