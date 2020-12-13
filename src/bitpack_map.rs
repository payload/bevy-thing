use bevy::prelude::*;

use crate::{bitpack, bitpack::Bitpack, map_asset::MapAsset};

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

#[derive(Eq, PartialEq, Hash)]
struct MapTile {
    tile: String,
    col: u32,
    row: u32,
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
    query: Query<(Entity, &MapTile)>,
) {
    let entities = query.iter().collect::<Vec<_>>();

    for event in event_reader.iter(&events) {
        match event {
            AssetEvent::Created { handle } => {
                let map = map_asset.get(handle).unwrap();
                sync_spawn(map, &[], commands, &atlas_asset, &bitpack);
            }
            AssetEvent::Modified { handle } => {
                let map = map_asset.get(handle).unwrap();
                let remains = sync_despawn(&entities, map, commands);
                sync_spawn(map, &remains, commands, &atlas_asset, &bitpack);
            }
            AssetEvent::Removed { handle: _ } => {
                sync_despawn(&entities, &MapAsset::default(), commands);
            }
        }
    }
}

type Coord = (u32, u32);

fn sync_despawn(
    entities: &[(Entity, &MapTile)],
    map: &MapAsset,
    commands: &mut Commands,
) -> Vec<Coord> {
    let mut remains = vec![];

    for &(entity, maptile) in entities.iter() {
        if map
            .get(maptile.col, maptile.row)
            .map_or(false, |m| m.to_string() == maptile.tile)
        {
            remains.push((maptile.col, maptile.row));
        } else {
            commands.despawn_recursive(entity);
        }
    }

    remains
}

fn sync_spawn(
    MapAsset { tiles, cols, rows }: &MapAsset,
    existing: &[Coord],
    commands: &mut Commands,
    atlases: &Assets<TextureAtlas>,
    bitpack: &Bitpack,
) {
    for row in 0..*rows {
        for col in 0..*cols {
            if let Some(&tile) = tiles.get((col + row * cols) as usize) {
                if tile != ' ' && !existing.contains(&(col, row)) {
                    spawn_map_tile(
                        tile.to_string(),
                        col,
                        row,
                        commands,
                        atlases.get_handle(&bitpack.atlas_handle),
                    );
                }
            }
        }
    }
}

fn spawn_map_tile(
    tile: String,
    col: u32,
    row: u32,
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
) {
    let (index, color) = get_index_color_from_tile(tile.chars().next().unwrap());
    commands
        .spawn((MapTile { tile, col, row },))
        .with_bundle(SpriteSheetBundle {
            texture_atlas,
            sprite: TextureAtlasSprite { index, color },
            transform: Transform {
                translation: Vec3::new(col as f32 * 16.0, row as f32 * -16.0, 0.0),
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
