use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, TypeUuid, Default)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b71b5052"]
pub struct MapAsset {
    pub tiles: Vec<u8>,
    pub cols: u32,
    pub rows: u32,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct MapTile {
    pub tile: u8,
    pub col: u32,
    pub row: u32,
}

impl MapAsset {
    pub fn get(&self, col: u32, row: u32) -> Option<MapTile> {
        if col < self.cols && row < self.rows {
            let tile = self.tiles[(col + self.cols * row) as usize];
            Some(MapTile { tile, col, row })
        } else {
            None
        }
    }

    pub fn contains(&self, MapTile { tile, col, row }: MapTile) -> bool {
        col < self.cols && row < self.rows && self.tiles[(col + self.cols * row) as usize] == tile
    }
}

#[derive(Default)]
pub struct MapAssetLoader;

impl AssetLoader for MapAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let rows = bytes.iter().filter(|&&c| c == '\n' as u8).count() + 1;
            let cols = bytes
                .split(|&c| c == '\n' as u8)
                .map(|l| l.len())
                .max()
                .unwrap_or(0);
            let mut vec = Vec::with_capacity(rows * cols);

            let mut acc_cols = 0;
            for &c in bytes.iter() {
                if c == '\n' as u8 {
                    vec.extend(std::iter::repeat(' ' as u8).take(cols - acc_cols));
                    acc_cols = 0;
                } else {
                    vec.push(c);
                    acc_cols += 1;
                }
            }
            vec.extend(std::iter::repeat(' ' as u8).take(cols - acc_cols));

            load_context.set_default_asset(LoadedAsset::new(MapAsset {
                tiles: vec,
                cols: cols as u32,
                rows: rows as u32,
            }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["map"]
    }
}
