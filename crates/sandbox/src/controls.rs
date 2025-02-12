use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_rapier3d::{
    plugin::{TimestepMode, WriteDefaultRapierContext},
    prelude::{Friction, RapierRigidBodyHandle},
};

use shared_vehicle::{
    accessory_controls::{
        excavator::{controls::ExcavatorControls, ExcavatorDef, ExcavatorDefHandle},
        truck::{
            controls::{TruckControls, TruckMeshMapping},
            TruckDef, TruckDefHandle,
        },
    },
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
            Update,
            (update_vehicle_controls, update_vehicle_controller).chain(),
        );
        app.add_systems(
            Update,
            (
                update_excavator_controls,
                update_truck_controls,
                update_truck_dump_friction,
            ),
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
            .nth(current_entity_index + 1)
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
        ui.label("Press TAB to cycle through vehicles");

        let current_vehicle_type = current_selection.entity.and_then(|e| {
            q_vehicles
                .get(e)
                .map(|(_, vehicle_type, name)| (*vehicle_type, name))
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
        egui::ComboBox::from_label("Select vehicle")
            .selected_text(&selected_display)
            .show_ui(ui, |ui| {
                _ = ui.selectable_label(true, selected_display);
                for (e, display_name) in other_vehicles {
                    ui.selectable_value(&mut current_selection.entity, Some(e), display_name);
                }
            });
        if let Some((vehicle_type, _)) = current_vehicle_type {
            ui.group(|ui| {
                ui.label("Arrow keys to move.");
                match vehicle_type {
                    VehicleType::Excavator => {
                        ui.label("T,G to move boom");
                        ui.label("U,J to move stick");
                        ui.label("I,K to move bucket base");
                        ui.label("O,L to move bucket jaw");
                    }
                    VehicleType::Truck => {
                        ui.label("T,G to move dump");
                    }
                    _ => {}
                }
            });
        }
        ui.group(|ui| {
            ui.label("Press ESC to show inspector egui");
            ui.label("Press D to show Debug renderer");
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

/// System to initialize and insert a [`VehicleController`] after bevy_rapier initializes the rigidbody.
///
pub fn update_vehicle_controller(
    time: Res<Time>,
    timestep_mode: Res<TimestepMode>,
    mut vehicles: Query<&mut VehicleController>,
    mut context: WriteDefaultRapierContext,
) {
    for mut vehicle_controller in vehicles.iter_mut() {
        let context = &mut *context;
        // capping delta time to max_dt, or we'll issue a move command that is too big,
        // resulting in an excavator difficult to control.
        let dt = match *timestep_mode {
            TimestepMode::Variable { max_dt, .. } => time.delta_secs().min(max_dt),
            _ => time.delta_secs(),
        };
        vehicle_controller.update_vehicle(
            dt,
            &mut context.bodies,
            &context.colliders,
            &context.query_pipeline,
        );
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

/// System to forward controls to [`ExcavatorControls`]
pub fn update_excavator_controls(
    current_selection: Res<CurrentSelection>,
    excavator_def: Res<Assets<ExcavatorDef>>,
    time: Res<Time>,
    inputs: Res<ButtonInput<KeyCode>>,
    timestep_mode: Res<TimestepMode>,
    mut q_controls: Query<(Entity, &mut ExcavatorControls, &mut ExcavatorDefHandle)>,
) {
    for (entity, mut control, def_handle) in q_controls.iter_mut() {
        if current_selection.entity != Some(entity) {
            continue;
        }
        let Some(def) = excavator_def.get(&def_handle.0) else {
            continue;
        };
        // capping delta time to max_dt, or we'll issue a move command that is too big,
        // resulting in an excavator difficult to control.
        let dt = match *timestep_mode {
            TimestepMode::Variable { max_dt, .. } => time.delta_secs().min(max_dt),
            _ => time.delta_secs(),
        };
        let mut control_change = ExcavatorControls::default();
        control_change.integrate_inputs(dt, &inputs, def);
        if control_change != ExcavatorControls::default() {
            control.add(&control_change);
        }
    }
}

/// System to forward controls to [`TruckControls`]
pub fn update_truck_controls(
    current_selection: Res<CurrentSelection>,
    truck_def: Res<Assets<TruckDef>>,
    time: Res<Time>,
    inputs: Res<ButtonInput<KeyCode>>,
    timestep_mode: Res<TimestepMode>,
    mut q_controls: Query<(Entity, &mut TruckControls, &mut TruckDefHandle)>,
) {
    for (entity, mut control, def_handle) in q_controls.iter_mut() {
        if current_selection.entity != Some(entity) {
            continue;
        }
        let Some(def) = truck_def.get(&def_handle.0) else {
            continue;
        };
        // capping delta time to max_dt, or we'll issue a move command that is too big,
        // resulting in a truck difficult to control.
        let dt = match *timestep_mode {
            TimestepMode::Variable { max_dt, .. } => time.delta_secs().min(max_dt),
            _ => time.delta_secs(),
        };
        let mut control_change = TruckControls::default();
        control_change.integrate_inputs(dt, &inputs, def);
        if control_change != TruckControls::default() {
            control.add(&control_change);
        }
    }
}

/// to help with sliding rocks, we'll lower the friction of the truck dump.
pub fn update_truck_dump_friction(
    current_selection: Res<CurrentSelection>,
    mut q_controls: Query<(Entity, &mut TruckControls, &TruckMeshMapping)>,
    mut q_friction: Query<&mut Friction>,
) {
    for (entity, control, mapping) in q_controls.iter_mut() {
        if current_selection.entity != Some(entity) {
            continue;
        }
        _ = q_friction.get_mut(mapping.main_dump).map(|mut friction| {
            friction.coefficient = 1f32 - control.main_dump;
        });
    }
}
