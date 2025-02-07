use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::common_conditions::input_just_pressed,
    prelude::*,
};
use bevy_editor_cam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::{
    prelude::*,
    rapier::prelude::{DebugRenderPipeline, IntegrationParameters},
};
use controls::ControlsPlugin;
use dotenvy::dotenv;
use shared_map::rock::Rock;
use shared_vehicle::{
    excavator_controls::ExcavatorControlsPlugin,
    rapier_vehicle_controller::debug::VehicleControllerDebugPlugin,
    vehicle_spawner::{self, VehicleSpawnerPlugin},
};
use vehicle_spawner::scoop::ScoopPlugin;

pub mod controls;
pub mod load_level;

fn main() {
    dotenv().expect(".env file not found");

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        MeshPickingPlugin,
        RapierPhysicsPlugin::<NoUserData>::default()
            .with_custom_initialization(RapierContextInitialization::NoAutomaticRapierContext),
        RapierDebugRenderPlugin::default(),
        VehicleControllerDebugPlugin,
        DefaultEditorCamPlugins,
        shared_map::MapDefPlugin,
        VehicleSpawnerPlugin,
        ExcavatorControlsPlugin,
        ControlsPlugin,
        ScoopPlugin,
        WorldInspectorPlugin::new(),
        // Adds frame time diagnostics
        FrameTimeDiagnosticsPlugin,
        // Adds a system that prints diagnostics to the console
        LogDiagnosticsPlugin::default(),
    ));
    app.insert_resource(TimestepMode::Variable {
        max_dt: 1.0 / 14.0,
        time_scale: 1.0,
        substeps: 2,
    });
    let mut debug_render_pipeline = DebugRenderPipeline::default();
    debug_render_pipeline.mode = DebugRenderMode::default() | DebugRenderMode::SOLVER_CONTACTS;
    app.insert_resource(DebugRenderContext {
        enabled: true,
        default_collider_debug: ColliderDebug::default(),
        pipeline: debug_render_pipeline,
    });

    app.add_systems(PreStartup, init_rapier_context);
    app.add_systems(Startup, load_level::setup);

    app.add_systems(Update, add_scoopable_to_rocks);

    app.add_systems(
        Update,
        (|mut rapier_debug: ResMut<DebugRenderContext>| {
            rapier_debug.enabled = !rapier_debug.enabled;
        })
        .run_if(input_just_pressed(KeyCode::KeyD)),
    );

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
            force_update_from_transform_changes: true,
            ..RapierConfiguration::new(1f32)
        },
        DefaultRapierContext,
    ));
}

pub fn add_scoopable_to_rocks(
    mut commands: Commands,
    rocks: Query<Entity, (With<Rock>, Without<vehicle_spawner::scoop::Scoopable>)>,
) {
    for rock in rocks.iter() {
        commands
            .entity(rock)
            .insert(vehicle_spawner::scoop::Scoopable)
            .insert(ActiveEvents::COLLISION_EVENTS);
    }
}
