use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
};

use bevy::{
    asset::io::file::FileAssetReader, input::common_conditions::input_just_pressed, prelude::*,
};
use bevy_editor_cam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::{prelude::*, rapier::prelude::IntegrationParameters};
use dotenvy::dotenv;
use shared_vehicle::{
    accessory_controls::{
        excavator::{controls::ExcavatorControls, ExcavatorDef, ExcavatorDefHandle},
        AccessoryControlsPlugin,
    },
    vehicle_spawner::{self, VehicleSpawnerPlugin, VehicleType},
};

fn main() {
    dotenv().expect(".env file not found");

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        MeshPickingPlugin,
        RapierPhysicsPlugin::<NoUserData>::default()
            .with_custom_initialization(RapierContextInitialization::NoAutomaticRapierContext),
        RapierDebugRenderPlugin::default(),
        DefaultEditorCamPlugins,
        AccessoryControlsPlugin,
        VehicleSpawnerPlugin,
        WorldInspectorPlugin::new(),
    ));

    app.add_systems(PreStartup, init_rapier_context);
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        export_excavator.run_if(input_just_pressed(KeyCode::KeyE)),
    );
    println!("\n\nInstructions:");
    println!("Press 'E' to export the excavator.");
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

pub fn setup(
    mut commands: Commands,
    // `_excavator_def` is useful if you want to create an asset procedurally.
    mut _excavator_def: ResMut<Assets<ExcavatorDef>>,
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
        Transform::default().looking_to(Vec3::new(1.0, -1.0, -1.0), Vec3::Z),
    ));

    /*
    // Create an excavator def through code
    let mut excavator_def = ExcavatorDefHandle(_excavator_def.add(ExcavatorDef {
        bucket_jaw: RotationControlDef {
            node_name: "HMS_bucket_jaws_JNT".to_string(),
            axis: Vec3::X,
            min_max_angle: Some(Vec2::new(0f32, 90f32.to_radians())),
            default_angle: 0.0,
            sensitivity: 1.0,
        },
        bucket_base: RotationControlDef {
            node_name: "HMS_bucket_bucket_JNT".to_string(),
            axis: Vec3::X,
            min_max_angle: Some(Vec2::new(0f32, 90f32.to_radians())),
            default_angle: 0.0,
            sensitivity: 1.0,
        },
        stick: RotationControlDef {
            node_name: "HMS_stick_JNT".to_string(),
            axis: Vec3::X,
            min_max_angle: Some(Vec2::new(0f32, 90f32.to_radians())),
            default_angle: 0.0,
            sensitivity: 1.0,
        },
        boom: RotationControlDef {
            node_name: "HMS_boom_JNT".to_string(),
            axis: Vec3::X,
            min_max_angle: Some(Vec2::new(0f32, 90f32.to_radians())),
            default_angle: 0.0,
            sensitivity: 1.0,
        },
        swing: RotationControlDef {
            node_name: "HMS_swing_drive".to_string(),
            axis: Vec3::Y,
            min_max_angle: None,
            default_angle: 0.0,
            sensitivity: 1.0,
        },
    })));
    // */
    // /*
    // Alternatively, to load an existing excavator:
    let excavator_def =
        ExcavatorDefHandle(asset_server.load("vehicledef/excavator.excavatordef.ron"));
    // */
    // spawn a vehicle for reference and testing
    vehicle_spawner::spawn(VehicleType::Excavator, &mut commands, &asset_server)
        .insert(excavator_def)
        .insert(ExcavatorControls::default())
        .insert(RigidBody::Fixed);
}

/// Saves the [`ExcavatorDef`]s to a file.
/// If it's not already saved, it will be saved as `procedural_{hash}.excavatordef.ron`.
pub fn export_excavator(
    mut assets: ResMut<Assets<ExcavatorDef>>,
    q_excavator_def: Query<&ExcavatorDefHandle>,
) {
    for handle in q_excavator_def.iter() {
        let Some(excavator) = assets.get_mut(&handle.0) else {
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
                        "vehicledef/procedural_{}.excavatordef.ron",
                        calculate_hash(excavator)
                    );
                    let mut p = PathBuf::new();
                    p.push(path);
                    p
                }),
        );
        let path = path.as_path();
        _ = std::fs::create_dir_all(path.parent().unwrap());
        if let Err(err) = excavator.save(path) {
            eprintln!("Failed to save the excavator to {:?}:\n{:#?}", path, err);
            continue;
        }

        println!("Saved the excavator to {:?}", path);
    }
}
