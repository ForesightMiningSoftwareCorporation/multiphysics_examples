use bevy::prelude::*;

pub struct TimerTriggerPlugin;

impl Plugin for TimerTriggerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TimerTrigger>();
        app.register_type::<TimerFinished>();
        app.add_event::<TimerFinished>();
        app.add_systems(Update, timer_trigger);
    }
}

#[derive(Component, Reflect)]
pub struct TimerTrigger(pub Timer);

#[derive(Event, Reflect)]
pub struct TimerFinished;

pub fn timer_trigger(
    mut commands: Commands,
    time: Res<Time>,
    mut q_timers: Query<(Entity, &mut TimerTrigger)>,
) {
    for (e, mut t) in q_timers.iter_mut() {
        t.0.tick(time.delta());
        if t.0.just_finished() {
            commands.trigger_targets(TimerFinished, e);
        }
    }
}
