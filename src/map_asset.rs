use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, TypeUuid, Default)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b71b5052"]
pub struct MapAsset {
    pub tiles: Vec<char>,
    pub cols: u32,
    pub rows: u32,
}

impl MapAsset {
    pub fn get(&self, col: u32, row: u32) -> Option<char> {
        self.tiles.get((col + self.cols * row) as usize).cloned()
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
            let string = String::from_utf8_lossy(bytes);
            let rows = string.lines().count();
            let cols = string.lines().map(|l| l.len()).max().unwrap_or(0);
            let mut vec = Vec::with_capacity(rows * cols);

            let mut acc_cols = 0;
            for c in string.chars() {
                if c == '\n' {
                    vec.extend(std::iter::repeat(' ').take(cols - acc_cols));
                    acc_cols = 0;
                } else {
                    vec.push(c);
                    acc_cols += 1;
                }
            }
            vec.extend(std::iter::repeat(' ').take(cols - acc_cols));

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
