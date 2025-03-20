use bevy::{
    asset::{
        io::{Reader, Writer},
        saver::{AssetSaver, SavedAsset},
        AssetLoader, AsyncWriteExt, LoadContext, RenderAssetUsages,
    },
    prelude::*,
    render::mesh::Indices,
};
use bevy_rapier3d::{
    dynamics::RigidBody,
    prelude::{CollisionGroups, Group},
};
use bevy_rapier3d::{prelude::Collider, rapier::prelude::HeightField};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::{fs::File, io::Write, path::Path};
use thiserror::Error;

use crate::{global_assets::GlobalAssets, rock::Rock};
use bevy_wgsparkl::components::MpmCouplingEnabled;

#[derive(Debug, Component, Reflect)]
pub struct MapDefHandle(pub Handle<MapDef>);

/// FIXME: Currently not used, we should remove that.
/// Tunnelling can happen with heightfields, resulting in rocks falling through the ground.
/// To fix that, we can either add a context skin, or use continuous collision detection (see `bevy_rapier3d::SoftCcd`).
pub const CONTACT_SKIN: f32 = 0.0;

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct RockData {
    pub translation: Vec3,
    /// This could be the grade of the rock or an id... name them better and add more if needed :)
    pub metadata: u32,
}

#[derive(Debug, Clone, Component, Serialize, Deserialize, Reflect)]
pub struct MapLoaded;
#[derive(Debug, Clone, Default, Asset, Serialize, Deserialize, Reflect)]
pub struct MapDef {
    pub vertices_width: usize,
    pub vertices_length: usize,
    /// Y is scale in height, because parry uses Y-up.
    pub scale: Vec3,
    pub height_map: Vec<f32>,
    pub rocks: Vec<RockData>,
    pub spawn_point: Option<Vec3>,
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
            spawn_point,
        } = self;
        vertices_width.hash(state);
        vertices_length.hash(state);
        if let Some(spawn_point) = spawn_point {
            spawn_point.x.to_bits().hash(state);
            spawn_point.y.to_bits().hash(state);
            spawn_point.z.to_bits().hash(state);
        }
        scale.x.to_bits().hash(state);
        scale.y.to_bits().hash(state);
        scale.z.to_bits().hash(state);
        for RockData {
            translation,
            metadata,
        } in rocks.iter()
        {
            translation.x.to_bits().hash(state);
            translation.y.to_bits().hash(state);
            translation.z.to_bits().hash(state);
            metadata.hash(state);
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
        match event {
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

/// Actually initializes the map.
pub fn on_map_def_handle_changed(
    mut commands: Commands,
    mut map_def_instances: Query<
        (Entity, &mut Transform, Ref<MapDefHandle>),
        Changed<MapDefHandle>,
    >,
    map_defs: Res<Assets<MapDef>>,
    mut meshes: ResMut<Assets<Mesh>>,
    global_assets: Res<GlobalAssets>,
    existing_rocks: Query<Entity, (With<Rock>, Without<MapDefHandle>)>,
) {
    for (e, mut transform, map_def_handle) in map_def_instances.iter_mut() {
        let Some(map_def): Option<&MapDef> = map_defs.get(&map_def_handle.0) else {
            continue;
        };
        trace!("updating map def: {:?}", e);
        // remove walls
        commands.entity(e).despawn_descendants();
        // Clear previous rocks
        for e in existing_rocks.iter() {
            commands.entity(e).despawn();
        }

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
        // // Bumping mesh vertices up to avoid seeing a gap between the ground and the rocks.
        // if let VertexAttributeValues::Float32x3(values) =
        //     mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap()
        // {
        //     for pos in values {
        //         pos[1] += CONTACT_SKIN;
        //     }
        // }
        // Bumping the whole map down, so its top stays at z = 0.
        // TODO: x and y (z in ron) seems inverted..?
        transform.translation = Vec3::new(map_def.scale.z / 2.0, map_def.scale.x / 2.0, 0.0);
        transform.translation.z = -CONTACT_SKIN;

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

            // HACK: an invisible floor below the topography to catch particles passing in-between
            //       triangles.
            let ground_height = 1f32;
            let min_height = *map_def
                .height_map
                .iter()
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            let collider = Collider::cuboid(map_def.scale.x, ground_height, map_def.scale.z);
            child_builder.spawn((
                Name::new("floor bottom"),
                collider,
                Transform::from_translation(Vec3::new(
                    0.0,
                    -min_height * map_def.scale.y - ground_height / 2f32,
                    0.0,
                )),
                CollisionGroups::new(Group::GROUP_2, Group::ALL),
                MpmCouplingEnabled,
            ));
            if let Some(spawn_point) = map_def.spawn_point {
                dbg!(transform.translation);
                dbg!(transform.rotation);
                dbg!(transform.scale);
                dbg!(spawn_point);
                child_builder.spawn((
                    Mesh3d(global_assets.spawn_mesh.clone_weak()),
                    MeshMaterial3d(global_assets.spawn_material.clone_weak()),
                    Transform::from_translation(
                        // cancel out the map's transform.
                        transform.compute_affine().inverse().matrix3 * spawn_point,
                    ),
                ));
            }
        });
        mesh.compute_normals();
        let mesh = meshes.add(mesh);
        commands.entity(e).insert((
            Mesh3d(mesh),
            MeshMaterial3d(global_assets.ground_material.clone_weak()),
            RigidBody::Fixed,
            collider_ground,
            //ContactSkin(CONTACT_SKIN),
            MpmCouplingEnabled,
            MapLoaded,
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
