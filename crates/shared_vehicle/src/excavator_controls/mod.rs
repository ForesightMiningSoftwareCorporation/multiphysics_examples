pub mod assets;
pub mod controls;
pub mod inputs;

use bevy::prelude::*;
use controls::{ExcavatorControls, ExcavatorControlsMapping};
use serde::{Deserialize, Serialize};

/// Plugin to register bevy types and support hot reloading.
pub struct ExcavatorControlsPlugin;

impl Plugin for ExcavatorControlsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ExcavatorDef>();
        app.register_type::<ExcavatorDefHandle>();
        app.register_type::<ExcavatorControls>();
        app.register_type::<ExcavatorControlsMapping>();

        app.init_asset::<ExcavatorDef>();
        app.init_asset_loader::<assets::ExcavatorDefLoader>();

        //app.add_systems(Update, assets::update_excavator_control_mapping);
        app.add_systems(Update, controls::propagate_controls);
        app.add_systems(Update, assets::set_default_controls);
    }
}

#[derive(Debug, Component, Reflect)]
pub struct ExcavatorDefHandle(pub Handle<ExcavatorDef>);

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
}

/// Definition of an excavator, to know which nodes to move and how they can be moved.
#[derive(Debug, Hash, Asset, Serialize, Deserialize, Reflect)]
pub struct ExcavatorDef {
    /// The jaw of the bucket
    ///
    /// HMS_bucket_jaws_JNT, axis X
    pub bucket_jaw: RotationControlDef,

    /// The base of the bucket
    ///
    /// HMS_bucket_bucket_JNT, axis X
    pub bucket_base: RotationControlDef,

    /// The second part of the excavator arm.
    ///
    /// HMS_stick_JNT, axis X
    pub stick: RotationControlDef,

    /// The base of the excavator arm.
    ///
    /// HMS_boom_JNT, axis X
    pub boom: RotationControlDef,
    /// top of the excavator, without the tracked wheels
    ///
    /// HMS_swing_drive ; axis Y (model is y up?)
    pub swing: RotationControlDef,

    pub look_ats: Vec<LookAtDef>,
}

#[derive(Debug, Default, Hash, Asset, Serialize, Deserialize, Reflect)]
pub struct LookAtDef {
    pub looker: String,
    pub target: String,
    pub both_ways: bool,
}
