use bevy::{
    asset::{
        io::{Reader, Writer},
        saver::{AssetSaver, SavedAsset},
        AssetLoader, AsyncWriteExt, LoadContext,
    },
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::{fs::File, io::Write, path::Path};
use thiserror::Error;

#[derive(Debug, Component, Reflect)]
pub struct MapDefHandle(pub Handle<MapDef>);

#[derive(Debug, Asset, Serialize, Deserialize, Reflect)]
pub struct MapDef {
    pub vertices_width: usize,
    pub vertices_length: usize,
    /// Y is scale in height, because parry uses Y-up.
    pub scale: Vec3,
    pub height_map: Vec<f32>,
}

impl Hash for MapDef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vertices_width.hash(state);
        self.vertices_length.hash(state);
        self.scale.x.to_bits().hash(state);
        self.scale.y.to_bits().hash(state);
        self.scale.z.to_bits().hash(state);
        for f in &self.height_map {
            f.to_bits().hash(state);
        }
    }
}

#[derive(Debug, Error)]
pub enum MapDefLoaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RonSpannedError(#[from] ron::error::SpannedError),
}

#[derive(Default)]
pub struct MapDefLoader;

/// Implementation mostly https://github.com/bevyengine/bevy/blob/main/examples/asset/processing/asset_processing.rs
impl AssetLoader for MapDefLoader {
    type Asset = MapDef;
    type Settings = ();
    type Error = MapDefLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<MapDef, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let ron: MapDef = ron::de::from_bytes(&bytes)?;

        Ok(ron)
    }

    fn extensions(&self) -> &[&str] {
        &["mapdef.ron", "mapdef"]
    }
}

impl MapDef {
    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut f = File::create(path)?;
        f.write_all(
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
        Ok(())
    }
}

#[derive(Default)]
pub struct MapDefSaver;

impl AssetSaver for MapDefSaver {
    type Asset = MapDef;
    type Settings = ();
    type OutputLoader = MapDefLoader;
    type Error = std::io::Error;

    async fn save(
        &self,
        writer: &mut Writer,
        asset: SavedAsset<'_, Self::Asset>,
        _settings: &Self::Settings,
    ) -> Result<(), Self::Error> {
        writer
            .write_all(
                ron::ser::to_string_pretty(asset.get(), ron::ser::PrettyConfig::default())
                    .unwrap()
                    .as_bytes(),
            )
            .await
            .unwrap();
        Ok(())
    }
}
