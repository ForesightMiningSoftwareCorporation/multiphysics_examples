use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::{plugin::WriteDefaultRapierContext, prelude::RapierRigidBodyHandle};

use crate::{
    rapier_vehicle_controller::{VehicleController, VehicleControllerParameters},
    vehicle_spawner::VehicleType,
};

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CurrentSelection>();
        app.register_type::<VehicleControllerParameters>();

        app.init_resource::<CurrentSelection>();

        app.add_systems(Update, init_vehicle_controller);
        app.add_systems(
            FixedUpdate,
            (update_vehicle_controls, update_vehicle_controller).chain(),
        );
        app.add_systems(Update, (ui_cycle_vehicles, ui_controls));
    }
}

#[derive(Debug, Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct CurrentSelection {
    pub entity: Option<Entity>,
}

fn ui_cycle_vehicles(
    inputs: Res<ButtonInput<KeyCode>>,
    mut current_selection: ResMut<CurrentSelection>,
    q_vehicles: Query<Entity, With<VehicleType>>,
) {
    if inputs.just_pressed(KeyCode::Tab) {
        let Some(current_entity) = current_selection.entity else {
            current_selection.entity = q_vehicles.iter().next();
            return;
        };
        let current_entity_index = q_vehicles.iter().position(|e| e == current_entity);
        let Some(current_entity_index) = current_entity_index else {
            warn!("Current selected entity not found in vehicle query, resetting to first entity");
            current_selection.entity = q_vehicles.iter().next();
            return;
        };
        let vehicles = q_vehicles.iter().collect::<Vec<_>>();
        let next_entity = vehicles
            .iter()
            .cycle()
            .skip(current_entity_index + 1)
            .next()
            .unwrap();
        current_selection.entity = Some(*next_entity);
    }
}

/// To select a vehicle to control through UI.
fn ui_controls(
    mut contexts: EguiContexts,
    mut current_selection: ResMut<CurrentSelection>,
    q_vehicles: Query<(Entity, &VehicleType, Option<&Name>)>,
) {
    egui::Window::new("Control").show(contexts.ctx_mut(), |ui| {
        ui.label("Press tab to cycle through vehicles");

        let current_vehicle_type = current_selection.entity.and_then(|e| {
            q_vehicles
                .get(e)
                .map(|(_, vehicle_type, name)| (*vehicle_type, name.clone()))
                .ok()
        });
        let selected_display = match current_vehicle_type {
            Some((_, Some(name))) => name.to_string(),
            Some((vehicle_type, _)) => format!("{}", vehicle_type),
            None => "None".to_string(),
        };
        ui.label(format!("current vehicle: {selected_display}"));
        let other_vehicles = q_vehicles
            .iter()
            .flat_map(|(e, vehicle_type, name)| {
                if let Some(current_selection) = current_selection.entity {
                    if e == current_selection {
                        return None;
                    }
                }
                Some((
                    e,
                    match (vehicle_type, name) {
                        (_, Some(name)) => name.to_string(),
                        (vehicle_type, _) => format!("{}", vehicle_type),
                    },
                ))
            })
            .collect::<Vec<_>>();
        let current_selection_clone = current_selection.entity.clone();
        egui::ComboBox::from_label("Select vehicle")
            .selected_text(&selected_display)
            .show_ui(ui, |ui| {
                ui.selectable_label(true, selected_display);
                for (e, display_name) in other_vehicles {
                    ui.selectable_value(&mut current_selection.entity, Some(e), display_name);
                }
            });
    });
}

/// System to initialize and insert a [`VehicleController`] after bevy_rapier initializes the rigidbody.
pub fn init_vehicle_controller(
    mut commands: Commands,
    vehicle: Query<
        (Entity, &VehicleControllerParameters, &RapierRigidBodyHandle),
        Added<RapierRigidBodyHandle>,
    >,
) {
    for (entity, vehicle_parameters, body_handle) in vehicle.iter() {
        let controller = VehicleController::new(body_handle.0, vehicle_parameters);
        commands.entity(entity).insert(controller);
    }
}

/// System to forward controls to [`VehicleController`]
pub fn update_vehicle_controls(
    current_selection: Res<CurrentSelection>,
    inputs: Res<ButtonInput<KeyCode>>,
    mut vehicles: Query<(Entity, &mut VehicleController, &VehicleControllerParameters)>,
) {
    for (e, mut vehicle_controller, parameters) in vehicles.iter_mut() {
        if current_selection.entity != Some(e) {
            vehicle_controller.stop();
            continue;
        }
        vehicle_controller.integrate_actions(&inputs, parameters);
    }
}

/// System to initialize and insert a [`VehicleController`] after bevy_rapier initializes the rigidbody.
///
pub fn update_vehicle_controller(
    time: Res<Time>,
    mut vehicles: Query<&mut VehicleController>,
    mut context: WriteDefaultRapierContext,
) {
    for mut vehicle_controller in vehicles.iter_mut() {
        let context = &mut *context;
        vehicle_controller.update_vehicle(
            time.delta_secs(),
            &mut context.bodies,
            &context.colliders,
            &context.query_pipeline,
        );
    }
}
