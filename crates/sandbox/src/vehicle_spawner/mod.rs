pub mod scoop;

use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_rapier3d::{prelude::{Collider, ColliderMassProperties, CollisionGroups, Group, MassProperties, RigidBody, Sensor}, rapier};
use scoop::{ScoopTarget, SensorStartScoop};

pub enum VehicleType {
    Bulldozer,
    Excavator,
    Truck,
}

pub fn spawn<'a>(
    vehicle_type: VehicleType,
    commands: &'a mut Commands,
    assets: &'a Res<AssetServer>,
) -> EntityCommands<'a> {
    match vehicle_type {
        VehicleType::Bulldozer => {
            // Bevy caches the assets so we can just load without any additional bookkeeping.
            let bulldozer = assets.load(
                GltfAssetLabel::Scene(0).from_asset("private/Bulldozer 3D Model/Bulldozer.glb"),
            );
            let chassis_dimensions = Vec3::new(1f32, 2f32, 0.4f32);
            let chassis_collider = Collider::cuboid(chassis_dimensions.x, chassis_dimensions.y, chassis_dimensions.z);
            let mut entity = commands.spawn((
                Visibility::default(),
                Transform::default(),
                chassis_collider,
                // mass is moved down, for a better adherence to the ground (also chains are heavier than the cabin)
                ColliderMassProperties::MassProperties(MassProperties {
                    local_center_of_mass: Vec3::new(0.0, 0.0, -1.0),
                    ..MassProperties::from_rapier(rapier::prelude::MassProperties::from_cuboid(0.8f32, chassis_dimensions.into()))
                }),
                RigidBody::Dynamic,
            ));
            // bulldozer front, to push rocks.
            entity.with_child((
                Transform::from_translation(Vec3::new(0.0, 2.5, -0.5)),
                Collider::cuboid(1f32, 0.4f32, 0.8f32),
                // no collision with ground
                CollisionGroups::new(Group::all(), Group::GROUP_1),
                // mass shouldn't impact too much or the vehicle will just fall towards its front.
                ColliderMassProperties::MassProperties(MassProperties {
                    local_center_of_mass: Vec3::new(0.0, -1.0, 0.0),
                    mass: 0.01,
                    principal_inertia: Vec3::ONE * 0.01,
                    ..default()
                })
            ));
            // Models are oftentimes not adapted to real usecase, rather than re-exporting a model,
            // we can adapt its scale, position, rotation by spawning it as a child.
            // for example, most models are provided with Y-up, but we're using Z-up.
            entity.with_child((
                SceneRoot(bulldozer.clone()),
                Transform::from_translation(Vec3::new(4.4, 0.0, 0.5))
                     .with_rotation(
                         Quat::from_axis_angle(Vec3::Z, TAU / 4.0) *
                         Quat::from_axis_angle(Vec3::X, TAU / 4.0),
                     )
                    .with_scale(Vec3::new(0.8, 0.8, 0.5)),
            ));
            
            entity
        }
        VehicleType::Excavator => {
            let chassis_dimensions = Vec3::new(1f32, 2f32, 0.4f32);
            let chassis_collider = Collider::cuboid(chassis_dimensions.x, chassis_dimensions.y, chassis_dimensions.z);
            let excavator =
                assets.load(GltfAssetLabel::Scene(0).from_asset("private/excavator/scene.gltf"));
            let mut entity = commands.spawn((
                Visibility::default(),
                Transform::default(),
                chassis_collider,
                // mass shouldn't impact too much or the vehicle will just fall towards its front.
                ColliderMassProperties::MassProperties(MassProperties {
                    local_center_of_mass: Vec3::new(0.0, 0.0, -1.0),
                    ..MassProperties::from_rapier(rapier::prelude::MassProperties::from_cuboid(0.8f32, chassis_dimensions.into()))
                }),
                RigidBody::Dynamic,
                SensorStartScoop,
            ));
            // Sensor to detect rocks, and move them to the truck.
            entity.with_child((
                Transform::from_translation(Vec3::new(0.0, 2.5, -0.5)),
                Sensor,
                Collider::cuboid(1f32, 0.4f32, 0.8f32),
                // no collision with ground
                CollisionGroups::new(Group::all(), Group::GROUP_1),
                // mass shouldn't impact too much or the vehicle will just fall towards its front.
                ColliderMassProperties::MassProperties(MassProperties {
                    local_center_of_mass: Vec3::new(0.0, -1.0, 0.0),
                    mass: 0.01,
                    principal_inertia: Vec3::ONE * 0.01,
                    ..default()
                }),
                SensorStartScoop
            ));
            // Model
            entity.with_child((
                SceneRoot(excavator.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, 1.2))
                    .with_scale(Vec3::new(2.0, 2.0, 2.0))
                    .with_rotation(
                        // Look up
                        Quat::from_axis_angle(Vec3::X, TAU / 4.0),
                    ),
            ));
            entity
        }
        VehicleType::Truck => {
            let truck =
                assets.load(GltfAssetLabel::Scene(0).from_asset("private/truck/scene.gltf"));
            let mut entity = commands.spawn((
                Visibility::default(),
                Transform::from_translation(Vec3::new(2f32, 0f32, 0f32)),
                Collider::cuboid(1f32, 1f32, 0.4f32),
                RigidBody::Dynamic,
            ));
            entity.with_children(|child_builder| {
                // loader to store rocks.

                // target for scoops
                child_builder.spawn((
                    Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
                    ScoopTarget {
                        possible_offset: Cuboid::new(0.5, 1.0, 0.1),
                    },
                ));
                // right wall
                child_builder.spawn((
                    Visibility::default(),
                    Transform::from_translation(Vec3::new(1.4f32, -0.7f32, 1.5f32)),
                    Collider::cuboid(0.2f32, 2.1f32, 1f32),
                ));
                // left wall
                child_builder.spawn((
                    Visibility::default(),
                    Transform::from_translation(Vec3::new(-1.4f32, -0.7f32, 1.5f32)),
                    Collider::cuboid(0.2f32, 2.1f32, 0.8f32),
                ));
                // front wall
                child_builder.spawn((
                    Visibility::default(),
                    Transform::from_translation(Vec3::new(0f32, 2f32, 1.5f32)),
                    Collider::cuboid(1.5f32,0.8f32, 1f32),
                ));
                // front inclined wall
                child_builder.spawn((
                    Visibility::default(),
                    Transform::from_translation(Vec3::new(0f32, 1.1f32, 1.55f32))
                    .with_rotation(Quat::from_rotation_x(-20f32.to_radians())),
                    Collider::cuboid(1.5f32,0.2f32, 0.8f32),
                ));
                // bottom wall
                child_builder.spawn((
                    Visibility::default(),
                    Transform::from_translation(Vec3::new(0f32, -1f32, 1.3f32))
                    .with_rotation(Quat::from_rotation_x(-10f32.to_radians())),
                    Collider::cuboid(1.5f32, 2f32, 0.1f32),
                ));
                // TODO: truck loader walls
                // TODO: component doing 1 or more overlap_box query to detect rocks and assess if truck is full.
                //   Counting rocks may be irrelevant because they may be arranged very differently.
            });
            // Model
            entity.with_child((
                SceneRoot(truck.clone()),
                Transform::from_translation(Vec3::new(0f32, 0f32, -0.4f32))
                    .with_scale(Vec3::new(0.005, 0.005, 0.005))
                    .with_rotation(
                        // Look back
                        Quat::from_axis_angle(Vec3::Y, TAU / 2.0) 
                        // look up
                        * Quat::from_axis_angle(Vec3::X, -TAU / 4.0)),
            ));
            entity
        }
    }
}
