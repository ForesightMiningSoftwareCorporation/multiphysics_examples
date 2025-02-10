use crate::controls::CurrentSelection;

use bevy::prelude::*;
use bevy_editor_cam::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::control::WheelTuning};
use shared_map::{map_def::MapDefHandle, rock::SpawnRockCommand};
use shared_vehicle::{
    excavator_controls::{controls::ExcavatorControls, ExcavatorDefHandle},
    rapier_vehicle_controller::VehicleControllerParameters,
    vehicle_spawner::{self, VehicleType},
};

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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

    // Ground
    let mut map = commands.spawn(MapDefHandle(
        asset_server.load("mapdef/no_cubes.mapdef.ron"),
    ));
    map.insert((
        Transform::default().with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        CollisionGroups::new(Group::GROUP_2, Group::ALL),
    ))
    .observe(
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
                isometry: Isometry3d::new(position + normal * 3.0, Quat::default()),
            });
        },
    );

    // Vehicles

    let wheel_tuning = WheelTuning {
        suspension_stiffness: 100.0,
        suspension_damping: 10.0,
        ..WheelTuning::default()
    };

    let bulldozer_entity =
        vehicle_spawner::spawn(VehicleType::Bulldozer, &mut commands, &asset_server)
            .insert(
                Transform::from_translation(Vec3::new(-10.0, 15.0, 3.0))
                    .with_rotation(Quat::from_rotation_z(180f32.to_radians())),
            )
            .insert(
                VehicleControllerParameters::empty()
                    .with_wheel_positions_for_half_size(Vec3::new(0.5, 1.0, 0.4))
                    .with_wheel_tuning(wheel_tuning)
                    .with_crawler(true),
            )
            .id();
    commands.insert_resource(CurrentSelection {
        entity: Some(bulldozer_entity),
    });

    let excavator_def =
        ExcavatorDefHandle(asset_server.load("vehicledef/excavator2.excavatordef.ron"));

    vehicle_spawner::spawn(VehicleType::Excavator2, &mut commands, &asset_server)
        .insert(
            Transform::from_translation(Vec3::new(0.0, 15.0, 3.0))
                .with_rotation(Quat::from_rotation_z(180f32.to_radians())),
        )
        .insert(
            VehicleControllerParameters::empty()
                .with_wheel_positions_for_half_size(Vec3::new(0.5, 0.5, 0.2))
                .with_wheel_tuning(wheel_tuning)
                .with_crawler(true),
        )
        .insert(excavator_def)
        .insert(ExcavatorControls::default());

    let truck_controller_parameters = VehicleControllerParameters {
        wheel_tuning,
        // truck has more mass so more powerful wheels.
        engine_force: 120f32,
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
    vehicle_spawner::spawn(VehicleType::Truck, &mut commands, &asset_server)
        .insert(
            Transform::from_translation(Vec3::new(10.0, 15.0, 3.0))
                .with_rotation(Quat::from_rotation_z(180f32.to_radians())),
        )
        .insert(truck_controller_parameters);
}
