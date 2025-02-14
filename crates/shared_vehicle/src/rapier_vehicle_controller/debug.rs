use crate::vehicle_spawner::VehicleType;

use super::{VehicleController, VehicleControllerParameters};
use bevy::{color::palettes, prelude::*};

pub struct VehicleControllerDebugPlugin;

impl Plugin for VehicleControllerDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<VehicleControllerGizmos>();
        app.world_mut()
            .get_resource_mut::<GizmoConfigStore>()
            .unwrap()
            .config_mut::<VehicleControllerGizmos>()
            .0
            .enabled = false;
        app.add_systems(Update, (show_vehicle_origin_gizmos, show_wheels_gizmos));
    }
}

// Gizmo group for vehicle controller debugging.
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct VehicleControllerGizmos {}

pub fn show_wheels_gizmos(
    mut gizmos: Gizmos<VehicleControllerGizmos>,
    q_vehicles: Query<(&VehicleControllerParameters, &VehicleController)>,
) {
    for (parameters, controller) in q_vehicles.iter() {
        for w in controller.controller.wheels() {
            let global_center_wheel = w.center().into();
            gizmos.sphere(
                global_center_wheel,
                parameters.wheel_radius,
                palettes::basic::RED,
            );
            let raycast_info = w.raycast_info();
            if raycast_info.is_in_contact {
                gizmos.line(
                    global_center_wheel,
                    raycast_info.contact_point_ws.into(),
                    palettes::basic::GREEN,
                );
            }
        }
    }
}

pub fn show_vehicle_origin_gizmos(
    time: Res<Time>,
    mut gizmos: Gizmos<VehicleControllerGizmos>,
    q_vehicles: Query<(&GlobalTransform, &VehicleType)>,
) {
    for (position, _vehicle_type) in q_vehicles.iter() {
        gizmos.sphere(
            Isometry3d::from_translation(position.translation()),
            5.0 * (time.elapsed_secs() * 0.25).sin(),
            palettes::basic::PURPLE,
        );
    }
}
