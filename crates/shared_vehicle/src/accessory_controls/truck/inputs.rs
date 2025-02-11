use bevy::prelude::*;

use super::{controls::TruckControls, TruckDef};

impl TruckControls {
    pub fn integrate_inputs(
        &mut self,
        elapsed: f32,
        inputs: &Res<ButtonInput<KeyCode>>,
        def: &TruckDef,
    ) {
        for key in inputs.get_pressed() {
            match *key {
                // Main dump
                KeyCode::KeyT => {
                    self.main_dump += 0.1 * elapsed * def.main_dump.sensitivity;
                }
                KeyCode::KeyG => {
                    self.main_dump -= 0.1 * elapsed * def.main_dump.sensitivity;
                }
                _ => {}
            }
        }
    }

    pub fn add(&mut self, other: &Self) {
        self.main_dump = (self.main_dump + other.main_dump).clamp(0.0, 1.0);
    }
}
