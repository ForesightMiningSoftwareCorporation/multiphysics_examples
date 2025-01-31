use crate::vehicle_spawner;

use super::rapier_vehicle_controller::VehicleControllerParameters;
use super::vehicle_spawner::VehicleType;
use bevy::prelude::*;
use bevy_editor_cam::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::control::WheelTuning;
use map_def::map_def::MapDefHandle;

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
        Transform::default().looking_to(Vec3::new(1.0, 1.0, -1.0), Vec3::Z),
    ));

    // Ground
    let mut map = commands.spawn(MapDefHandle(asset_server.load("mapdef/final.mapdef.ron")));
    map.insert((
        Transform::default().with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        CollisionGroups::new(Group::GROUP_2, Group::ALL),
    ));

    // Vehicles

    let wheel_tuning = WheelTuning {
        suspension_stiffness: 100.0,
        suspension_damping: 10.0,
        ..WheelTuning::default()
    };

    // vehicle_spawner::spawn(VehicleType::Bulldozer, &mut commands, &asset_server)
    //     .insert(Transform::from_translation(Vec3::new(0.0, 3.0, 3.0)))
    //     .insert(
    //         VehicleControllerParameters::empty()
    //             .with_wheel_positions_for_half_size(Vec3::new(0.5, 1.0, 0.4))
    //             .with_wheel_tuning(wheel_tuning)
    //             .with_crawler(true),
    //     );

    vehicle_spawner::spawn(VehicleType::Excavator, &mut commands, &asset_server)
        .insert(Transform::from_translation(Vec3::new(-4.0, 5.0, 3.0)))
        .insert(
            VehicleControllerParameters::empty()
                .with_wheel_positions_for_half_size(Vec3::new(0.5, 0.5, 0.2))
                .with_wheel_tuning(wheel_tuning)
                .with_crawler(true),
        );

    vehicle_spawner::spawn(VehicleType::Truck, &mut commands, &asset_server)
    .insert(Transform::from_translation(Vec3::new(4.0, 5.0, 3.0)))
    // .insert(
    //     VehicleControllerParameters::empty()
    //         .with_wheel_positions_for_half_size(Vec3::new(0.5, 0.5, 0.2))
    //         .with_wheel_tuning(wheel_tuning),
    // )
    ;
}
