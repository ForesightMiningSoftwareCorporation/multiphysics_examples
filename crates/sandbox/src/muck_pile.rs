use bevy::{prelude::*, render::primitives::Aabb};
use shared_map::global_assets::GlobalAssets;

/// Spawns a muck pile.
pub struct SpawnMuckPileCommand {
    pub aabb: Aabb,
}

pub struct MuckPile {
    pub rock_count: usize,
}

impl Command for SpawnMuckPileCommand {
    fn apply(self, world: &mut World) {
        let assets = world.resource::<GlobalAssets>();

        let position: Vec3 = self.aabb.center.into();
        world.spawn((
            Name::new("Muck Pile"),
            Mesh3d(assets.muck_pile_mesh.clone_weak()),
            MeshMaterial3d(assets.muck_pile_material.clone_weak()),
            Transform::from_translation(position + Vec3::Z * 0.01f32)
                .with_scale(self.aabb.half_extents.into()),
            PickingBehavior::IGNORE,
        ));
    }
}
