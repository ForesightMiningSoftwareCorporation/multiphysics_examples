use bevy::asset::load_internal_asset;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::common_conditions::{input_just_pressed, input_toggle_active},
    prelude::*,
};
use bevy_editor_cam::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::{
    prelude::*,
    rapier::prelude::{DebugRenderPipeline, IntegrationParameters},
};
use bevy_wgsparkl::instancing3d::INSTANCING_SHADER_HANDLE;
use controls::ControlsPlugin;
use dotenvy::dotenv;
use load_level::add_muck_pile_for_excavator;
use shared_map::global_assets::{init_global_assets, GlobalAssets};
use shared_map::rock::Rock;
use shared_vehicle::{
    accessory_controls::AccessoryControlsPlugin,
    rapier_vehicle_controller::debug::VehicleControllerDebugPlugin,
    vehicle_spawner::{self, VehicleSpawnerPlugin},
};
use ui_gizmo_toggle::UiGizmoToggle;
use vehicle_spawner::scoop::ScoopPlugin;

pub mod controls;
pub mod load_level;
pub mod mpm;
pub mod muck_pile;
pub mod stats_rocks;
pub mod ui_gizmo_toggle;

fn main() {
    dotenv().expect("\n.env file not found. Please copy and adapt the .env.example\n\nError");

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        MeshPickingPlugin,
        DefaultEditorCamPlugins,
        // Adds frame time diagnostics
        FrameTimeDiagnosticsPlugin,
        // Adds a system that prints diagnostics to the console
        LogDiagnosticsPlugin::default(),
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        (
            VehicleControllerDebugPlugin,
            shared_map::MapDefPlugin,
            VehicleSpawnerPlugin,
            AccessoryControlsPlugin,
            ControlsPlugin,
            // FIXME: These are CPU implementations, not compatible with wgsparkl.
            // ScoopPlugin,
            // StatsRocksPlugin,
        ),
        bevy_egui::EguiPlugin,
        WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        UiGizmoToggle,
        bevy_wgsparkl::instancing3d::ParticlesMaterialPlugin,
    ));

    load_internal_asset!(
        app,
        INSTANCING_SHADER_HANDLE,
        "../../bevy_wgsparkl/src/instancing3d.wgsl",
        Shader::from_wgsl
    );

    app.insert_resource(TimestepMode::Variable {
        max_dt: 1.0 / 60.0,
        time_scale: 1.0,
        // Substep should be 1, to play well with kinematic position based
        // (otherwise, bevy_rapier would move the body at the first half step, and not the second, resulting in a too quick movement)
        substeps: 1,
    });
    let mut debug_render_pipeline = DebugRenderPipeline::default();
    debug_render_pipeline.mode = DebugRenderMode::default();
    app.insert_resource(DebugRenderContext {
        enabled: false,
        default_collider_debug: ColliderDebug::default(),
        pipeline: debug_render_pipeline,
    });

    app.add_systems(Startup, init_rapier_configuration);
    app.add_systems(
        Startup,
        (
            init_global_assets.run_if(|res: Option<Res<GlobalAssets>>| res.is_none()),
            load_level::spawn_level,
            bevy_wgsparkl::startup::setup_app,
        )
            .chain(),
    );

    app.add_systems(Update, add_scoopable_to_rocks);
    app.add_systems(Update, add_muck_pile_for_excavator);
    app.add_systems(Update, bevy_wgsparkl::step::step_simulation);
    app.add_systems(
        Update,
        (
            crate::mpm::setup_mpm_particles,
            bevy_wgsparkl::startup::setup_graphics,
        )
            .chain(),
    );

    app.add_systems(
        Update,
        (|mut rapier_debug: ResMut<DebugRenderContext>| {
            rapier_debug.enabled = !rapier_debug.enabled;
        })
        .run_if(input_just_pressed(KeyCode::KeyD)),
    );

    app.run();
}

pub fn init_rapier_configuration(
    mut config: Query<&mut RapierConfiguration, With<DefaultRapierContext>>,
) {
    let mut config = config.single_mut();
    *config = RapierConfiguration {
        gravity: -Vec3::Z * 9.81,
        force_update_from_transform_changes: true,
        ..RapierConfiguration::new(1f32)
    };
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
