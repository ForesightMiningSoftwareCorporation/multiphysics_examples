use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

use crate::accessory_controls::RotationControlDef;

use super::{ExcavatorDef, ExcavatorDefHandle};

/// Real time knob to control the excavator.
#[derive(
    Debug, PartialEq, Default, Component, Serialize, Deserialize, InspectorOptions, Reflect,
)]
#[reflect(InspectorOptions)]
pub struct ControlKnob {
    // Current value.
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub current_value: f32,
    // desired value, we'll lerp to it based on
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub desired: f32,
}

impl ControlKnob {
    /// Updates current value and desired, returns the new rotation.
    fn smooth_move(&mut self, def: &RotationControlDef, dt: f32) -> Quat {
        let new_value = self
            .current_value
            .lerp(self.desired, (dt * def.sensitivity_lerp_mult).min(1.0));
        self.current_value = new_value;
        let rotation = def.remap_in_range(self.current_value);
        rotation
    }
}

/// Real time knobs to control the excavator.
#[derive(Debug, PartialEq, Default, Component, Serialize, Deserialize, Reflect)]
pub struct ExcavatorControls {
    /// target angle ratio for [`ExcavatorDef::bucket_jaw`]
    pub bucket_jaw: ControlKnob,

    /// target angle ratio for [`ExcavatorDef::bucket_base`]
    pub bucket_base: ControlKnob,

    /// target angle ratio for [`ExcavatorDef::stick`]
    pub stick: ControlKnob,

    /// target angle ratio for [`ExcavatorDef::boom`]
    pub boom: ControlKnob,

    /// target angle in radians for [`ExcavatorDef::swing`]
    pub swing: ControlKnob,
}

/// Real time knobs to control the excavator.
///
/// Entities may be [`Entity::PLACEHOLDER`] if they are not found.
#[derive(Debug, Clone, Component, Serialize, Deserialize, Reflect)]
pub struct ExcavatorControlsMapping {
    /// Entity corresponding to [`ExcavatorDef::bucket_jaw`]
    pub bucket_jaw: Entity,

    /// Entity corresponding to [`ExcavatorDef::bucket_base`]
    pub bucket_base: Entity,

    /// Entity corresponding to [`ExcavatorDef::stick`]
    pub stick: Entity,

    /// Entity corresponding to [`ExcavatorDef::boom`]
    pub boom: Entity,

    /// Entity corresponding to [`ExcavatorDef::swing`]
    pub swing: Entity,
}

/// Changes the rotation of the entities based on the [`ExcavatorControls`].
pub fn propagate_controls(
    time: Res<Time>,
    excavator_defs: Res<Assets<ExcavatorDef>>,
    mut q_controls: Query<(
        &ExcavatorDefHandle,
        &mut ExcavatorControls,
        &ExcavatorControlsMapping,
    )>,
    mut q_transform: Query<&mut Transform>,
) {
    for (handle, mut controls, mapping) in q_controls.iter_mut() {
        let Some(def) = excavator_defs.get(&handle.0) else {
            continue;
        };
        let dt = time.delta_secs();

        let ExcavatorControlsMapping {
            bucket_jaw,
            bucket_base,
            stick,
            boom,
            swing,
        } = *mapping;
        if let Ok(mut transform) = q_transform.get_mut(bucket_jaw) {
            transform.rotation = controls.bucket_jaw.smooth_move(&def.bucket_jaw, dt);
        }
        if let Ok(mut transform) = q_transform.get_mut(bucket_base) {
            transform.rotation = controls.bucket_base.smooth_move(&def.bucket_base, dt);
        }
        if let Ok(mut transform) = q_transform.get_mut(stick) {
            transform.rotation = controls.stick.smooth_move(&def.stick, dt);
        }
        if let Ok(mut transform) = q_transform.get_mut(boom) {
            transform.rotation = controls.boom.smooth_move(&def.boom, dt);
        }
        if let Ok(mut transform) = q_transform.get_mut(swing) {
            transform.rotation = controls.swing.smooth_move(&def.swing, dt);
        }
    }
}
