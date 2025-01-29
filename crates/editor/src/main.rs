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
    color::palettes,
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
        //RapierDebugRenderPlugin::default(),
        DefaultEditorCamPlugins,
    ));
    app.init_asset::<MapDef>();
    app.init_asset_loader::<MapDefLoader>();

    app.add_systems(PreStartup, init_rapier_context);
    app.add_systems(Startup, setup);
    app.add_systems(Startup, init_global_assets);
    app.add_systems(Update, export_map.run_if(input_just_pressed(KeyCode::KeyE)));
    app.add_systems(Update, on_map_def_changed);
    println!("\n\nInstructions:");
    println!("Press 'E' to export the map.");
    println!("Press 'C' and move your cursor on the ground to spawn rocks.");
    println!("\n\n");
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

#[derive(Debug, Default, Resource, Reflect)]
pub struct GlobalAssets {
    pub ground_material: Handle<StandardMaterial>,
    pub rock_material: Handle<StandardMaterial>,
    pub rock_mesh: Handle<Mesh>,
}

pub fn init_global_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let global_assets = GlobalAssets {
        ground_material: materials.add(Color::WHITE),
        rock_material: materials.add(Color::from(palettes::css::DARK_GRAY)),
        rock_mesh: meshes.add(Cuboid::new(0.2, 0.2, 0.2)),
    };
    commands.insert_resource(global_assets);
}

#[derive(Debug, Default, Reflect, Component)]
pub struct Rock;

pub fn setup(
    mut commands: Commands,
    mut map_def: ResMut<Assets<MapDef>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera3d::default(),
        EditorCam {
            orbit_constraint: OrbitConstraint::Fixed {
                up: Vec3::Z,
                can_pass_tdc: false,
            },
            ..EditorCam::default()
        },
        Transform::from_xyz(0.0, -5.0, 3.0).looking_at(Vec3::new(0.0, 0.0, 0.3), Vec3::Z),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(1.0, 1.0, -1.0), Vec3::Z),
    ));

    // Ground

    let width = 50;
    let length = 50;
    let height = 10f32;

    // Create a ground procedurally

    /*
    let mut map = commands.spawn(MapDefHandle(
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
                    noise * step + step
                })
                .collect::<Vec<_>>(),
            rocks: vec![],
        }),
    ));
    // */
    // /*
    // Alternatively, to load an existing map:
    let mut map = commands.spawn(MapDefHandle(asset_server.load("mapdef/final.mapdef.ron")));
    // */
    map.insert(
        Transform::default().with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    )
    .observe(
        |trigger: Trigger<Pointer<Move>>,
         mut commands: Commands,
         inputs: Res<ButtonInput<KeyCode>>,
         assets: Res<GlobalAssets>| {
            if !inputs.pressed(KeyCode::KeyC) {
                return;
            }
            let Some(position) = trigger.hit.position else {
                return;
            };
            let Some(normal) = trigger.hit.normal else {
                return;
            };
            commands.queue(SpawnRockCommand {
                isometry: Isometry3d::new(Vec3::from(position + normal * 3.0), Quat::default()),
            });
        },
    );
}

pub struct SpawnRockCommand {
    pub isometry: Isometry3d,
}

impl Command for SpawnRockCommand {
    fn apply(self, world: &mut World) {
        let assets = world.resource::<GlobalAssets>();
        world.spawn((
            Mesh3d(assets.rock_mesh.clone_weak()),
            MeshMaterial3d(assets.rock_material.clone_weak()),
            Collider::cuboid(0.1, 0.1, 0.1),
            RigidBody::Dynamic,
            Transform::from_isometry(self.isometry),
            PickingBehavior::IGNORE,
            Rock,
        ));
    }
}

pub fn on_map_def_changed(
    mut commands: Commands,
    mut scene_asset_event_reader: EventReader<AssetEvent<MapDef>>,
    map_def_instances: Query<(Entity, &MapDefHandle)>,
    map_defs: Res<Assets<MapDef>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_rocks: Query<Entity, (With<Rock>, Without<MapDefHandle>)>,
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

    for (e, map_def_handle) in map_def_instances.iter() {
        if map_def_to_initialize.contains(&map_def_handle.0.id()) {
            let Some(map_def): Option<&MapDef> = map_defs.get(&map_def_handle.0) else {
                continue;
            };
            // Clear previous rocks
            for e in existing_rocks.iter() {
                commands.entity(e).despawn();
            }
            // Create new rocks
            for r in map_def.rocks.iter() {
                commands.queue(SpawnRockCommand { isometry: *r });
            }

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
            let mut mesh = heightfield_to_bevy_mesh(height_field.raw);
            mesh.compute_normals();
            let mesh = meshes.add(mesh);
            commands.entity(e).insert((
                Mesh3d(mesh),
                MeshMaterial3d(material.clone()),
                collider_ground,
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
pub fn export_map(
    mut assets: ResMut<Assets<MapDef>>,
    q_map_def: Query<&MapDefHandle>,
    q_rocks: Query<&Transform, With<Rock>>,
) {
    for handle in q_map_def.iter() {
        let Some(map) = assets.get_mut(&handle.0) else {
            continue;
        };
        map.rocks = q_rocks.iter().map(Transform::to_isometry).collect();
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
            continue;
        }

        println!("Saved the map to {:?}", path);
    }
}
