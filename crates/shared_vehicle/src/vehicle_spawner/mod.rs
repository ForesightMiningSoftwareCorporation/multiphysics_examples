pub mod bulldozer;
pub mod excavator;
pub mod follow;
pub mod react_on_scene_instance_ready;
pub mod scoop;
pub mod truck;

use crate::look_at::LookAtPlugin;
use bevy::prelude::*;
use follow::FollowPlugin;
use react_on_scene_instance_ready::ReactOnSceneInstanceReadyPlugin;

pub struct VehicleSpawnerPlugin;

impl Plugin for VehicleSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<VehicleType>();
        app.add_plugins(ReactOnSceneInstanceReadyPlugin);
        app.add_plugins(FollowPlugin);
        app.add_plugins(LookAtPlugin);
    }
}

#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleType {
    Bulldozer,
    Excavator,
    Truck,
}

impl std::fmt::Display for VehicleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn spawn<'a>(
    vehicle_type: VehicleType,
    commands: &'a mut Commands,
    assets: &'a Res<AssetServer>,
    transform: Transform,
) -> EntityCommands<'a> {
    let mut bulldozer_transform = transform;
    bulldozer_transform.scale *= 1.3;
    match vehicle_type {
        VehicleType::Bulldozer => bulldozer::spawn_bulldozer(commands, assets, bulldozer_transform),
        VehicleType::Excavator => excavator::spawn_excavator(commands, assets, transform),
        VehicleType::Truck => truck::spawn_truck(commands, assets, transform),
    }
}
