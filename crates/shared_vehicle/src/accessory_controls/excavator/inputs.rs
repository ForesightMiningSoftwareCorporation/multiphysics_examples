use bevy::prelude::*;

use super::{controls::ExcavatorControls, ExcavatorDef};

impl ExcavatorControls {
    pub fn integrate_inputs(
        &mut self,
        elapsed: f32,
        inputs: &Res<ButtonInput<KeyCode>>,
        def: &ExcavatorDef,
    ) {
        for key in inputs.get_pressed() {
            match *key {
                // swing
                KeyCode::KeyT => {
                    self.swing += 0.1 * elapsed * def.swing.sensitivity;
                }
                KeyCode::KeyG => {
                    self.swing -= 0.1 * elapsed * def.swing.sensitivity;
                }
                // boom
                KeyCode::KeyY => {
                    self.boom += 0.1 * elapsed * def.boom.sensitivity;
                }
                KeyCode::KeyH => {
                    self.boom -= 0.1 * elapsed * def.boom.sensitivity;
                }
                // stick
                KeyCode::KeyU => {
                    self.stick += 0.1 * elapsed * def.stick.sensitivity;
                }
                KeyCode::KeyJ => {
                    self.stick -= 0.1 * elapsed * def.stick.sensitivity;
                }
                // bucket base
                KeyCode::KeyI => {
                    self.bucket_base += 0.1 * elapsed * def.bucket_base.sensitivity;
                }
                KeyCode::KeyK => {
                    self.bucket_base -= 0.1 * elapsed * def.bucket_base.sensitivity;
                }
                // bucket jaw
                KeyCode::KeyO => {
                    self.bucket_jaw += 0.1 * elapsed * def.bucket_jaw.sensitivity;
                }
                KeyCode::KeyL => {
                    self.bucket_jaw -= 0.1 * elapsed * def.bucket_jaw.sensitivity;
                }
                _ => {}
            }
        }
    }

    pub fn add(&mut self, other: &Self) {
        self.swing += other.swing;
        self.boom = (self.boom + other.boom).clamp(0.0, 1.0);
        self.stick = (self.stick + other.stick).clamp(0.0, 1.0);
        self.bucket_base = (self.bucket_base + other.bucket_base).clamp(0.0, 1.0);
        self.bucket_jaw = (self.bucket_jaw + other.bucket_jaw).clamp(0.0, 1.0);
    }
}
