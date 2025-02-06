//! Rapier async collider initialization compute
//! the rapier colliders mass properties according to their volume,
//! that makes it difficult to tweak engines forces,
//! so this module can override mass to make it more predictable.

use bevy::prelude::*;
use bevy_rapier3d::prelude::ColliderMassProperties;

pub struct OverrideMassOnSpawnPlugin;

impl Plugin for OverrideMassOnSpawnPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<OverrideMassOnSpawn>();
        app.add_systems(
            PostUpdate,
            override_mass.after(bevy_rapier3d::plugin::systems::init_async_scene_colliders),
        );
    }
}

#[derive(Component, Debug, Default, Reflect)]
pub struct OverrideMassOnSpawn {
    pub names_to_override: Vec<String>,
}

pub fn override_mass(
    mut commands: Commands,
    scene_spawner: Res<SceneSpawner>,
    override_colliders: Query<(Entity, &bevy::scene::SceneInstance, &OverrideMassOnSpawn)>,
    children: Query<&Children>,
    q_name: Query<&Name>,
) {
    for (scene_entity, scene_instance, override_mass) in override_colliders.iter() {
        if scene_spawner.instance_is_ready(**scene_instance) {
            for child_entity in children.iter_descendants(scene_entity) {
                if let Ok(name) = q_name.get(child_entity) {
                    if override_mass.names_to_override.contains(&name.to_string()) {
                        commands
                            .entity(child_entity)
                            .insert(ColliderMassProperties::Density(0f32));
                    }
                }
            }
            commands
                .entity(scene_entity)
                .remove::<OverrideMassOnSpawn>();
        }
    }
}
