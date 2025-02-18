use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy_rapier3d::{
    prelude::{
        Collider, ColliderMassProperties, CollisionGroups, Group, MassProperties, RigidBody,
    },
    rapier,
};
use bevy_wgsparkl::components::MpmCouplingEnabled;

use super::VehicleType;

pub const BULLDOZER_PATH: &str = "private/Bulldozer 3D Model/Bulldozer.glb";

pub fn spawn_bulldozer<'a>(
    commands: &'a mut Commands,
    assets: &'a Res<AssetServer>,
    transform: Transform,
) -> EntityCommands<'a> {
    // Bevy caches the assets so we can just load without any additional bookkeeping.
    let bulldozer = assets.load(GltfAssetLabel::Scene(0).from_asset(BULLDOZER_PATH));
    let chassis_dimensions = Vec3::new(1f32, 1.5f32, 0.2f32);
    let chassis_collider = Collider::cuboid(
        chassis_dimensions.x,
        chassis_dimensions.y,
        chassis_dimensions.z,
    );
    let mut entity = commands.spawn((
        Name::new("bulldozer"),
        VehicleType::Bulldozer,
        Visibility::default(),
        transform,
        chassis_collider,
        // mass is moved down, for a better adherence to the ground (also chains are heavier than the cabin)
        ColliderMassProperties::MassProperties(MassProperties {
            local_center_of_mass: Vec3::new(0.0, 0.0, -1.0) * transform.scale,
            ..MassProperties::from_rapier(rapier::prelude::MassProperties::from_cuboid(
                2.8f32,
                (chassis_dimensions * transform.scale).into(),
            ))
        }),
        RigidBody::Dynamic,
    ));
    // bulldozer front, to push rocks.
    entity.with_child((
        Name::new("bulldozer front"),
        Transform::from_translation(Vec3::new(0.0, 1.0, -0.5)),
        Collider::cuboid(1.0f32, 2f32, 1.2f32),
        // no collision with ground
        CollisionGroups::new(Group::all(), !Group::GROUP_2),
        // mass shouldn't impact too much or the vehicle will just fall towards its front.
        ColliderMassProperties::MassProperties(MassProperties {
            local_center_of_mass: Vec3::new(0.0, -1.0, 0.0),
            mass: 0.01,
            principal_inertia: Vec3::ONE * 0.01,
            ..default()
        }),
        MpmCouplingEnabled,
    ));
    // Models are oftentimes not adapted to real usecase, rather than re-exporting a model,
    // we can adapt its scale, position, rotation by spawning it as a child.
    // for example, most models are provided with Y-up, but we're using Z-up.
    entity.with_child((
        Name::new("bulldozer model"),
        SceneRoot(bulldozer.clone()),
        Transform::from_translation(Vec3::new(4.4, 0.0, 0.75))
            .with_rotation(
                Quat::from_axis_angle(Vec3::Z, TAU / 4.0)
                    * Quat::from_axis_angle(Vec3::X, TAU / 4.0),
            )
            .with_scale(Vec3::new(0.8, 0.8, 0.5)),
    ));

    entity
}
