//! Plugin for a fake scooping logic (excavator picking up rocks and storing them into the truck).
//!
//! This plugin is agnostic to other modules for reuse, but references rocks in comments for context.
//!
//! This module is not actually used currently, but may be used if simulating realistically is too challenging.

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::{
    prelude::{CollisionEvent, CollisionGroups, ExternalImpulse, Group, RigidBody},
    rapier::prelude::CollisionEventFlags,
};

pub struct ScoopPlugin;

impl Plugin for ScoopPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SensorStartScoop>();
        app.register_type::<FlyScoopTo>();
        app.register_type::<ScoopTarget>();
        app.register_type::<ScoopCommand>();

        app.add_systems(
            Update,
            (scoop_detect_trigger, scooped_move_to_target).chain(),
        );
    }
}

/// Component applied to the sensor of the excavator scoop
///
/// When a [`Scoopable`] enters the sensor, it will be moved to the truck.
#[derive(Component, Debug, Reflect)]
pub struct SensorStartScoop;

#[derive(Component, Debug, Reflect)]
pub struct Scoopable;

/// Because model does not provide an animation or rig,
/// it's simpler to fake the move to the truck.
#[derive(Component, Debug, Reflect)]
pub struct FlyScoopTo {
    /// The target to move to. It is expected to have component [`ScoopTarget`].
    pub target: Entity,
    /// We randomize the exact target to avoid all rocks being sent to the same place.
    ///
    /// This is the offset from the [`Self::target`] (use [`GlobalTransform::transform_point`] ).
    pub offset_from_target: Vec3,
    /// position where the scoop started.
    ///
    /// Useful to compute progress of the current scooping progress.
    pub start_ground_position: Vec3,
    /// [`Time<Virtual>`] elapsed time when the scoop started.
    pub start_scoop_time: f32,
}

/// Component to find where to send scooped rocks.
#[derive(Component, Debug, Reflect, Clone)]
pub struct ScoopTarget {
    // To avoid sending all rocks to the same place, we can randomize their placement.
    pub possible_offset: Cuboid,
}

/// Command to move a rock (`to_scoop`) to the scoop target (`target`).
#[derive(Debug, Reflect)]
pub struct ScoopCommand {
    /// The rock to move. It is expected to have a [`Scoopable`] component.
    ///
    /// [`ScoopCommand`] will add [`FlyScoopTo`] component to that entity.
    pub to_scoop: Entity,
    /// The target to move the rock to. It is expected to have [`ScoopTarget`] component.
    pub target: Entity,
}

impl Command for ScoopCommand {
    fn apply(self, world: &mut World) {
        let start_position = world
            .query::<&Transform>()
            .get(world, self.to_scoop)
            .unwrap()
            .translation;
        let time_started = world.get_resource::<Time<Virtual>>().unwrap().elapsed();
        let scoop_target_copy = world
            .query::<&ScoopTarget>()
            .get(world, self.target)
            .unwrap()
            .clone();
        let mut random = rand::thread_rng();
        let offset = scoop_target_copy
            .possible_offset
            .sample_interior(&mut random);
        world
            .entity_mut(self.to_scoop)
            .insert(FlyScoopTo {
                target: self.target,
                start_ground_position: start_position,
                start_scoop_time: time_started.as_secs_f32(),
                offset_from_target: offset,
            })
            .insert(CollisionGroups::new(Group::NONE, Group::NONE))
            .insert(RigidBody::Fixed);
    }
}

pub fn scoop_detect_trigger(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    q_rock: Query<&Transform, (With<Scoopable>, Without<FlyScoopTo>)>,
    q_excavator_scoop: Query<&Transform, With<SensorStartScoop>>,
    q_scoop_target: Query<(Entity, &Transform), With<ScoopTarget>>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, collision_event_flags) => {
                if !collision_event_flags.contains(CollisionEventFlags::SENSOR) {
                    continue;
                }
                // is there a rock?
                let (maybe_excavator, (rock, rock_position)) =
                    match (q_rock.get(*entity1), q_rock.get(*entity2)) {
                        (Ok(transform), Err(_)) => (entity2, (entity1, transform.translation)),
                        (Err(_), Ok(transform)) => (entity1, (entity2, transform.translation)),
                        _ => continue,
                    };
                // is there an excavator scoop?
                // _excavator may be useful to find the excavator attempting to scoop.
                let (_excavator, rock) = if q_excavator_scoop.get(*maybe_excavator).is_ok() {
                    (maybe_excavator, rock)
                } else {
                    continue;
                };

                // Find the closest scoop target
                let mut closest = None;
                for (scoop_target_entity, scoop_target_transform) in q_scoop_target.iter() {
                    let distance = scoop_target_transform.translation.distance(rock_position);
                    match closest {
                        None => {
                            closest = Some((scoop_target_entity, distance));
                        }
                        Some((_, previous_closest_distance)) => {
                            if distance < previous_closest_distance {
                                closest = Some((scoop_target_entity, distance));
                            }
                        }
                    }
                }
                if let Some(closest) = closest {
                    commands.queue(ScoopCommand {
                        to_scoop: *rock,
                        target: closest.0,
                    });
                }
            }
            CollisionEvent::Stopped(_, _, _) => continue,
        }
    }
}

pub fn scooped_move_to_target(
    mut commands: Commands,
    time: Res<Time>,
    mut q_move_scooped: Query<(Entity, &mut Transform, &FlyScoopTo), Without<ScoopTarget>>,
    q_scoop_target: Query<&GlobalTransform, With<ScoopTarget>>,
) {
    for (entity, mut scooped_transform, fly_scooped_to) in q_move_scooped.iter_mut() {
        let target_transform = q_scoop_target.get(fly_scooped_to.target).unwrap();
        let target = target_transform.transform_point(fly_scooped_to.offset_from_target);
        let progress_ratio =
            (time.elapsed().as_secs_f32() - fly_scooped_to.start_scoop_time) / 0.7f32;
        // move towards target at a specific speed.
        let mut new_position = fly_scooped_to
            .start_ground_position
            .lerp(target, progress_ratio);
        let to_target = target - fly_scooped_to.start_ground_position;
        new_position.z = fly_scooped_to.start_ground_position.z
            // parabolic movement
            + 4f32 * (progress_ratio * PI).sin()
            // apply Y difference progressively.
            + to_target.z * progress_ratio;
        if progress_ratio >= 0.98f32 {
            // reached target
            let force_continuity = 
            //(new_position - scooped_transform.translation
                Vec3::Z * -0.2f32;
            scooped_transform.translation = target;

            commands
                .entity(entity)
                .insert(RigidBody::Dynamic)
                .insert(ExternalImpulse {
                    impulse: force_continuity,
                    torque_impulse: Vec3::ONE * 0.001f32,
                })
                .insert(CollisionGroups::new(Group::all(), Group::all()))
                .remove::<FlyScoopTo>();
            continue;
        }
        scooped_transform.translation = new_position;
    }
}
