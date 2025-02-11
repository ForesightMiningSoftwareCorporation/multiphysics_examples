use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

use super::{TruckDef, TruckDefHandle};

/// Real time knobs to control the Truck.
#[derive(
    Debug, PartialEq, Default, Component, Serialize, Deserialize, InspectorOptions, Reflect,
)]
#[reflect(InspectorOptions)]
pub struct TruckControls {
    /// target angle ratio for [`TruckDef::main_dump`]
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub main_dump: f32,
}

/// Meshes controlled by [`TruckControlsMapping`]
///
/// Entities may be [`Entity::PLACEHOLDER`] if they are not found.
#[derive(Debug, Clone, Component, Serialize, Deserialize, Reflect)]
pub struct TruckMeshMapping {
    /// Entity following [`TruckDef::main_dump`]
    pub main_dump: Entity,
}

/// Real time knobs to control the Truck.
///
/// Entities may be [`Entity::PLACEHOLDER`] if they are not found.
#[derive(Debug, Clone, Component, Serialize, Deserialize, Reflect)]
pub struct TruckControlsMapping {
    /// Entity corresponding to [`TruckDef::main_dump`]
    pub main_dump: Entity,
}

/// Changes the rotation of the entities based on the [`TruckControls`].
pub fn propagate_controls(
    truck_defs: Res<Assets<TruckDef>>,
    q_controls: Query<(&TruckDefHandle, &TruckControls, &TruckControlsMapping)>,
    mut q_transform: Query<&mut Transform>,
) {
    for (handle, controls, mapping) in q_controls.iter() {
        let Some(def) = truck_defs.get(&handle.0) else {
            continue;
        };

        let TruckControlsMapping { main_dump } = *mapping;
        if let Ok(mut transform) = q_transform.get_mut(main_dump) {
            transform.rotation = def.main_dump.remap_in_range(controls.main_dump);
        }
    }
}
