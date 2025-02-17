use bevy::{
    asset::{
        io::{Reader, Writer},
        saver::{AssetSaver, SavedAsset},
        AssetLoader, AsyncWriteExt, LoadContext,
    },
    prelude::*,
};
use std::{fs::File, io::Write, path::Path};
use thiserror::Error;

use crate::vehicle_spawner::react_on_scene_instance_ready::OnSceneReady;

use super::{
    controls::{TruckControls, TruckControlsMapping},
    TruckDef, TruckDefHandle,
};

impl TruckDef {
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

#[derive(Debug, Error)]
pub enum TruckDefLoaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RonSpannedError(#[from] ron::error::SpannedError),
}

#[derive(Default)]
pub struct TruckDefLoader;

/// Implementation mostly from <https://github.com/bevyengine/bevy/blob/main/examples/asset/processing/asset_processing.rs>
impl AssetLoader for TruckDefLoader {
    type Asset = TruckDef;
    type Settings = ();
    type Error = TruckDefLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<TruckDef, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let ron: TruckDef = ron::de::from_bytes(&bytes)?;

        Ok(ron)
    }

    fn extensions(&self) -> &[&str] {
        &["truckdef.ron", "truckdef"]
    }
}

#[derive(Default)]
pub struct TruckDefSaver;

impl AssetSaver for TruckDefSaver {
    type Asset = TruckDef;
    type Settings = ();
    type OutputLoader = TruckDefLoader;
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

/// If an asset has been added or modified, notifies [`TruckDefHandle`] change detection
/// to call [`set_default_controls`].
pub fn on_def_changed(
    mut event_reader: EventReader<AssetEvent<TruckDef>>,
    mut truckdef_instances: Query<&mut TruckDefHandle>,
) {
    let mut truck_def_to_update = vec![];
    for event in event_reader.read() {
        match event {
            AssetEvent::Added { id } => {
                truck_def_to_update.push(*id);
            }

            AssetEvent::Modified { id } => {
                truck_def_to_update.push(*id);
            }
            _ => {}
        }
    }
    for mut truck_def_handle in truckdef_instances.iter_mut() {
        if !truck_def_to_update.contains(&truck_def_handle.0.id()) {
            continue;
        }
        truck_def_handle.set_changed();
    }
}

/// Inserts the [`TruckControlsMapping`] to entities with [`TruckDefHandle`].
pub fn update_truck_control_mapping(
    trigger: Trigger<OnSceneReady>,
    mut commands: Commands,
    truckdef_instances: Query<&TruckDefHandle>,
    truck_defs: Res<Assets<TruckDef>>,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
) {
    let entity = trigger.entity();
    if let Ok(handle) = truckdef_instances.get(entity) {
        let Some(def) = truck_defs.get(&handle.0) else {
            return;
        };
        let mut mapping = TruckControlsMapping {
            main_dump: Entity::PLACEHOLDER,
        };
        for e in children_query.iter_descendants(entity) {
            let Ok(name) = name_query.get(e) else {
                continue;
            };
            let name = name.as_str();
            if mapping.main_dump == Entity::PLACEHOLDER && name == def.main_dump.node_name {
                mapping.main_dump = e;
                continue;
            }
        }
        commands.entity(entity).insert(mapping);
    }
}

pub fn set_default_controls(
    mut truckdef_instances: Query<(&TruckDefHandle, &mut TruckControls), Changed<TruckDefHandle>>,
    truck_defs: Res<Assets<TruckDef>>,
) {
    for (handle, mut controls) in truckdef_instances.iter_mut() {
        let Some(def) = truck_defs.get(&handle.0) else {
            continue;
        };
        *controls = TruckControls {
            main_dump: def.main_dump.get_default_knob().current_value,
        };
    }
}
