//! isolated example of a vehicle controller. it's taking the same parameters as the truck in the sandbox.

use bevy::prelude::*;
use bevy_rapier3d::{
    prelude::*,
    rapier::{control::WheelTuning, prelude::IntegrationParameters},
    render::RapierDebugRenderPlugin,
};
use shared_vehicle::rapier_vehicle_controller::{VehicleController, VehicleControllerParameters};

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        bevy_rapier3d::prelude::RapierPhysicsPlugin::<NoUserData>::default()
            .with_custom_initialization(RapierContextInitialization::NoAutomaticRapierContext),
        RapierDebugRenderPlugin::default(),
    ));
    app.add_systems(Startup, (init_rapier_context, setup));
    app.add_systems(
        FixedUpdate,
        (
            init_vehicle_controller,
            update_vehicle_controls,
            update_vehicle_controller,
        )
            .chain(),
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

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-23.0, 10.0, 18.0).looking_at(Vec3::new(0.0, 0.0, 1.3), Vec3::Z),
    ));

    // Ground
    commands.spawn((
        Collider::cuboid(100.0, 100.0, 1.0),
        Transform::from_translation(Vec3::Z - 1.5),
        RigidBody::Fixed,
    ));

    let wheel_tuning = WheelTuning {
        suspension_stiffness: 100.0,
        suspension_damping: 10.0,
        ..WheelTuning::default()
    };
    let truck_controller_parameters = VehicleControllerParameters {
        wheel_tuning,
        // truck has more mass and uses only 2 power wheels so more powerful wheels.
        engine_force: 20f32,
        wheel_brake: [1f32, 1f32],
        wheel_positions: [
            [-1.3, 1.6, 0.3].into(),
            [1.3, 1.6, 0.3].into(),
            [-1.3, -1.2, 0.3].into(),
            [1.3, -1.2, 0.3].into(),
        ],
        wheel_radius: 0.7,
        ..VehicleControllerParameters::empty()
    };
    let chassis_dimensions = Vec3::new(0.5, 1.0, 0.2);
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(
            chassis_dimensions.x,
            chassis_dimensions.y,
            chassis_dimensions.z,
        ),
        Transform::from_translation(Vec3::Z + 1.5),
        // mass is shifted down to avoid falling on its sides.
        ColliderMassProperties::MassProperties(MassProperties {
            local_center_of_mass: Vec3::new(0.0, 0.0, -1.0),
            ..MassProperties::from_rapier(
                bevy_rapier3d::rapier::prelude::MassProperties::from_cuboid(
                    2f32,
                    chassis_dimensions.into(),
                ),
            )
        }),
        truck_controller_parameters,
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

/// System to initialize and insert a [`VehicleController`] after bevy_rapier initializes the rigidbody.
///
pub fn update_vehicle_controller(
    time: Res<Time<Fixed>>,
    timestep_mode: Res<TimestepMode>,
    mut vehicles: Query<&mut VehicleController>,
    mut context: WriteDefaultRapierContext,
) {
    for mut vehicle_controller in vehicles.iter_mut() {
        let context = &mut *context;
        // capping delta time to max_dt, or we'll issue a move command that is too big,
        // resulting in an excavator difficult to control.
        let dt = match *timestep_mode {
            TimestepMode::Variable { max_dt, .. } => time.delta_secs().min(max_dt),
            _ => time.delta_secs(),
        };
        vehicle_controller.update_vehicle(
            dt,
            &mut context.bodies,
            &context.colliders,
            &context.query_pipeline,
        );
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
