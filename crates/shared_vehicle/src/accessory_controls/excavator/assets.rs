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
    controls::{ExcavatorControls, ExcavatorControlsMapping},
    ExcavatorDef, ExcavatorDefHandle,
};

impl ExcavatorDef {
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
pub enum ExcavatorDefLoaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RonSpannedError(#[from] ron::error::SpannedError),
}

#[derive(Default)]
pub struct ExcavatorDefLoader;

/// Implementation mostly from <https://github.com/bevyengine/bevy/blob/main/examples/asset/processing/asset_processing.rs>
impl AssetLoader for ExcavatorDefLoader {
    type Asset = ExcavatorDef;
    type Settings = ();
    type Error = ExcavatorDefLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<ExcavatorDef, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let ron: ExcavatorDef = ron::de::from_bytes(&bytes)?;

        Ok(ron)
    }

    fn extensions(&self) -> &[&str] {
        &["excavatordef.ron", "excavatordef"]
    }
}

#[derive(Default)]
pub struct ExcavatorDefSaver;

impl AssetSaver for ExcavatorDefSaver {
    type Asset = ExcavatorDef;
    type Settings = ();
    type OutputLoader = ExcavatorDefLoader;
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

/// If an asset has been added or modified, notifies [`ExcavatorDefHandle`] change detection
/// to call [`set_default_controls`].
pub fn on_def_changed(
    mut event_reader: EventReader<AssetEvent<ExcavatorDef>>,
    mut excavatordef_instances: Query<&mut ExcavatorDefHandle>,
) {
    let mut excavator_def_to_update = vec![];
    for event in event_reader.read() {
        match event {
            AssetEvent::Added { id } => {
                excavator_def_to_update.push(*id);
            }

            AssetEvent::Modified { id } => {
                excavator_def_to_update.push(*id);
            }
            _ => {}
        }
    }
    for mut excavator_def_handle in excavatordef_instances.iter_mut() {
        if !excavator_def_to_update.contains(&excavator_def_handle.0.id()) {
            continue;
        }
        excavator_def_handle.set_changed();
    }
}

/// An observer to insert the [`ExcavatorControlsMapping`] to entities with [`ExcavatorDefHandle`].
pub fn update_excavator_control_mapping(
    trigger: Trigger<OnSceneReady>,
    mut commands: Commands,
    excavatordef_instances: Query<&ExcavatorDefHandle>,
    excavator_defs: Res<Assets<ExcavatorDef>>,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
) {
    let entity = trigger.entity();
    if let Ok(handle) = excavatordef_instances.get(entity) {
        let Some(def) = excavator_defs.get(&handle.0) else {
            return;
        };
        let mut mapping = ExcavatorControlsMapping {
            bucket_jaw: Entity::PLACEHOLDER,
            bucket_base: Entity::PLACEHOLDER,
            stick: Entity::PLACEHOLDER,
            boom: Entity::PLACEHOLDER,
            swing: Entity::PLACEHOLDER,
        };
        for e in children_query.iter_descendants(entity) {
            let Ok(name) = name_query.get(e) else {
                continue;
            };
            let name = name.as_str();
            if name == def.bucket_jaw.node_name {
                mapping.bucket_jaw = e;
                continue;
            } else if name == def.bucket_base.node_name {
                mapping.bucket_base = e;
                continue;
            } else if name == def.stick.node_name {
                mapping.stick = e;
                continue;
            } else if name == def.boom.node_name {
                mapping.boom = e;
                continue;
            } else if name == def.swing.node_name {
                mapping.swing = e;
                continue;
            }
        }
        commands.entity(entity).insert(mapping);
    }
}

pub fn set_default_controls(
    mut excavatordef_instances: Query<
        (&ExcavatorDefHandle, &mut ExcavatorControls),
        Changed<ExcavatorDefHandle>,
    >,
    excavator_defs: Res<Assets<ExcavatorDef>>,
) {
    for (handle, mut controls) in excavatordef_instances.iter_mut() {
        let Some(def) = excavator_defs.get(&handle.0) else {
            continue;
        };
        *controls = ExcavatorControls {
            bucket_jaw: def.bucket_jaw.get_default_knob(),
            bucket_base: def.bucket_base.get_default_knob(),
            stick: def.stick.get_default_knob(),
            boom: def.boom.get_default_knob(),
            swing: def.swing.get_default_knob(),
        };
    }
}
