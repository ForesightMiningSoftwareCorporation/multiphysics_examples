pub mod scoop;
pub mod react_on_scene_instance_ready;
pub mod follow;

use std::f32::consts::TAU;

use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::{plugin::WriteDefaultRapierContext, prelude::{ CoefficientCombineRule, Collider, ColliderMassProperties, CollisionGroups, ComputedColliderShape, Dominance, Group, MassProperties, Restitution, RigidBody, Sensor}, rapier::{self, prelude::RigidBodyBuilder}};
use follow::{CopyPosition, FollowPlugin};
use react_on_scene_instance_ready::{OnSceneReady, ReactOnSceneInstanceReady, ReactOnSceneInstanceReadyPlugin};
use scoop::{ScoopTarget, SensorStartScoop};

pub struct VehicleSpawnerPlugin;

impl Plugin for VehicleSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<VehicleType>();
        app.add_plugins(ReactOnSceneInstanceReadyPlugin);
        app.add_plugins(FollowPlugin);
    }
}

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleType {
    Bulldozer,
    Excavator,
    Excavator2,
    Truck,
}

impl std::fmt::Display for VehicleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
                Name::new("bulldozer"),
                vehicle_type,
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
                Name::new("bulldozer front"),
                Transform::from_translation(Vec3::new(0.0, 2.5, -0.5)),
                Collider::cuboid(1f32, 0.4f32, 0.8f32),
                // no collision with ground
                CollisionGroups::new(Group::all(), !Group::GROUP_2),
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
                Name::new("bulldozer model"),
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
                Name::new("excavator"),
                vehicle_type,
                Visibility::default(),
                Transform::default(),
                chassis_collider,
                // mass is shifted down to avoid falling on its sides.
                ColliderMassProperties::MassProperties(MassProperties {
                    local_center_of_mass: Vec3::new(0.0, 0.0, -1.0),
                    ..MassProperties::from_rapier(rapier::prelude::MassProperties::from_cuboid(0.8f32, chassis_dimensions.into()))
                }),
                RigidBody::Dynamic,
            ));
            // Sensor to detect rocks, and move them to the truck.
            entity.with_child((
                Name::new("scoop sensor"),
                Transform::from_translation(Vec3::new(0.0, 2.5, -0.5)),
                Sensor,
                Collider::cuboid(1f32, 0.4f32, 0.8f32),
                // no collision with ground
                CollisionGroups::new(Group::all(), Group::GROUP_1),
                // mass shouldn't be impacted as it's a sensor.
                ColliderMassProperties::Density(0f32),
                SensorStartScoop
            ));
            // Model
            entity.with_child((
                Name::new("excavator model"),
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
        VehicleType::Excavator2 => {
            let chassis_dimensions = Vec3::new(1f32, 2f32, 0.4f32);
            let chassis_collider = Collider::cuboid(chassis_dimensions.x, chassis_dimensions.y, chassis_dimensions.z);
            let excavator =
                assets.load(GltfAssetLabel::Scene(0).from_asset("private/excavator2/excavator2.gltf"));
            let mut entity = commands.spawn((
                Name::new("excavator2"),
                vehicle_type,
                Visibility::default(),
                Transform::default(),
                chassis_collider,
                // mass is shifted down to avoid falling on its sides.
                ColliderMassProperties::MassProperties(MassProperties {
                    local_center_of_mass: Vec3::new(0.0, 0.0, -1.0),
                    ..MassProperties::from_rapier(rapier::prelude::MassProperties::from_cuboid(0.8f32, chassis_dimensions.into()))
                }),
                CollisionGroups::new(Group::GROUP_3, !Group::GROUP_3),
                RigidBody::Dynamic,
                //Dominance::group(10)
            ));
            // Sensor to detect rocks, and move them to the truck.
            /*entity.with_child((
                Name::new("scoop sensor"),
                Transform::from_translation(Vec3::new(0.0, 2.5, -0.5)),
                Sensor,
                Collider::cuboid(1f32, 0.4f32, 0.8f32),
                // no collision with ground
                CollisionGroups::new(Group::all(), Group::GROUP_1),
                // mass shouldn't be impacted as it's a sensor.
                ColliderMassProperties::Density(0f32),
                SensorStartScoop
            ));*/
            let meshes_to_convert_to_collider: HashMap<String, Option<ComputedColliderShape>> = [
                // Boom
                ("Mesh.018".to_string(), Some(ComputedColliderShape::default())),
                // Bucket base
                ("Mesh.004".to_string(), Some(ComputedColliderShape::default())),
                // Bucket jaws
                ("Mesh.003".to_string(), Some(ComputedColliderShape::default())),
                // Rear chassis ; Consider replacing that with a cube
                //("Mesh.059".to_string(), Some(ComputedColliderShape::default())),
                // Stick
                ("Mesh.007".to_string(), Some(ComputedColliderShape::default())),
            ].into();
            // Model
            entity.with_children(| child_builder| {
                child_builder.spawn(
                (
                Name::new("excavator2 model"),
                SceneRoot(excavator.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, -0.5))
                    .with_scale(Vec3::new(0.4, 0.4, 0.4))
                    .with_rotation(
                        // Look back
                        Quat::from_axis_angle(Vec3::Z, TAU / 2.0) *
                        // Look up
                        Quat::from_axis_angle(Vec3::X, TAU / 4.0),
                    ),
                    ReactOnSceneInstanceReady,
                    // NOTE: Compute automatically colliders, we're only selecting a subset of the meshes for better performances.
                    // bevy_rapier3d::prelude::AsyncSceneCollider { shape: Some(ComputedColliderShape::default()), named_shapes: default() }
                    bevy_rapier3d::prelude::AsyncSceneCollider { shape: None, named_shapes: 
                        meshes_to_convert_to_collider.clone() },
            )).observe(move |
                trigger: Trigger<OnSceneReady>, mut commands: Commands, 
                q_children: 
                    Query<&Children>,
                q_parents: Query<&Parent>,
                q_names: Query<&Name>,| {
                    
                    for entity in q_children.iter_descendants(trigger.entity()) {
                        let Ok(name) = q_names.get(entity) else {
                            continue;
                        };
                        commands.entity(entity).insert(CollisionGroups::new(Group::GROUP_3, !Group::GROUP_3));
                        if meshes_to_convert_to_collider.contains_key(&name.to_string()) {
                            // We found a joint that we want to control,
                            // we're transforming it to an unparented kinematic rigidbody,
                            // to avoid mass impacting our vehicle collider.
                            // We still need that collider to have some mass to push rocks.
                            commands.entity(entity).insert(CopyPosition(q_parents.get(entity).unwrap().get()));
                            commands.entity(entity).remove_parent_in_place();
                            //commands.entity(entity).insert(Dominance::group(10));
                            commands.entity(entity).insert(RigidBody::KinematicPositionBased);
                            commands.entity(entity).insert(Restitution{ coefficient: 0.0, combine_rule: CoefficientCombineRule::Min });

                            // no collision with self and others from same group (all excavator parts)
                        }
                    }
            });
        });
            entity
        }
        VehicleType::Truck => {
            let truck =
                assets.load(GltfAssetLabel::Scene(0).from_asset("private/truck/scene.gltf"));
                let chassis_dimensions = Vec3::new(1.5f32, 2f32, 0.4f32);
            let mut entity = commands.spawn((
                Name::new("truck"),
                vehicle_type,
                Visibility::default(),
                Transform::from_translation(Vec3::new(2f32, 0f32, 0f32)),
                RigidBody::Dynamic,
            ));
            entity.with_children(|child_builder| {
                // chassis
                child_builder.spawn((
                    Name::new("chassis"),
                    Transform::from_translation(Vec3::new(0f32, 0f32, 0.5f32)),
                    Collider::cuboid(chassis_dimensions.x, chassis_dimensions.y, chassis_dimensions.z),
                    // mass is shifted down to avoid falling on its sides.
                    ColliderMassProperties::MassProperties(MassProperties {
                        local_center_of_mass: Vec3::new(0.0, 0.0, -1.0),
                        ..MassProperties::from_rapier(rapier::prelude::MassProperties::from_cuboid(2f32, chassis_dimensions.into()))
                    }),
                ));

                // target for scoops
                child_builder.spawn((
                    Name::new("scoop target"),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
                    ScoopTarget {
                        possible_offset: Cuboid::new(0.5, 1.0, 0.1),
                    },
                ));

                // loader to store rocks.

                // right wall
                child_builder.spawn((
                    Name::new("right wall"),
                    Visibility::default(),
                    Transform::from_translation(Vec3::new(1.4f32, -0.7f32, 1.5f32)),
                    Collider::cuboid(0.2f32, 2.1f32, 1f32),
                    ColliderMassProperties::Density(0.1f32),
                ));
                // left wall
                child_builder.spawn((
                    Name::new("left wall"),
                    Visibility::default(),
                    Transform::from_translation(Vec3::new(-1.4f32, -0.7f32, 1.5f32)),
                    Collider::cuboid(0.2f32, 2.1f32, 0.8f32),
                    ColliderMassProperties::Density(0.1f32),
                ));
                // front wall
                child_builder.spawn((
                    Name::new("front wall"),
                    Visibility::default(),
                    Transform::from_translation(Vec3::new(0f32, 2f32, 1.5f32)),
                    Collider::cuboid(1.5f32,0.8f32, 1f32),
                    ColliderMassProperties::Density(0.1f32),
                ));
                // front inclined wall
                child_builder.spawn((
                    Name::new("front inclined wall"),
                    Visibility::default(),
                    Transform::from_translation(Vec3::new(0f32, 1.1f32, 1.55f32))
                    .with_rotation(Quat::from_rotation_x(-20f32.to_radians())),
                    Collider::cuboid(1.5f32,0.2f32, 0.8f32),
                    ColliderMassProperties::Density(0.1f32),
                ));
                // bottom wall
                child_builder.spawn((
                    Name::new("bottom wall"),
                    Visibility::default(),
                    Transform::from_translation(Vec3::new(0f32, -1f32, 1.3f32))
                    .with_rotation(Quat::from_rotation_x(-10f32.to_radians())),
                    Collider::cuboid(1.5f32, 2f32, 0.1f32),
                    ColliderMassProperties::Density(0.1f32),
                ));
                // cheat invisible wall behind, to avoid rocks falling.
                child_builder.spawn((
                    Name::new("invisible back wall"),
                    Visibility::default(),
                    Transform::from_translation(Vec3::new(0f32, -2.8f32, 1.5f32)),
                    Collider::cuboid(1.5f32,0.2f32, 1f32),
                    ColliderMassProperties::Density(0.1f32),
                ));
                // TODO: truck loader walls
                // TODO: component doing 1 or more overlap_box query to detect rocks and assess if truck is full.
                //   Counting rocks may be irrelevant because they may be arranged very differently.
            });
            // Model
            entity.with_child((
                Name::new("truck model"),
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
