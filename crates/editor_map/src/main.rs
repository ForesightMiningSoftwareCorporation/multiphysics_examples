use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
};

use bevy::{
    asset::io::file::FileAssetReader, input::common_conditions::input_just_pressed, prelude::*,
};
use bevy_editor_cam::prelude::*;
use bevy_egui::EguiContexts;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use dotenvy::dotenv;
use shared_map::{
    map_def::{MapDef, MapDefHandle, RockData},
    rock::{Rock, SpawnRockCommand},
};

fn main() {
    dotenv().expect(".env file not found");

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        MeshPickingPlugin,
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        DefaultEditorCamPlugins,
        shared_map::MapDefPlugin,
        bevy_egui::EguiPlugin,
        WorldInspectorPlugin::new(),
    ));

    app.add_systems(Startup, init_rapier_configuration);
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        update_rocks_and_export_map.run_if(input_just_pressed(KeyCode::KeyE)),
    );
    app.add_systems(Update, (on_map_def_handle_changed, ui_controls));
    app.run();
}

pub fn init_rapier_configuration(
    mut config: Query<&mut RapierConfiguration, With<DefaultRapierContext>>,
) {
    let mut config = config.single_mut();
    *config = RapierConfiguration {
        gravity: -Vec3::Z * 9.81,
        ..RapierConfiguration::new(1f32)
    };
}

pub fn setup(
    mut commands: Commands,
    // `_map_def` is useful if you want to create an asset procedurally.
    mut _map_def: ResMut<Assets<MapDef>>,
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
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/specular_rgb9e5_zstd.ktx2"),
            intensity: 500.0,
            ..default()
        },
        Transform::from_xyz(-63.0, 15.0, 58.0).looking_at(Vec3::new(0.0, 10.0, 0.3), Vec3::Z),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(1.0, -1.0, -1.0), Vec3::Z),
    ));

    // Ground

    /*
    // Create a ground procedurally ; hot reloading doesn't work this way:
    // you'd have to remap the handle to reference the exported file, rather than the in-memory asset.
    let width = 50;
    let length = 50;
    let height = 10f32;
    let mut map = commands.spawn((
        Transform::from_rotation(Quat::from_axis_angle(Vec3::X, 90f32.to_radians())),
        MapDefHandle(
            _map_def.add(MapDef {
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
                spawn_points: vec![],
            }),
        ),
    ));
    // */
    // /*
    // Alternatively, to load an existing map:
    let mut map = commands.spawn((
        Transform::default().with_rotation(
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        ),
        // Transform::from_rotation(
        //     Quat::from_axis_angle(Vec3::X, 90f32.to_radians())
        //         * Quat::from_axis_angle(Vec3::Y, 90f32.to_radians()),
        // ),
        MapDefHandle(asset_server.load("mapdef/final.mapdef.ron")),
        //MapDefHandle(asset_server.load("private/Sim data/transformed/imported_cubes.mapdef.ron")),
    ));
    // */
    map.observe(
        |trigger: Trigger<Pointer<Move>>,

         mut commands: Commands,

         inputs: Res<ButtonInput<KeyCode>>| {
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

/// Updates the rocks list then saves the [`MapDef`]s to a file.
/// If it's not already saved, it will be saved as `procedural_{hash}.mapdef.ron`.
///
/// FIXME: this doesn't support multiple maps, as all rocks will be associated with all maps.
/// Also, any modification to the instantiated height map will not be saved.
/// Worse, the height field will be reloaded to its previous state due to hot reloading trigger.
pub fn update_rocks_and_export_map(
    mut assets: ResMut<Assets<MapDef>>,
    q_map_def: Query<&MapDefHandle>,
    q_rocks: Query<&Transform, With<Rock>>,
) {
    for handle in q_map_def.iter() {
        let Some(map) = assets.get_mut(&handle.0) else {
            continue;
        };
        map.rocks = q_rocks
            .iter()
            .map(|t| RockData {
                translation: t.translation,
                // TODO: export metadata/grade..?
                metadata: 0,
            })
            .collect();
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
                    let path = format!("mapdef/procedural_{}.mapdef.ron", calculate_hash(map));
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

pub fn ui_controls(mut ctx: EguiContexts) {
    bevy_egui::egui::Window::new("Control").show(ctx.ctx_mut(), |ui| {
        ui.label("Press 'E' to export the map");
    });
}

pub fn on_map_def_handle_changed(
    mut commands: Commands,
    mut map_def_instances: Query<(Entity, &MapDefHandle), Changed<MapDefHandle>>,
    mut rocks: Query<Entity, With<Rock>>,
    map_defs: Res<Assets<MapDef>>,
) {
    for (e, map_def_handle) in map_def_instances.iter_mut() {
        let Some(map_def): Option<&MapDef> = map_defs.get(&map_def_handle.0) else {
            continue;
        };
        // remove walls
        commands.entity(e).despawn_descendants();
        for e in rocks.iter() {
            commands.entity(e).despawn();
        }
        for r in map_def.rocks.iter() {
            commands.queue(SpawnRockCommand {
                isometry: Isometry3d::new(r.translation, Quat::default()),
            });
        }
    }
}
