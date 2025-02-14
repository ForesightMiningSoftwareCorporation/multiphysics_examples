//! Thin wrapper around [rapier's raycast vehicle controller](https://github.com/dimforge/rapier/blob/master/examples3d/vehicle_controller3.rs).

use bevy::prelude::*;
use bevy::reflect::Reflect;
use bevy_rapier3d::{
    na::Vector,
    rapier::{
        control::{DynamicRayCastVehicleController, WheelTuning},
        prelude::{ColliderSet, QueryPipeline, RigidBodyHandle, RigidBodySet},
    },
};

pub mod debug;
pub mod reflect;

/// Parameters to initialize a [`VehicleController`].
///
/// This can be used as a component to automatically initialize a [`VehicleController`]
/// when the associated Entity has a [`RapierRigidBodyHandle`][bevy_rapier3d::prelude::RapierRigidBodyHandle].
///
/// It is practical because idiomatic bevy_rapier usually does not create the rigidbody instantly.
///
/// See <https://github.com/soraxas/rapier/blob/8ef99688f0d83f9c2bc67ced26ac773e2426f7b4/examples3d/vehicle_controller3.rs#L26-L27> for example of "good" configuration.
#[derive(Component, Clone, Debug, Reflect)]
pub struct VehicleControllerParameters {
    /// Positions to be passed to [`DynamicRayCastVehicleController::add_wheel`]
    pub wheel_positions: [Vec3; 4],
    #[reflect(remote = reflect::WheelTuningWrapper)]
    pub wheel_tuning: WheelTuning,
    pub suspension_rest_length: f32,
    pub wheel_radius: f32,
    /// if true, wheels do not turn, only engine force is applied to the opposite side of the turn.
    ///
    /// This is useful for vehicles that have continuous tracks.
    pub crawler: bool,
    /// corresponds to [`Wheel`][bevy_rapier3d::rapier::control::Wheel]
    ///
    /// \[0\] is front, \[1\] is back
    pub wheel_brake: [f32; 2],
    /// force applied to [`Wheel::engine_force`][bevy_rapier3d::rapier::control::Wheel::engine_force] depending on inputs.
    pub engine_force: f32,
}

impl Default for VehicleControllerParameters {
    /// Returns the same parameters as rapier official examples.
    fn default() -> Self {
        let hw = 0.3;
        let hh = 0.15;
        Self::empty().with_wheel_positions_for_half_size(Vec3::new(hw, hh, hw))
    }
}

impl VehicleControllerParameters {
    pub fn empty() -> VehicleControllerParameters {
        VehicleControllerParameters {
            wheel_positions: [Vec3::ZERO; 4],
            wheel_tuning: WheelTuning::default(),
            suspension_rest_length: 0.0,
            wheel_radius: 0.5,
            crawler: false,
            wheel_brake: [0.25, 0.25],
            engine_force: 10.0,
        }
    }
    pub fn with_wheel_positions_for_half_size(
        mut self,
        half_size: Vec3,
    ) -> VehicleControllerParameters {
        let Vec3 {
            x: width,
            y: length,
            z: height,
        } = half_size;
        self.wheel_positions = [
            [-width * 1.5, length, -height - 1.0].into(),
            [width * 1.5, length, -height - 1.0].into(),
            [-width * 1.5, -length, -height - 1.0].into(),
            [width * 1.5, -length, -height - 1.0].into(),
        ];
        self.suspension_rest_length = height;
        self.wheel_radius = height / 2.0;
        self
    }
    pub fn with_wheel_tuning(mut self, wheel_tuning: WheelTuning) -> Self {
        self.wheel_tuning = wheel_tuning;
        self
    }
    pub fn with_crawler(mut self, is_crawler: bool) -> Self {
        self.crawler = is_crawler;
        self
    }
}

#[derive(Component)]
pub struct VehicleController {
    pub controller: DynamicRayCastVehicleController,
}

impl VehicleController {
    pub fn new(
        body_chassis: RigidBodyHandle,
        parameters: &VehicleControllerParameters,
    ) -> VehicleController {
        /*
         * Vehicle we will control manually.
         */
        let mut vehicle = DynamicRayCastVehicleController::new(body_chassis);
        if !parameters.crawler {
            vehicle.index_up_axis = 2;
            vehicle.index_forward_axis = 1;
        }

        for (i, pos) in parameters.wheel_positions.iter().enumerate() {
            let wheel = vehicle.add_wheel(
                (*pos).into(),
                -Vector::z(),
                Vector::x(),
                parameters.suspension_rest_length,
                parameters.wheel_radius,
                &parameters.wheel_tuning,
            );
            wheel.brake = parameters.wheel_brake[i / 2];
        }
        VehicleController {
            controller: vehicle,
        }
    }

    fn integrate_actions_crawler(
        &mut self,
        inputs: &ButtonInput<KeyCode>,
        parameters: &VehicleControllerParameters,
    ) {
        let mut engine_force_base = 0f32;
        let mut engine_force_right = 0f32;
        let mut engine_force_left = 0f32;

        for key in inputs.get_pressed() {
            match *key {
                KeyCode::ArrowRight => {
                    engine_force_left += parameters.engine_force;
                }
                KeyCode::ArrowLeft => {
                    engine_force_right += parameters.engine_force;
                }
                KeyCode::ArrowUp => {
                    engine_force_base += parameters.engine_force;
                }
                KeyCode::ArrowDown => {
                    engine_force_base -= parameters.engine_force;
                }
                _ => {}
            }
        }

        let wheels = self.controller.wheels_mut();

        // no steering for the "crawler" mode.
        // front
        wheels[0].engine_force = engine_force_base + engine_force_left * engine_force_base.signum();
        wheels[1].engine_force =
            engine_force_base + engine_force_right * engine_force_base.signum();
        // back
        wheels[2].engine_force = engine_force_base + engine_force_left * engine_force_base.signum();
        wheels[3].engine_force =
            engine_force_base + engine_force_right * engine_force_base.signum();
    }

    pub fn integrate_actions(
        &mut self,
        inputs: &Res<ButtonInput<KeyCode>>,
        parameters: &VehicleControllerParameters,
    ) {
        if parameters.crawler {
            self.integrate_actions_crawler(inputs, parameters);
            return;
        }
        let mut engine_force = 0.0;
        let mut steering_angle = 0.0;

        for key in inputs.get_pressed() {
            match *key {
                KeyCode::ArrowRight => {
                    steering_angle += -0.7;
                }
                KeyCode::ArrowLeft => {
                    steering_angle += 0.7;
                }
                KeyCode::ArrowUp => {
                    engine_force += parameters.engine_force;
                }
                KeyCode::ArrowDown => {
                    engine_force += -parameters.engine_force;
                }
                _ => {}
            }
        }

        let wheels = self.controller.wheels_mut();
        // front wheels are powered and steering.
        wheels[0].engine_force = engine_force;
        wheels[0].steering = steering_angle;
        wheels[1].engine_force = engine_force;
        wheels[1].steering = steering_angle;
    }

    pub fn stop(&mut self) {
        let wheels = self.controller.wheels_mut();
        wheels[0].engine_force = 0f32;
        wheels[0].steering = 0f32;
        wheels[1].engine_force = 0f32;
        wheels[1].steering = 0f32;
        wheels[2].engine_force = 0f32;
        wheels[2].steering = 0f32;
        wheels[3].engine_force = 0f32;
        wheels[3].steering = 0f32;
    }

    pub fn update_vehicle(
        &mut self,
        dt: f32,
        bodies: &mut RigidBodySet,
        colliders: &ColliderSet,
        queries: &QueryPipeline,
    ) {
        self.update_vehicle_with_filter(
            dt,
            bodies,
            colliders,
            queries,
            bevy_rapier3d::rapier::prelude::QueryFilter::exclude_dynamic()
                .exclude_rigid_body(self.controller.chassis),
        );
    }

    pub fn update_vehicle_with_filter(
        &mut self,
        dt: f32,
        bodies: &mut RigidBodySet,
        colliders: &ColliderSet,
        queries: &QueryPipeline,
        filter: bevy_rapier3d::rapier::prelude::QueryFilter,
    ) {
        self.controller
            .update_vehicle(dt, bodies, colliders, queries, filter);
    }
}
