use bevy::{
    asset::{
        io::{Reader, Writer},
        saver::{AssetSaver, SavedAsset},
        AssetLoader, AsyncWriteExt, LoadContext, RenderAssetUsages,
    },
    prelude::*,
    render::mesh::Indices,
};
use bevy_rapier3d::{prelude::Collider, rapier::prelude::HeightField};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::{fs::File, io::Write, path::Path};
use thiserror::Error;

use crate::{
    global_assets::GlobalAssets,
    rock::{Rock, SpawnRockCommand},
};

#[derive(Debug, Component, Reflect)]
pub struct MapDefHandle(pub Handle<MapDef>);

#[derive(Debug, Asset, Serialize, Deserialize, Reflect)]
pub struct MapDef {
    pub vertices_width: usize,
    pub vertices_length: usize,
    /// Y is scale in height, because parry uses Y-up.
    pub scale: Vec3,
    pub height_map: Vec<f32>,
    pub rocks: Vec<Isometry3d>,
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

impl Hash for MapDef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Destructuring to avoid forgetting to add new fields to the hash if the structure changes.
        let Self {
            vertices_width,
            vertices_length,
            scale,
            rocks,
            height_map,
        } = self;
        vertices_width.hash(state);
        vertices_length.hash(state);
        scale.x.to_bits().hash(state);
        scale.y.to_bits().hash(state);
        scale.z.to_bits().hash(state);
        for r in rocks.iter() {
            r.translation.x.to_bits().hash(state);
            r.translation.y.to_bits().hash(state);
            r.translation.z.to_bits().hash(state);
        }
        for f in height_map {
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

/// Implementation mostly from <https://github.com/bevyengine/bevy/blob/main/examples/asset/processing/asset_processing.rs>
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
        &["mapdef.ron"]
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

/// If an asset has been added or modified, notifies [`MapDefHandle`] change detection to call [`on_map_def_handle_changed`].
pub fn on_map_def_changed(
    mut scene_asset_event_reader: EventReader<AssetEvent<MapDef>>,
    mut map_def_instances: Query<&mut MapDefHandle>,
) {
    let mut map_def_to_initialize = vec![];
    for event in scene_asset_event_reader.read() {
        match dbg!(event) {
            AssetEvent::Added { id } => {
                map_def_to_initialize.push(*id);
            }

            AssetEvent::Modified { id } => {
                map_def_to_initialize.push(*id);
            }
            _ => {}
        }
    }
    for mut map_def_handle in map_def_instances.iter_mut() {
        if !map_def_to_initialize.contains(&map_def_handle.0.id()) {
            continue;
        }
        map_def_handle.set_changed();
    }
}

/// Actually initializes the map and creates the rock entities.
///
/// FIXME: ⚠️ it supports only 1 map, as it will remove all existing rocks.
/// A path to fixing that would be to use hierarchy or other ways to group rocks with their map.
pub fn on_map_def_handle_changed(
    mut commands: Commands,
    map_def_instances: Query<(Entity, Ref<MapDefHandle>), Changed<MapDefHandle>>,
    map_defs: Res<Assets<MapDef>>,
    mut meshes: ResMut<Assets<Mesh>>,
    global_assets: Res<GlobalAssets>,
    existing_rocks: Query<Entity, (With<Rock>, Without<MapDefHandle>)>,
) {
    for (e, map_def_handle) in map_def_instances.iter() {
        let Some(map_def): Option<&MapDef> = map_defs.get(&map_def_handle.0) else {
            continue;
        };
        commands.entity(e).despawn_descendants();
        // Clear previous rocks
        for e in existing_rocks.iter() {
            commands.entity(e).despawn();
        }
        // Create new rocks
        for r in map_def.rocks.iter() {
            commands.queue(SpawnRockCommand { isometry: *r });
        }
        // Create new invisible walls
        commands.entity(e).with_children(|child_builder| {
            let wall_depth_half = 10.0 / 2.0;
            let wall_height_half = (map_def.scale.z + 15.0) / 2.0;
            let collider_x = Collider::cuboid(
                map_def.scale.x / 2.0 + wall_depth_half,
                wall_height_half,
                wall_depth_half,
            );
            child_builder.spawn((
                Name::new("wall left"),
                collider_x.clone(),
                Transform::from_translation(Vec3::new(
                    0.0,
                    wall_height_half,
                    -map_def.scale.x / 2.0 - wall_depth_half,
                )),
            ));
            child_builder.spawn((
                Name::new("wall left"),
                collider_x.clone(),
                Transform::from_translation(Vec3::new(
                    0.0,
                    wall_height_half,
                    map_def.scale.x / 2.0 + wall_depth_half,
                )),
            ));
            let collider_z = Collider::cuboid(
                wall_depth_half,
                wall_height_half,
                map_def.scale.z / 2.0 + wall_depth_half,
            );

            child_builder.spawn((
                Name::new("wall back"),
                collider_z.clone(),
                Transform::from_translation(Vec3::new(
                    -map_def.scale.z / 2.0 - wall_depth_half,
                    wall_height_half,
                    0.0,
                )),
            ));
            child_builder.spawn((
                Name::new("wall front"),
                collider_z.clone(),
                Transform::from_translation(Vec3::new(
                    map_def.scale.z / 2.0 + wall_depth_half,
                    wall_height_half,
                    0.0,
                )),
            ));
        });

        let width = map_def.vertices_width;
        let length = map_def.vertices_length;
        let height = map_def.scale.y;
        let collider_ground = Collider::heightfield(
            map_def.height_map.clone(),
            width,
            length,
            // `Collider::heightfield` uses Y-up, we've rotated it.
            Vec3::new(map_def.scale.x, height, map_def.scale.z),
        );
        let height_field = collider_ground.as_heightfield().unwrap();
        let mut mesh = heightfield_to_bevy_mesh(height_field.raw);
        mesh.compute_normals();
        let mesh = meshes.add(mesh);
        commands.entity(e).insert((
            Mesh3d(mesh),
            MeshMaterial3d(global_assets.ground_material.clone_weak()),
            collider_ground,
        ));
    }
}

/// See [bevy_rapier#628](https://github.com/dimforge/bevy_rapier/pull/628) for more [Shape][`bevy_rapier3d::parry::shape::Shape`] to [`Mesh`] conversions.
pub fn heightfield_to_bevy_mesh(height_field: &HeightField) -> Mesh {
    let (vtx, idx) = height_field.to_trimesh();

    let mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(idx.into_iter().flatten().collect()));
    let mesh = mesh.with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vtx.iter()
            .map(|pos| [pos.x, pos.y, pos.z])
            .collect::<Vec<_>>(),
    );

    mesh
}
