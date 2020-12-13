use bevy::prelude::*;

use crate::{
    bitpack,
    bitpack::Bitpack,
    map_asset::{MapAsset, MapTile},
};

#[test]
fn public_interface() {
    App::build().add_plugin(BitpackMapPlugin);
}

pub struct BitpackMapPlugin;

impl Plugin for BitpackMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(sync_system);
    }
}

fn sync_system(
    commands: &mut Commands,
    // events
    mut event_reader: Local<EventReader<AssetEvent<MapAsset>>>,
    events: Res<Events<AssetEvent<MapAsset>>>,
    // state
    bitpack: Res<Bitpack>,
    // assets
    map_asset: Res<Assets<MapAsset>>,
    atlas_asset: Res<Assets<TextureAtlas>>,
    // queries
    maptile_query: Query<(Entity, &MapTile)>,
    // map_instances_query: Query<(&Transform, &Handle<MapAsset>)>,
) {
    let entities = maptile_query.iter().collect::<Vec<_>>();
    // let map_instances = map_instances_query.iter().collect::<Vec<_>>();
    let events = event_reader.iter(&events).collect::<Vec<_>>();

    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                let map = map_asset.get(handle).unwrap();
                sync_spawn(map, handle, &[], commands, &atlas_asset, &bitpack);
            }
            AssetEvent::Modified { handle } => {
                let map = map_asset.get(handle).unwrap();
                let remains = sync_despawn(&entities, map, commands);
                sync_spawn(map, handle, &remains, commands, &atlas_asset, &bitpack);
            }
            AssetEvent::Removed { handle: _ } => {
                sync_despawn(&entities, &MapAsset::default(), commands);
            }
        }
    }
}

/*
fn asset_event_handle(event: &AssetEvent<MapAsset>) -> Handle<MapAsset> {
    use AssetEvent::*;
    match event {
        Created { handle } => handle.clone(),
        Modified { handle } => handle.clone(),
        Removed { handle } => handle.clone(),
    }
}
*/

type Coord = (u32, u32);

fn sync_despawn(
    entities: &[(Entity, &MapTile)],
    map: &MapAsset,
    commands: &mut Commands,
) -> Vec<Coord> {
    let mut remains = vec![];

    for &(entity, &maptile) in entities.iter() {
        if map.contains(maptile) {
            remains.push((maptile.col, maptile.row));
        } else {
            commands.despawn_recursive(entity);
        }
    }

    remains
}

fn sync_spawn(
    map: &MapAsset,
    map_handle: &Handle<MapAsset>,
    existing: &[Coord],
    commands: &mut Commands,
    atlases: &Assets<TextureAtlas>,
    bitpack: &Bitpack,
) {
    for row in 0..map.rows {
        for col in 0..map.cols {
            if let Some(maptile) = map.get(col, row) {
                if maptile.tile != ' ' as u8 && !existing.contains(&(col, row)) {
                    spawn_map_tile(
                        maptile,
                        commands,
                        atlases.get_handle(&bitpack.atlas_handle),
                        map_handle.clone(),
                    );
                }
            }
        }
    }
}

fn spawn_map_tile(
    maptile: MapTile,
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
    map_handle: Handle<MapAsset>,
) {
    let (index, color) = get_index_color_from_tile(maptile.tile as char);
    commands
        .spawn((maptile, map_handle))
        .with_bundle(SpriteSheetBundle {
            texture_atlas,
            sprite: TextureAtlasSprite { index, color },
            transform: Transform {
                translation: Vec3::new(maptile.col as f32 * 16.0, maptile.row as f32 * -16.0, 0.0),
                scale: Vec3::splat(1.0),
                rotation: Quat::default(),
            },
            ..Default::default()
        });
}

fn get_index_color_from_tile(c: char) -> (u32, Color) {
    match c {
        'T' => (bitpack::TREE1 as u32, Color::GREEN),
        'P' => (bitpack::MAGICIAN1 as u32, Color::BLACK),
        _ => (0, Color::WHITE),
    }
}
