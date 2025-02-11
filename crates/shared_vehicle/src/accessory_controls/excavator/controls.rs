use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

use super::{ExcavatorDef, ExcavatorDefHandle};

/// Real time knobs to control the excavator.
#[derive(
    Debug, PartialEq, Default, Component, Serialize, Deserialize, InspectorOptions, Reflect,
)]
#[reflect(InspectorOptions)]
pub struct ExcavatorControls {
    /// target angle ratio for [`ExcavatorDef::bucket_jaw`]
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bucket_jaw: f32,

    /// target angle ratio for [`ExcavatorDef::bucket_base`]
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bucket_base: f32,

    /// target angle ratio for [`ExcavatorDef::stick`]
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub stick: f32,

    /// target angle ratio for [`ExcavatorDef::boom`]
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub boom: f32,

    /// target angle in radians for [`ExcavatorDef::swing`]
    pub swing: f32,
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
    excavator_defs: Res<Assets<ExcavatorDef>>,
    q_controls: Query<(
        &ExcavatorDefHandle,
        &ExcavatorControls,
        &ExcavatorControlsMapping,
    )>,
    mut q_transform: Query<&mut Transform>,
) {
    for (handle, controls, mapping) in q_controls.iter() {
        let Some(def) = excavator_defs.get(&handle.0) else {
            continue;
        };

        let ExcavatorControlsMapping {
            bucket_jaw,
            bucket_base,
            stick,
            boom,
            swing,
        } = *mapping;
        if let Ok(mut transform) = q_transform.get_mut(bucket_jaw) {
            transform.rotation = def.bucket_jaw.remap_in_range(controls.bucket_jaw);
        }
        if let Ok(mut transform) = q_transform.get_mut(bucket_base) {
            transform.rotation = def.bucket_base.remap_in_range(controls.bucket_base);
        }
        if let Ok(mut transform) = q_transform.get_mut(stick) {
            transform.rotation = def.stick.remap_in_range(controls.stick);
        }
        if let Ok(mut transform) = q_transform.get_mut(boom) {
            transform.rotation = def.boom.remap_in_range(controls.boom);
        }
        if let Ok(mut transform) = q_transform.get_mut(swing) {
            transform.rotation = def.swing.remap_in_range(controls.swing);
        }
    }
}
