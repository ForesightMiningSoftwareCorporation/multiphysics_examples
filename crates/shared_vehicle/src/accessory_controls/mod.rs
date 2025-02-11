pub mod excavator;
pub mod truck;

use bevy::prelude::*;
use excavator::ExcavatorAccessoryPlugin;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use truck::TruckAccessoryPlugin;

pub struct AccessoryControlsPlugin;

impl Plugin for AccessoryControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ExcavatorAccessoryPlugin, TruckAccessoryPlugin));
    }
}

#[derive(Debug, Serialize, Deserialize, Reflect)]
pub struct RotationControlDef {
    /// The name of the node in the model, should be unique.
    pub node_name: String,
    pub axis: Vec3,
    pub min_max_angle: Option<Vec2>,
    pub default_angle: f32,
    pub sensitivity: f32,
}

impl RotationControlDef {
    pub fn clamp_angle(&self, angle: f32) -> f32 {
        let Some(min_max_angle) = self.min_max_angle else {
            return angle;
        };
        angle.clamp(min_max_angle.x, min_max_angle.y)
    }

    pub fn get_default_ratio(&self) -> f32 {
        let Some(min_max_angle) = self.min_max_angle else {
            return self.default_angle;
        };
        self.default_angle.remap(
            self.clamp_angle(min_max_angle.x),
            self.clamp_angle(min_max_angle.y),
            0.0,
            1.0,
        )
    }

    pub fn remap_in_range(&self, rotation: f32) -> Quat {
        let Some(min_max) = self.min_max_angle else {
            return Quat::from_axis_angle(self.axis, rotation);
        };
        Quat::from_axis_angle(self.axis, rotation.remap(0.0, 1.0, min_max.x, min_max.y))
    }
}
impl Hash for RotationControlDef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Destructuring to avoid forgetting to add new fields to the hash if the structure changes.
        let Self {
            node_name,
            axis,
            min_max_angle,
            default_angle,
            sensitivity,
        } = self;
        node_name.hash(state);
        if let Some(min_max_angle) = min_max_angle {
            min_max_angle.x.to_bits().hash(state);
            min_max_angle.y.to_bits().hash(state);
        };
        axis.x.to_bits().hash(state);
        axis.y.to_bits().hash(state);
        axis.z.to_bits().hash(state);
        default_angle.to_bits().hash(state);
        sensitivity.to_bits().hash(state);
    }
}

#[derive(Debug, Default, Hash, Asset, Serialize, Deserialize, Reflect)]
pub struct LookAtDef {
    pub looker: String,
    pub target: String,
    pub both_ways: bool,
}
