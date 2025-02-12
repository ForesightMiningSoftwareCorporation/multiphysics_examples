use std::f32::consts::TAU;

use crate::accessory_controls::truck::{
    assets::update_truck_control_mapping, controls::TruckMeshMapping,
};
use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::{
    prelude::{
        Collider, ColliderMassProperties, CollisionGroups, ComputedColliderShape, Friction, Group,
        MassProperties, RigidBody,
    },
    rapier,
};

use super::{
    follow::CopyPosition,
    react_on_scene_instance_ready::{OnSceneReady, ReactOnSceneInstanceReady},
    scoop::ScoopTarget,
    VehicleType,
};

pub fn spawn_truck<'a>(
    commands: &'a mut Commands,
    assets: &'a Res<AssetServer>,
) -> EntityCommands<'a> {
    let truck = assets.load(GltfAssetLabel::Scene(0).from_asset("private/truck/truck.gltf"));
    let chassis_dimensions = Vec3::new(1.5f32, 2f32, 0.4f32);
    let mut entity = commands.spawn((
        Name::new("truck"),
        VehicleType::Truck,
        Visibility::default(),
        Transform::from_translation(Vec3::new(2f32, 0f32, 0f32)),
        RigidBody::Dynamic,
    ));
    entity.with_children(|child_builder| {
        // chassis
        child_builder.spawn((
            Name::new("chassis"),
            Transform::from_translation(Vec3::new(0f32, 0f32, 0.5f32)),
            Collider::cuboid(
                chassis_dimensions.x,
                chassis_dimensions.y,
                chassis_dimensions.z,
            ),
            // mass is shifted down to avoid falling on its sides.
            ColliderMassProperties::MassProperties(MassProperties {
                local_center_of_mass: Vec3::new(0.0, 0.0, -1.0),
                ..MassProperties::from_rapier(rapier::prelude::MassProperties::from_cuboid(
                    2f32,
                    chassis_dimensions.into(),
                ))
            }),
            CollisionGroups::new(Group::GROUP_4, !Group::GROUP_4),
        ));

        // target for scoops
        child_builder.spawn((
            Name::new("scoop target"),
            Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
            ScoopTarget {
                possible_offset: Cuboid::new(0.5, 1.0, 0.1),
            },
        ));
    });
    let main_dump_name = "bucket_low_Bucket_0";
    let meshes_to_convert_to_collider: HashMap<String, Option<ComputedColliderShape>> = [
        // main dump
        (
            main_dump_name.to_string(),
            Some(ComputedColliderShape::default()),
        ),
    ]
    .into();
    // Model
    entity
        .with_child((
            Name::new("truck model"),
            SceneRoot(truck.clone()),
            Transform::from_translation(Vec3::new(0f32, 0f32, -0.4f32))
                .with_scale(Vec3::new(0.005, 0.005, 0.005))
                .with_rotation(
                    // Look back
                    Quat::from_axis_angle(Vec3::Y, TAU / 2.0) 
                // look up
                * Quat::from_axis_angle(Vec3::X, -TAU / 4.0),
                ),
            ReactOnSceneInstanceReady,
            bevy_rapier3d::prelude::AsyncSceneCollider {
                shape: None,
                named_shapes: meshes_to_convert_to_collider.clone(),
            },
        ))
        .observe(
            move |// apply kinematic body parts to relevant pieces.
                  trigger: Trigger<OnSceneReady>,
                  mut commands: Commands,
                  q_children: Query<&Children>,
                  q_parents: Query<&Parent>,
                  // truck has name duplicates so we filter to only meshes.
                  q_names: Query<&Name, With<Mesh3d>>| {
                let mut mesh_mapping = TruckMeshMapping {
                    main_dump: Entity::PLACEHOLDER,
                };
                for entity in q_children.iter_descendants(trigger.entity()) {
                    let Ok(name) = q_names.get(entity) else {
                        continue;
                    };
                    commands
                        .entity(entity)
                        .insert(CollisionGroups::new(Group::GROUP_4, !Group::GROUP_4));
                    if meshes_to_convert_to_collider.contains_key(&name.to_string()) {
                        // We found a joint that we want to control,
                        // we're transforming it to a parentless kinematic rigidbody,
                        // to avoid mass impacting our vehicle collider.
                        // We still need that collider to have some mass to push rocks.
                        commands
                            .entity(entity)
                            .insert(CopyPosition(q_parents.get(entity).unwrap().get()));
                        commands.entity(entity).remove_parent_in_place();
                        commands
                            .entity(entity)
                            .insert(RigidBody::KinematicPositionBased);
                        if name.as_str() == main_dump_name {
                            mesh_mapping.main_dump = entity;
                            commands.entity(entity).insert(Friction::default());
                        }

                        // no collision with self and others from same group (all truck parts)
                    }
                }
                commands.entity(trigger.entity()).insert(mesh_mapping);
            },
        );
    entity.observe(update_truck_control_mapping);
    entity
}
