//! Pistons in excavator look very off when not aligned, so this module helps with that.

use bevy::prelude::*;

use crate::vehicle_spawner::follow::PropagateGlobalToTransform;

pub struct LookAtPlugin;

impl Plugin for LookAtPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<LookAt>();
        app.init_gizmo_group::<LookAtGizmos>();

        app.add_systems(Update, look_at);
    }
}

#[derive(Component, Clone, Reflect, PartialEq)]
#[require(PropagateGlobalToTransform)]
pub struct LookAt {
    pub target: Entity,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct LookAtGizmos {}

/// Changes [`GlobalTransform`] to look at the target. This plays well with [`PropagateGlobalToTransform`].
/// Otherwise, this change would be overwritten by bevy's transform propagation.
pub fn look_at(
    looker: Query<(Entity, &LookAt)>,
    mut targets: Query<&mut GlobalTransform>,
    mut _my_gizmos: Gizmos<LookAtGizmos>,
) {
    for (looker, look_at) in looker.iter() {
        let Ok([mut looker, target]) = targets.get_many_mut([looker, look_at.target]) else {
            continue;
        };
        let mut looker_copy: Transform = looker.compute_transform();
        // The up direction to the right is to avoid a weird rotation when the looker is upside down.
        // This works great for excavators, but may not be the best for other systems.
        looker_copy.look_at(target.translation(), looker.up());
        *looker = GlobalTransform::from(looker_copy);

        /*
        // Debugging lookat.

        _my_gizmos.circle(looker.translation(), 0.4, bevy::color::palettes::css::GREEN);
        _my_gizmos
            .arrow(looker.translation(), target.translation(), ORANGE_RED)
            .with_tip_length(0.5);
        _my_gizmos.circle(looker.translation(), 0.4, bevy::color::palettes::css::BLUE);

        */
    }
}
