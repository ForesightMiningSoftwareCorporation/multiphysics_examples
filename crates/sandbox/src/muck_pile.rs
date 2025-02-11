use bevy::{
    prelude::*,
    render::{primitives::Aabb, view::NoFrustumCulling},
};
use shared_map::global_assets::GlobalAssets;

use crate::stats_rocks::CountRocksInZone;

/// Spawns a muck pile.
pub struct SpawnMuckPileCommand {
    pub position: Isometry3d,
    pub local_aabb: Aabb,
    pub name: String,
}

impl Default for SpawnMuckPileCommand {
    fn default() -> Self {
        Self {
            position: Isometry3d::default(),
            local_aabb: Aabb {
                center: Vec3::ZERO.into(),
                half_extents: Vec3::ONE.into(),
            },
            name: "Muck Pile".to_string(),
        }
    }
}

impl SpawnMuckPileCommand {
    pub fn to_bundle_minimal(self) -> impl Bundle {
        (
            Name::new(self.name),
            Transform::from_translation(Vec3::from(self.position.translation))
                .with_scale(self.local_aabb.half_extents.into())
                .with_rotation(self.position.rotation),
            self.local_aabb,
            CountRocksInZone(0),
        )
    }

    pub fn to_bundle(self, assets: &GlobalAssets) -> impl Bundle {
        (
            self.to_bundle_minimal(),
            // because of the aabb the frustum culling can be annoying.
            NoFrustumCulling,
            Mesh3d(assets.muck_pile_mesh.clone_weak()),
            MeshMaterial3d(assets.muck_pile_material.clone_weak()),
            PickingBehavior::IGNORE,
        )
    }
}

impl EntityCommand for SpawnMuckPileCommand {
    fn apply(self, entity: Entity, world: &mut World) {
        let assets = world.resource::<GlobalAssets>().clone();
        world.entity_mut(entity).insert(self.to_bundle(&assets));
    }
}
impl Command for SpawnMuckPileCommand {
    fn apply(self, world: &mut World) {
        let assets = world.resource::<GlobalAssets>().clone();
        world.spawn(self.to_bundle(&assets));
    }
}
