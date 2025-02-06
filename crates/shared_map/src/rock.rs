use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::global_assets::GlobalAssets;

#[derive(Debug, Default, Reflect, Component)]
pub struct Rock;

/// Spawns a rock at the given isometry.
pub struct SpawnRockCommand {
    pub isometry: Isometry3d,
}

impl Command for SpawnRockCommand {
    fn apply(self, world: &mut World) {
        let assets = world.resource::<GlobalAssets>();
        world.spawn((
            Name::new("Rock"),
            Mesh3d(assets.rock_mesh.clone_weak()),
            MeshMaterial3d(assets.rock_material.clone_weak()),
            Collider::cuboid(0.1, 0.1, 0.1),
            RigidBody::Dynamic,
            Transform::from_isometry(self.isometry),
            PickingBehavior::IGNORE,
            Rock,
        ));
    }
}
