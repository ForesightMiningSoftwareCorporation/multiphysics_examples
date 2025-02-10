//! When a gltf is loading, it is not immediately ready. This plugin allows you to react to when a scene is ready.
//!
//! I think The bevy built-in [`bevy::scene::SceneInstanceReady`] triggers too early, so this is a workaround.

use bevy::prelude::*;

pub struct ReactOnSceneInstanceReadyPlugin;

impl Plugin for ReactOnSceneInstanceReadyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ReactOnSceneInstanceReady>();
        app.register_type::<OnSceneReady>();
        app.add_event::<OnSceneReady>();
        app.add_systems(
            PostUpdate,
            react_on_scene_instance_ready
                .after(bevy_rapier3d::plugin::systems::init_async_scene_colliders),
        );
    }
}

/// Component to opt into [`OnSceneReady`] events. This is removed when the event is sent.
#[derive(Component, Clone, Reflect)]
pub struct ReactOnSceneInstanceReady;

/// Trigger sent when a scene is ready on the scene root entity.
#[derive(Event, Clone, Debug, Reflect)]
pub struct OnSceneReady;

pub fn react_on_scene_instance_ready(
    mut commands: Commands,
    scene_spawner: Res<SceneSpawner>,
    scenes: Query<(Entity, &bevy::scene::SceneInstance), With<ReactOnSceneInstanceReady>>,
) {
    for (scene_entity, scene_instance) in scenes.iter() {
        if !scene_spawner.instance_is_ready(**scene_instance) {
            continue;
        }
        commands.trigger_targets(OnSceneReady, scene_entity);
        commands
            .entity(scene_entity)
            .remove::<ReactOnSceneInstanceReady>();
    }
}
