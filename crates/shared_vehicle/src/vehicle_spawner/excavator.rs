use std::f32::consts::TAU;

use super::react_on_scene_instance_ready::{OnSceneReady, ReactOnSceneInstanceReady};
use super::{follow::CopyPosition, VehicleType};
use crate::{
    accessory_controls::{
        excavator::{ExcavatorDef, ExcavatorDefHandle},
        LookAtDef,
    },
    look_at::LookAt,
};
use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_rapier3d::parry::bounding_volume::Aabb;
use bevy_rapier3d::parry::shape::SharedShape;
use bevy_rapier3d::{
    prelude::{
        Collider, ColliderMassProperties, CollisionGroups, ComputedColliderShape, Group,
        MassProperties, RigidBody,
    },
    rapier,
};
use bevy_wgsparkl::components::MpmCouplingEnabled;

pub fn spawn_excavator<'a>(
    commands: &'a mut Commands,
    assets: &'a Res<AssetServer>,
    transform: Transform,
) -> EntityCommands<'a> {
    let chassis_dimensions = Vec3::new(1f32, 2f32, 0.4f32);
    let chassis_collider = Collider::cuboid(
        chassis_dimensions.x,
        chassis_dimensions.y,
        chassis_dimensions.z,
    );
    let excavator =
        assets.load(GltfAssetLabel::Scene(0).from_asset("private/excavator/excavator.gltf"));
    let mut entity = commands.spawn((
        Name::new("excavator"),
        VehicleType::Excavator,
        Visibility::default(),
        transform,
        chassis_collider,
        // mass is shifted down to avoid falling on its sides.
        ColliderMassProperties::MassProperties(MassProperties {
            local_center_of_mass: Vec3::new(0.0, 0.0, -1.0) * transform.scale,
            ..MassProperties::from_rapier(rapier::prelude::MassProperties::from_cuboid(
                0.8f32,
                (chassis_dimensions * transform.scale).into(),
            ))
        }),
        CollisionGroups::new(Group::GROUP_3, !Group::GROUP_3),
        RigidBody::Dynamic,
    ));
    // Sensor to detect rocks, and move them to the truck.
    /*entity.with_child((
        Name::new("scoop sensor"),
        Transform::from_translation(Vec3::new(0.0, 2.5, -0.5)),
        Sensor,
        Collider::cuboid(1f32, 0.4f32, 0.8f32),
        // no collision with ground
        CollisionGroups::new(Group::all(), Group::GROUP_1),
        SensorStartScoop
    ));*/

    let bucket_base_name = "Mesh.004";
    let bucket_jaws_name = "Mesh.003";
    let meshes_to_convert_to_collider: HashMap<String, Option<ComputedColliderShape>> = [
        /*
        // Boom
        (
            "Mesh.018".to_string(),
            Some(ComputedColliderShape::default()),
        ),
        // Stick
        (
            "Mesh.007".to_string(),
            Some(ComputedColliderShape::default()),
        ),*/
        // Bucket base
        (
            bucket_base_name.to_string(),
            Some(ComputedColliderShape::default()),
        ),
        // Bucket jaws
        (
            bucket_jaws_name.to_string(),
            Some(ComputedColliderShape::default()),
        ),
    ]
    .into();
    // Model
    entity.with_children(| child_builder| {
        child_builder.spawn((
            Name::new("excavator model"),
            SceneRoot(excavator.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.0, -0.75f32))
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
            bevy_rapier3d::prelude::AsyncSceneCollider { shape: None, named_shapes: meshes_to_convert_to_collider.clone() },
        ))
        .observe(move |
            // apply kinematic body parts to relevant pieces.
            trigger: Trigger<OnSceneReady>,
            mut commands: Commands,
            q_children: Query<&Children>,
            q_parents: Query<&Parent>,
            q_names: Query<&Name>,
            //
            // add flavor lookat parts
            assets: Res<Assets<ExcavatorDef>>,
            q_excavator_def: Query<&ExcavatorDefHandle>,
            //
            | {
                let look_ats: Option<&Vec<LookAtDef>> = (||{
                    for ancestor in q_parents.iter_ancestors(trigger.entity()) {
                        let Ok(handler) = q_excavator_def.get(ancestor) else {
                            continue;
                        };
                        let excavator_def = assets.get(&handler.0)?;
                        return Some(&excavator_def.look_ats);
                    }
                    None
                })();
                let mut name_to_entity = HashMap::<String, Entity>::default();
                for lookat in look_ats.iter().flat_map(|lookats| lookats.iter()) {
                    name_to_entity.insert(lookat.looker.clone(), Entity::PLACEHOLDER);
                    name_to_entity.insert(lookat.target.clone(), Entity::PLACEHOLDER);
                }
                for entity in q_children.iter_descendants(trigger.entity()) {
                    let Ok(name) = q_names.get(entity) else {
                        continue;
                    };
                    commands.entity(entity).insert(CollisionGroups::new(Group::GROUP_3, !Group::GROUP_3));
                    if meshes_to_convert_to_collider.contains_key(&name.to_string()) {
                        // We found a joint that we want to control,
                        // we're transforming it to a parentless kinematic rigidbody,
                        // to avoid mass impacting our vehicle collider.
                        // We still need that collider to have some mass to push rocks.
                        commands.entity(entity).insert(CopyPosition(q_parents.get(entity).unwrap().get()));
                        commands.entity(entity).remove_parent_in_place();
                        commands.entity(entity).insert(RigidBody::KinematicPositionBased);
                        commands.entity(entity).insert(MpmCouplingEnabled);

                        // This is the hard-coded AABB of the shovel bucket.
                        let aabb = Aabb { mins: [-35.630722, 297.40216, 449.29544].into(), maxs: [335.56757, 608.67285, 705.7152].into() };

                        let (vtx, mut idx) = aabb.to_trimesh();

                        // This super ugly piece removes faces of the aabb from the jaw or base
                        // to approximate their original shapes.
                        if name.to_string() == bucket_jaws_name {
                            // Remove the last two faces.
                            for _ in 0..4 {
                                idx.remove(8);
                            }
                            // Remove the first three faces.
                            for _ in 0..6 {
                                idx.remove(0);
                            }
                        } else {
                            assert_eq!(name.to_string(), bucket_base_name, "Unsupported collision mesh supported for the excavator.");

                            // Remove the fourth and the last faces.
                            idx.remove(10);
                            idx.remove(10);
                            idx.remove(6);
                            idx.remove(6);
                        }



                        let new_shape = SharedShape::trimesh(vtx.to_vec(), idx).unwrap();
                        commands.entity(entity).insert(Collider::from(new_shape));

                        // no collision with self and others from same group (all excavator parts)
                    }
                    // fill our map of name to entity
                    if name_to_entity.contains_key(&name.to_string()) {
                        name_to_entity.insert(name.to_string(), entity);
                    }
                }
                // actually add the component for each lookat
                for lookat in look_ats.iter().flat_map(|lookats| lookats.iter()) {
                    let Some(mut entity) = commands.get_entity(name_to_entity[&lookat.looker]) else {
                        warn!("Could not find entity named {} in lookats", lookat.looker);
                        continue;
                    };

                    entity.try_insert(LookAt {
                        target: name_to_entity[&lookat.target],
                    });
                    if lookat.both_ways {
                        let target = entity.id();
                        let Some(mut looker) = commands.get_entity(name_to_entity[&lookat.target]) else {
                            warn!("Could not find target named {} in lookats", lookat.target);
                            continue;
                        };
                        looker.try_insert(LookAt {
                            target,
                        });
                    }
                }
            }
        );
    });
    entity
}
