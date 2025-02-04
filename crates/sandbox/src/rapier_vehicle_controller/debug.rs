use bevy::{color::palettes, prelude::*};
use bevy_rapier3d::render::DebugRenderContext;

use super::{VehicleController, VehicleControllerParameters};

pub struct VehicleControllerDebugPlugin;

impl Plugin for VehicleControllerDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<VehicleControllerGizmos>()
            .add_systems(Update, show_wheels_gizmos);
    }
}

// Gizmo group for vehicle controller debugging.
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct VehicleControllerGizmos {}

pub fn show_wheels_gizmos(
    mut gizmos: Gizmos<VehicleControllerGizmos>,
    q_vehicles: Query<(
        &GlobalTransform,
        &VehicleControllerParameters,
        &VehicleController,
    )>,
    rapier_debug: Res<DebugRenderContext>,
) {
    if rapier_debug.enabled == false {
        return;
    }
    for (global_transform, parameters, controller) in q_vehicles.iter() {
        for w in controller.controller.wheels() {
            let global_center_wheel = w.center().into();
            gizmos.sphere(
                global_center_wheel,
                parameters.wheel_radius,
                palettes::basic::RED,
            );
            let raycast_info = w.raycast_info();
            if (raycast_info.is_in_contact) {
                gizmos.line(
                    global_center_wheel,
                    raycast_info.contact_point_ws.into(),
                    palettes::basic::GREEN,
                );
            }
        }
    }
}
