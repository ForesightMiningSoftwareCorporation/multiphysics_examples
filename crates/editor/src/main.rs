use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
};

use bevy::{
    asset::{
        io::{file::FileAssetReader, AssetSource},
        saver::ErasedAssetSaver,
        RenderAssetUsages,
    },
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::mesh::Indices,
};
use bevy_editor_cam::prelude::*;
use bevy_rapier3d::{
    parry::shape::HeightField, prelude::*, rapier::prelude::IntegrationParameters,
};
use map_def::{MapDef, MapDefHandle, MapDefLoader, MapDefSaver};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        MeshPickingPlugin,
        RapierPhysicsPlugin::<NoUserData>::default()
            .with_custom_initialization(RapierContextInitialization::NoAutomaticRapierContext),
        RapierDebugRenderPlugin::default(),
        DefaultEditorCamPlugins,
    ));
    app.init_asset::<MapDef>();
    app.register_asset_loader(MapDefLoader);

    app.add_systems(PreStartup, init_rapier_context);
    app.add_systems(Startup, setup);
    app.add_systems(Update, export_map.run_if(input_just_pressed(KeyCode::KeyE)));
    app.add_systems(Update, on_map_def_changed);
    println!("Press 'E' to export the map.");
    app.run();
}

pub fn init_rapier_context(mut commands: Commands) {
    let mut rapier_context = RapierContext::default();
    rapier_context.integration_parameters = IntegrationParameters {
        length_unit: 1f32,
        ..default()
    };
    commands.spawn((
        Name::new("Rapier Context"),
        rapier_context,
        RapierConfiguration {
            gravity: -Vec3::Z * 9.81,
            ..RapierConfiguration::new(1f32)
        },
        DefaultRapierContext,
    ));
}

pub fn setup(
    mut commands: Commands,
    mut map_def: ResMut<Assets<MapDef>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera3d::default(),
        EditorCam::default(),
        Transform::from_xyz(0.0, -5.0, 3.0).looking_at(Vec3::new(0.0, 0.0, 0.3), Vec3::Z),
    ));
    commands.spawn((DirectionalLight {
        shadows_enabled: true,
        ..default()
    },));

    // Ground

    let width = 100;
    let length = 100;
    let height = 10f32;

    commands.spawn(MapDefHandle(
        map_def.add(MapDef {
            vertices_width: width,
            vertices_length: length,
            scale: Vec3::new(width as f32, height, length as f32),
            height_map: (0..(width * length))
                .map(|i| {
                    let i = i as f32;
                    let length = length as f32;
                    let x = i % width as f32;
                    let y = i / width as f32;
                    let noise =
                        ((x * x) / (length * 10f32)).sin() * ((y * y) / (length * 10f32)).cos();
                    // make a step at half x
                    let noise = noise / 2f32;
                    let step = if x > (length / 2f32) { 0.5 } else { 0f32 };
                    noise + step
                })
                .collect::<Vec<_>>(),
        }),
    ));

    // Alternatively, to load an existing map:

    // commands.spawn(MapDefHandle(
    //     asset_server.load("mapdef/procedural_13494235624792029799.mapdef.ron"),
    // ));
}

pub fn on_map_def_changed(
    mut commands: Commands,
    mut scene_asset_event_reader: EventReader<AssetEvent<MapDef>>,
    map_def_instances: Query<(Entity, &MapDefHandle)>,
    map_defs: Res<Assets<MapDef>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut map_def_to_initialize = vec![];
    for event in scene_asset_event_reader.read() {
        match dbg!(event) {
            AssetEvent::Added { id } => {
                map_def_to_initialize.push(dbg!(*id));
            }
            AssetEvent::Modified { id } => {
                map_def_to_initialize.push(dbg!(*id));
            }
            AssetEvent::LoadedWithDependencies { id } => {
                map_def_to_initialize.push(dbg!(*id));
            }
            _ => {}
        }
    }

    for (e, map_def_handle) in map_def_instances.iter() {
        if map_def_to_initialize.contains(&map_def_handle.0.id()) {
            let Some(map_def): Option<&MapDef> = map_defs.get(&map_def_handle.0) else {
                continue;
            };
            // TODO: it's wasteful to recreate a material each time.
            let material = materials.add(Color::WHITE);

            let width = map_def.vertices_width;
            let length = map_def.vertices_length;
            let height = map_def.scale.y;
            let collider_ground = Collider::heightfield(
                map_def.height_map.clone(),
                width,
                length,
                // `Collider::heightfield` uses Y-up, we'll convert later.
                Vec3::new(100.0, height, 100.0),
            );
            let height_field = collider_ground.as_heightfield().unwrap();
            let mesh = heightfield_to_bevy_mesh(height_field.raw);
            let mesh = meshes.add(mesh);
            commands.entity(e).insert((
                Mesh3d(mesh),
                MeshMaterial3d(material.clone()),
                collider_ground,
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        }
    }
}

/// See [bevy_rapier#628](https://github.com/dimforge/bevy_rapier/pull/628) for more [Shape][`bevy_rapier3d::parry::shape::Shape`] to [`Mesh`] conversions.
pub fn heightfield_to_bevy_mesh(height_field: &HeightField) -> Mesh {
    let (vtx, idx) = height_field.to_trimesh();

    let mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(idx.iter().cloned().flatten().collect()));
    let mesh = mesh.with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vtx.iter()
            .map(|pos| [pos.x, pos.y, pos.z])
            .collect::<Vec<_>>(),
    );

    mesh
}

/// Save the map to a file.
/// If it's not already saved, it will be saved as `procedural_{hash}.mapdef.ron`.
pub fn export_map(assets: Res<Assets<MapDef>>, q_map_def: Query<&MapDefHandle>) {
    for handle in q_map_def.iter() {
        let Some(map): Option<&MapDef> = assets.get(&handle.0) else {
            continue;
        };
        let mut path = PathBuf::new();
        path.push(FileAssetReader::get_base_path());
        path.push("assets");
        path.push(
            handle
                .0
                .path()
                .map(|path| PathBuf::from(path.path()))
                .unwrap_or_else(|| {
                    fn calculate_hash<T: Hash>(t: &T) -> u64 {
                        let mut s = std::hash::DefaultHasher::new();
                        t.hash(&mut s);
                        s.finish()
                    }
                    let path = format!(
                        "mapdef/procedural_{}.mapdef.ron",
                        calculate_hash(map).to_string()
                    );
                    let mut p = PathBuf::new();
                    p.push(path);
                    p
                }),
        );
        let path = path.as_path();
        _ = std::fs::create_dir_all(path.parent().unwrap());
        if let Err(err) = map.save(path) {
            eprintln!("Failed to save the map to {:?}:\n{:#?}", path, err);
        }
    }
}
