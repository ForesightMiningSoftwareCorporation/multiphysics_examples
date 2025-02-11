use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_rapier3d::{ prelude::{ Collider, ColliderMassProperties, MassProperties, RigidBody}, rapier};
use super::{scoop::ScoopTarget, VehicleType};


pub fn spawn_truck<'a>(
    commands: &'a mut Commands,
    assets: &'a Res<AssetServer>,
) -> EntityCommands<'a> {
    let truck =
    assets.load(GltfAssetLabel::Scene(0).from_asset("private/truck/scene.gltf"));
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
