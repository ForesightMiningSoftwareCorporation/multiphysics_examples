pub mod assets;
pub mod controls;
pub mod inputs;

use bevy::prelude::*;
use controls::{TruckControls, TruckControlsMapping};
use serde::{Deserialize, Serialize};

use super::RotationControlDef;

#[derive(Debug, Component, Reflect)]
pub struct TruckDefHandle(pub Handle<TruckDef>);

/// Definition of an Truck's accessories, to know which nodes to move and how they can be moved.
#[derive(Debug, Hash, Asset, Serialize, Deserialize, Reflect)]
pub struct TruckDef {
    /// Main dump
    pub main_dump: RotationControlDef,
}

pub struct TruckAccessoryPlugin;

impl Plugin for TruckAccessoryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TruckDef>();
        app.register_type::<TruckDefHandle>();
        app.register_type::<TruckControls>();
        app.register_type::<TruckControlsMapping>();

        app.init_asset::<TruckDef>();
        app.init_asset_loader::<assets::TruckDefLoader>();

        app.add_systems(Update, controls::propagate_controls);
        app.add_systems(Update, assets::set_default_controls);
        app.add_observer(assets::update_truck_control_mapping);
    }
}
