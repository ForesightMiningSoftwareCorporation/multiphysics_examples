use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_editor_cam::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_rapier3d::{prelude::*, rapier::prelude::IntegrationParameters};
use controls::ControlsPlugin;
use dotenvy::dotenv;
use map_def::rock::Rock;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rapier_vehicle_controller::debug::VehicleControllerDebugPlugin;
use vehicle_spawner::scoop::ScoopPlugin;

pub mod controls;
pub mod load_level;
pub mod rapier_vehicle_controller;
pub mod vehicle_spawner;

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
        map_def::MapDefPlugin,
        ControlsPlugin,
        ScoopPlugin,
    ))
    .add_plugins(EguiPlugin);

    let seeded_rng = ChaCha8Rng::seed_from_u64(4);
    app.insert_resource(RandomSource(seeded_rng));

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

#[derive(Resource)]
struct RandomSource(pub ChaCha8Rng);
