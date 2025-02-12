pub mod assets;
pub mod controls;
pub mod inputs;

use assets::ExcavatorDefLoader;
use bevy::prelude::*;
use controls::{ExcavatorControls, ExcavatorControlsMapping};
use serde::{Deserialize, Serialize};

use super::{LookAtDef, RotationControlDef};

#[derive(Debug, Component, Reflect)]
pub struct ExcavatorDefHandle(pub Handle<ExcavatorDef>);

/// Definition of an excavator's accessories, to know which nodes to move and how they can be moved.
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

pub struct ExcavatorAccessoryPlugin;

impl Plugin for ExcavatorAccessoryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ExcavatorDef>();
        app.register_type::<ExcavatorDefHandle>();
        app.register_type::<ExcavatorControls>();
        app.register_type::<ExcavatorControlsMapping>();

        app.init_asset::<ExcavatorDef>();
        app.init_asset_loader::<ExcavatorDefLoader>();

        app.add_systems(Update, controls::propagate_controls);
        app.add_systems(Update, assets::set_default_controls);

        app.add_observer(assets::update_excavator_control_mapping);
    }
}
