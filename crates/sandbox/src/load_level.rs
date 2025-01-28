use crate::vehicle_spawner;

use super::rapier_vehicle_controller::VehicleControllerParameters;
use super::vehicle_spawner::VehicleType;
use bevy::prelude::*;
use bevy_editor_cam::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::control::WheelTuning;

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        EditorCam::default(),
        Transform::from_xyz(0.0, -5.0, 3.0).looking_at(Vec3::new(0.0, 0.0, 0.3), Vec3::Z),
    ));
    commands.spawn((DirectionalLight {
        shadows_enabled: true,
        ..default()
    },));

    // Ground

    let ground_size = 200.1;
    let ground_height = 0.1;

    let floor = meshes.add(Cuboid::from_size(
        Vec2::splat(ground_size * 2f32).extend(ground_height * 2f32),
    ));
    let material = materials.add(Color::WHITE);

    commands.spawn((
        Mesh3d(floor),
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(0.0, 0.0, -ground_height),
        Collider::cuboid(ground_size, ground_size, ground_height),
    ));

    // Vehicles

    let wheel_tuning = WheelTuning {
        suspension_stiffness: 100.0,
        suspension_damping: 10.0,
        ..WheelTuning::default()
    };
    vehicle_spawner::spawn(VehicleType::Bulldozer, &mut commands, &asset_server)
        .insert(Transform::from_translation(Vec3::ZERO))
        .insert(
            VehicleControllerParameters::empty()
                .with_wheel_positions_for_half_size(Vec3::new(0.5, 0.5, 0.2))
                .with_wheel_tuning(wheel_tuning),
        );

    vehicle_spawner::spawn(VehicleType::Excavator, &mut commands, &asset_server)
        .insert(Transform::from_translation(Vec3::new(-4f32, 0f32, 0f32)))
        .insert(
            VehicleControllerParameters::empty()
                .with_wheel_positions_for_half_size(Vec3::new(0.5, 0.5, 0.2))
                .with_wheel_tuning(wheel_tuning),
        );

    vehicle_spawner::spawn(VehicleType::Truck, &mut commands, &asset_server)
        .insert(Transform::from_translation(Vec3::new(4f32, 0f32, 0f32)))
        .insert(
            VehicleControllerParameters::empty()
                .with_wheel_positions_for_half_size(Vec3::new(0.5, 0.5, 0.2))
                .with_wheel_tuning(wheel_tuning),
        );
}
