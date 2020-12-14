/*
    bring stone, mage and random controling from level1 and add

    a level2.map file with . for stones and M for mages.
    on map change the changed entities despawn and respawn.
*/

use bevy::prelude::*;

use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;

use crate::level1::*;

pub struct Level2Plugin;

impl Plugin for Level2Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup)
            .add_startup_system(add_camera)
            .add_system(kinematic_system)
            .add_system(control_random_movement_system)
            .add_system(control_random_item_basics_system)
            .add_system(carry_system)
            .add_system(throw_system)
            .add_system(sync_tilemap_spawner_system)
            .add_asset::<TileMap>()
            .init_asset_loader::<TileMapLoader>();
    }
}

pub fn setup(commands: &mut Commands, asset_server: Res<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    let tilemap_handle: Handle<TileMap> = asset_server.load("level2.tilemap");

    let level2_tilemap = (
        Transform::default(),
        GlobalTransform::default(),
        TileMapSpawner::new(tilemap_handle),
    );

    let map = [
        ('.', "stone"),
        ('M', "mage"),
    ];
    todo
    // add a resource which the tilemapspawner uses to map characters to entity factories

    commands.spawn(level2_tilemap);
}

struct TileMapSpawner {
    handle: Handle<TileMap>,
    width: f32,
    height: f32,
}

impl TileMapSpawner {
    fn new(handle: Handle<TileMap>) -> Self {
        Self {
            handle,
            width: 16.0,
            height: 16.0,
        }
    }
}

fn sync_tilemap_spawner_system(
    commands: &mut Commands,
    mut event_reader: Local<EventReader<AssetEvent<TileMap>>>,
    events: Res<Events<AssetEvent<TileMap>>>,
    tilemaps: Res<Assets<TileMap>>,
    spawner_query: Query<(Entity, &TileMapSpawner)>,
    tile_query: Query<(Entity, &Tile, &Parent)>,
) {
    let events = event_reader
        .iter(&events)
        .map(MyAssetEvent::from)
        .collect::<Vec<MyAssetEvent>>();

    //let entities = query.iter().collect::<Vec<_>>();

    for MyAssetEvent(event, handle) in events {
        match event {
            Event::Created => {
                let tilemap = tilemaps.get(handle.clone()).unwrap();

                for (entity, spawner) in spawner_query.iter().filter(|(_, t)| t.handle == handle) {
                    for tile in tilemap.map.iter() {
                        let trans = Vec3::new(
                            spawner.width * tile.1 as f32,
                            spawner.height * tile.2 as f32,
                            0.0,
                        );
                        commands.spawn((
                            tile.clone(),
                            Parent(entity),
                            Transform::from_translation(trans),
                            GlobalTransform::default(),
                        ));
                    }
                }
            }
            Event::Modified => {
                let tilemap = tilemaps.get(handle.clone()).unwrap();

                for (entity, spawner) in spawner_query.iter().filter(|(_, t)| t.handle == handle) {
                    let relevant = tile_query
                        .iter()
                        .filter(|(_, _, Parent(parent))| parent == &entity)
                        .collect::<Vec<_>>();

                    relevant
                        .iter()
                        .filter(|(_, tile, _)| !tilemap.map.contains(tile))
                        .for_each(|(tile_entity, _, _)| {
                            commands.despawn_recursive(*tile_entity);
                        });

                    let existing_tiles: Vec<_> =
                        relevant.iter().map(|(_, &tile, _)| tile).collect();

                    tilemap
                        .map
                        .iter()
                        .filter(|tile| !existing_tiles.contains(tile))
                        .for_each(|tile| {
                            let trans = Vec3::new(
                                spawner.width * tile.1 as f32,
                                spawner.height * tile.2 as f32,
                                0.0,
                            );
                            commands.spawn((
                                tile.clone(),
                                Parent(entity),
                                Transform::from_translation(trans),
                                GlobalTransform::default(),
                            ));
                        })
                }
            }
            Event::Removed => (),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Deserialize, Default, Clone, Copy)]
struct Tile(u8, u32, u32);

enum Event {
    Created,
    Modified,
    Removed,
}

struct MyAssetEvent(Event, Handle<TileMap>);

impl From<&AssetEvent<TileMap>> for MyAssetEvent {
    fn from(event: &AssetEvent<TileMap>) -> Self {
        match event {
            AssetEvent::Created { handle } => MyAssetEvent(Event::Created, handle.clone()),
            AssetEvent::Modified { handle } => MyAssetEvent(Event::Modified, handle.clone()),
            AssetEvent::Removed { handle } => MyAssetEvent(Event::Removed, handle.clone()),
        }
    }
}

#[derive(Debug, Deserialize, TypeUuid, Default)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b71b5051"]
struct TileMap {
    map: Vec<Tile>,
}

#[derive(Default)]
pub struct TileMapLoader;

impl AssetLoader for TileMapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            println!("TileMapLoader reload");
            let mut map = vec![];
            let mut col = 0;
            let mut row = 0;

            for &c in bytes.iter() {
                if c == '\n' as u8 {
                    col = 0;
                    row += 1;
                } else if c != ' ' as u8 {
                    map.push(Tile(c, col, row));
                    col += 1;
                }
            }

            load_context.set_default_asset(LoadedAsset::new(TileMap { map }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tilemap"]
    }
}
