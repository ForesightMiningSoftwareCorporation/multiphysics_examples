use bevy::prelude::*;
use bevy_rapier3d::plugin::{PhysicsSet, RapierTransformPropagateSet};

pub struct FollowPlugin;

impl Plugin for FollowPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CopyPosition>();
        app.add_systems(
            PostUpdate,
            (follow, global_transform_to_transform)
                .chain()
                //.after(TransformSystem::TransformPropagate)
                //.after(PhysicsSet::SyncBackend)
                .before(PhysicsSet::StepSimulation),
        );
    }
}

/// After the transform propagation,
/// this sets the transform so that its global position is the same(ish) as given entity.
///
/// Note that the 2 global transforms may not be exactly equal due to float imprecisions.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub struct CopyPosition(pub Entity);

/// Follows the position of a target entity, this only sets the [`GlobalTransform`].
/// [`global_transform_to_transform`] will update [`Transform`]s.
pub fn follow(
    // Get component to copy to
    mut q_copy_position: Query<(&CopyPosition, &mut GlobalTransform)>,
    // Get component to copy from
    // FIXME: we could support following a CopyPosition, but frame delays and cycles would have to be accounted for.
    q_global_transform: Query<&GlobalTransform, Without<CopyPosition>>,
) {
    for (CopyPosition(entity_to_follow), mut global_transform) in q_copy_position.iter_mut() {
        let Ok(global_transform_to_follow) = q_global_transform.get(*entity_to_follow) else {
            continue;
        };
        *global_transform = *global_transform_to_follow;
    }
}

/// Inversing the propagation, from global to local transform, because we did set the global transform directly.
pub fn global_transform_to_transform(
    global_transform: Query<&GlobalTransform>,
    parents: Query<&Parent>,
    mut local_transform: Query<(Entity, &mut Transform, &CopyPosition)>,
) {
    for (entity_to_adapt, mut local_transform_to_adapt, CopyPosition(to_copy)) in
        local_transform.iter_mut()
    {
        let global_transform_to_copy = global_transform.get(*to_copy).unwrap();

        let parent_global_transform = parents.get(entity_to_adapt).map_or_else(
            |_| GlobalTransform::default(),
            |parent| *global_transform.get(parent.get()).unwrap(),
        );
        *local_transform_to_adapt =
            global_transform_to_copy.reparented_to(&parent_global_transform);
    }
}
