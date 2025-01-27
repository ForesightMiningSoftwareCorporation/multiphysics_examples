use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, RigidBody};

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
            let mut entity = commands.spawn((
                Visibility::default(),
                Transform::from_translation(Vec3::ZERO),
                Collider::cuboid(1f32, 1f32, 0.4f32),
                RigidBody::Dynamic,
            ));
            // Models are oftentimes not adapted to real usecase, rather than re-exporting a model,
            // we can adapt its scale, position, rotation by spawning it as a child.
            // for example, most models are provided with Y-up, but we're using Z-up.
            entity.with_child((
                SceneRoot(bulldozer.clone()),
                Transform::from_translation(Vec3::new(4.4, 0.0, 0.9))
                     .with_rotation(
                         Quat::from_axis_angle(Vec3::Z, TAU / 4.0) *
                         Quat::from_axis_angle(Vec3::X, TAU / 4.0),
                     )
                    .with_scale(Vec3::new(0.8, 0.8, 0.5)),
            ));
            entity
        }
        VehicleType::Excavator => {
            let excavator =
                assets.load(GltfAssetLabel::Scene(0).from_asset("private/excavator/scene.gltf"));
            let mut entity = commands.spawn((
                Visibility::default(),
                Transform::default(),
                Collider::cuboid(1f32, 1f32, 0.4f32),
                RigidBody::Dynamic,
            ));
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
