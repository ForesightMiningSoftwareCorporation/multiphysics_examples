use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_editor_cam::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::prelude::IntegrationParameters};
use dotenvy::dotenv;
use map_def::rock::Rock;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rapier_vehicle_controller::{VehicleController, VehicleControllerParameters};
use vehicle_spawner::scoop::ScoopPlugin;

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
        DefaultEditorCamPlugins,
        map_def::MapDefPlugin,
        ScoopPlugin,
    ));
    app.register_type::<VehicleControllerParameters>();

    let seeded_rng = ChaCha8Rng::seed_from_u64(4);
    app.insert_resource(RandomSource(seeded_rng));

    app.add_systems(PreStartup, init_rapier_context);
    app.add_systems(Startup, load_level::setup);

    app.add_systems(Update, init_vehicle_controller);
    app.add_systems(
        FixedUpdate,
        (update_vehicle_controls, update_vehicle_controller).chain(),
    );
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

/// System to initialize and insert a [`VehicleController`] after bevy_rapier initializes the rigidbody.
pub fn init_vehicle_controller(
    mut commands: Commands,
    vehicle: Query<
        (Entity, &VehicleControllerParameters, &RapierRigidBodyHandle),
        Added<RapierRigidBodyHandle>,
    >,
) {
    for (entity, vehicle_parameters, body_handle) in vehicle.iter() {
        let controller = VehicleController::new(body_handle.0, vehicle_parameters);
        commands.entity(entity).insert(controller);
    }
}

/// System to forward controls to [`VehicleController`]
pub fn update_vehicle_controls(
    inputs: Res<ButtonInput<KeyCode>>,
    mut vehicles: Query<(&mut VehicleController, &VehicleControllerParameters)>,
) {
    for (mut vehicle_controller, parameters) in vehicles.iter_mut() {
        vehicle_controller.integrate_actions(&inputs, parameters);
    }
}

/// System to initialize and insert a [`VehicleController`] after bevy_rapier initializes the rigidbody.
///
pub fn update_vehicle_controller(
    time: Res<Time>,
    mut vehicles: Query<&mut VehicleController>,
    mut context: WriteDefaultRapierContext,
) {
    for mut vehicle_controller in vehicles.iter_mut() {
        let context = &mut *context;
        vehicle_controller.update_vehicle(
            time.delta_secs(),
            &mut context.bodies,
            &context.colliders,
            &context.query_pipeline,
        );
    }
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
