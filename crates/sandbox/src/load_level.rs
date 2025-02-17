use crate::{controls::CurrentSelection, muck_pile::SpawnMuckPileCommand};

use bevy::{prelude::*, render::view::NoFrustumCulling};
use bevy_editor_cam::prelude::*;
use bevy_math::Vec3A;
use bevy_rapier3d::{prelude::*, rapier::control::WheelTuning};
use shared_map::{
    map_def::{MapDefHandle, CONTACT_SKIN},
    rock::SpawnRockCommand,
};
use shared_vehicle::{
    accessory_controls::{
        excavator::{
            controls::{ExcavatorControls, ExcavatorControlsMapping},
            ExcavatorDefHandle,
        },
        truck::{controls::TruckControls, TruckDefHandle},
    },
    rapier_vehicle_controller::VehicleControllerParameters,
    vehicle_spawner::{self, VehicleType},
};

pub fn spawn_level(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Ground
    let mut map = commands.spawn(MapDefHandle(
        asset_server.load("private/Sim data/transformed/imported_cubes.mapdef.ron"),
    ));
    // let mut map = commands.spawn(MapDefHandle(
    //     asset_server.load("mapdef/1000_cubes.mapdef.ron"),
    // ));

    map.insert((
        Transform::default().with_rotation(
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        ),
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
        suspension_stiffness: 80.0,
        suspension_damping: 10.0,
        ..WheelTuning::default()
    };

    // We'll spawn vehicles around this point.
    let spawn_point = Vec3::new(170.0, 120.0, 24.0);
    // TODO: consider changing the engine speed (or mass of objects) depending on the chosen scale.
    //
    let scale = 2.5f32;
    let mut bulldozer_parameters = VehicleControllerParameters::empty()
        .with_wheel_positions_for_half_size(Vec3::new(0.5, 1.5, 0.4), Vec3::Z * -CONTACT_SKIN)
        .with_wheel_tuning(wheel_tuning)
        .with_crawler(true);
    bulldozer_parameters.wheel_radius *= scale;
    bulldozer_parameters.engine_force = 100.0 * scale * scale;
    bulldozer_parameters
        .wheel_brake
        .iter_mut()
        .for_each(|w| *w = 14.0 * scale);
    bulldozer_parameters
        .wheel_positions
        .iter_mut()
        .for_each(|w| {
            *w *= scale;
        });
    let bulldozer_entity = vehicle_spawner::spawn(
        VehicleType::Bulldozer,
        &mut commands,
        &asset_server,
        Transform::from_translation(spawn_point + Vec3::Z)
            .with_rotation(Quat::from_rotation_z(180f32.to_radians()))
            .with_scale(Vec3::splat(scale)),
    )
    .insert(bulldozer_parameters)
    .id();
    commands.insert_resource(CurrentSelection {
        entity: Some(bulldozer_entity),
    });

    let excavator_def =
        ExcavatorDefHandle(asset_server.load("vehicledef/excavator.excavatordef.ron"));
    let mut excavator_parameters = VehicleControllerParameters::empty()
        .with_wheel_positions_for_half_size(Vec3::new(0.7, 1.0, 0.4), Vec3::Z * -CONTACT_SKIN)
        .with_wheel_tuning(wheel_tuning)
        .with_crawler(true);
    excavator_parameters.engine_force = 40.0 * scale * scale;
    excavator_parameters.wheel_radius *= scale;
    excavator_parameters
        .wheel_brake
        .iter_mut()
        .for_each(|w| *w = 2.0 * scale);
    excavator_parameters
        .wheel_positions
        .iter_mut()
        .for_each(|w| {
            *w *= scale;
        });
    vehicle_spawner::spawn(
        VehicleType::Excavator,
        &mut commands,
        &asset_server,
        Transform::from_translation(spawn_point + Vec3::new(10.0, 0.0, 0.0))
            .with_rotation(Quat::from_rotation_z(180f32.to_radians()))
            .with_scale(Vec3::splat(scale)),
    )
    .insert(excavator_parameters)
    .insert(excavator_def)
    .insert(ExcavatorControls::default());

    let mut truck_controller_parameters = VehicleControllerParameters {
        wheel_tuning,
        // truck has more mass and uses only 2 power wheels so more powerful wheels.
        engine_force: 400f32 * scale * scale,
        // rear wheel is always braking
        wheel_brake: [10.5f32 * scale * scale, 1.6f32 * scale * scale],
        wheel_positions: [
            Vec3::new(-1.3, 1.6, 0.3 - CONTACT_SKIN),
            Vec3::new(1.3, 1.6, 0.3 - CONTACT_SKIN),
            Vec3::new(-1.3, -1.2, 0.3 - CONTACT_SKIN),
            Vec3::new(1.3, -1.2, 0.3 - CONTACT_SKIN),
        ],
        wheel_radius: 0.7 * scale,
        ..VehicleControllerParameters::empty()
    };
    truck_controller_parameters
        .wheel_positions
        .iter_mut()
        .for_each(|w| {
            *w *= scale;
        });
    let truck_def = TruckDefHandle(asset_server.load("vehicledef/truck.truckdef.ron"));
    vehicle_spawner::spawn(
        VehicleType::Truck,
        &mut commands,
        &asset_server,
        Transform::from_translation(spawn_point - Vec3::new(10.0, 0.0, 0.0))
            .with_rotation(Quat::from_rotation_z(180f32.to_radians()))
            .with_scale(Vec3::splat(scale)),
    )
    .insert(truck_controller_parameters)
    .insert(truck_def)
    .insert(TruckControls::default())
    .with_children(|child_builder| {
        // muck pile in the truck
        child_builder.spawn(
            SpawnMuckPileCommand {
                local_aabb: bevy::render::primitives::Aabb {
                    center: Vec3A::new(0.0, 0.0, 1.5),
                    half_extents: Vec3A::new(3.0, 3.0, 2.0),
                },
                name: "Truck pile".to_string(),
                position: Isometry3d::default(),
            }
            .to_bundle_minimal(),
        );
    });

    // Muck piles
    commands.queue(SpawnMuckPileCommand {
        local_aabb: bevy::render::primitives::Aabb {
            center: Vec3A::new(0.0, 0.0, 0.5),
            half_extents: Vec3A::new(15.0, 15.0, 15.0),
        },
        name: "Muck Pile".to_string(),
        position: Isometry3d::from_translation(spawn_point + Vec3::new(-60.0, 0.0, 0.01 - 6.0)),
    });

    // Camera, light
    commands.spawn((
        Camera3d::default(),
        EditorCam {
            orbit_constraint: OrbitConstraint::Fixed {
                up: Vec3::Z,
                can_pass_tdc: false,
            },
            ..EditorCam::default()
        },
        Transform::from_translation(spawn_point + Vec3::new(-63.0, 15.0, 58.0))
            .looking_at(spawn_point + Vec3::new(0.0, 10.0, 0.3), Vec3::Z),
    ));
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(1.0, -1.0, -1.0), Vec3::Z),
    ));
}

pub fn add_muck_pile_for_excavator(
    mut commands: Commands,
    excavator_mapping: Query<(&Name, &ExcavatorControlsMapping), Added<ExcavatorControlsMapping>>,
) {
    for (name, mapping) in excavator_mapping.iter() {
        commands
            .spawn((
                SpawnMuckPileCommand {
                    local_aabb: bevy::render::primitives::Aabb {
                        // 300 because excavator has been rescaled..
                        center: Vec3A::new(0.0, 0.0, 300.0),
                        half_extents: Vec3A::new(0.75, 0.75, 0.75),
                    },
                    name: format!("{} pile", name.as_str()),
                    position: Isometry3d::default(),
                }
                .to_bundle_minimal(),
                NoFrustumCulling,
            ))
            .set_parent(mapping.bucket_base);
    }
}
