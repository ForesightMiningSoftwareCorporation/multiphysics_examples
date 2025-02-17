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
                    self.swing.desired += 0.1 * elapsed * def.swing.sensitivity;
                }
                KeyCode::KeyG => {
                    self.swing.desired -= 0.1 * elapsed * def.swing.sensitivity;
                }
                // boom
                KeyCode::KeyY => {
                    self.boom.desired += 0.1 * elapsed * def.boom.sensitivity;
                }
                KeyCode::KeyH => {
                    self.boom.desired -= 0.1 * elapsed * def.boom.sensitivity;
                }
                // stick
                KeyCode::KeyU => {
                    self.stick.desired += 0.1 * elapsed * def.stick.sensitivity;
                }
                KeyCode::KeyJ => {
                    self.stick.desired -= 0.1 * elapsed * def.stick.sensitivity;
                }
                // bucket base
                KeyCode::KeyI => {
                    self.bucket_base.desired += 0.1 * elapsed * def.bucket_base.sensitivity;
                }
                KeyCode::KeyK => {
                    self.bucket_base.desired -= 0.1 * elapsed * def.bucket_base.sensitivity;
                }
                // bucket jaw
                KeyCode::KeyO => {
                    self.bucket_jaw.desired += 0.1 * elapsed * def.bucket_jaw.sensitivity;
                }
                KeyCode::KeyL => {
                    self.bucket_jaw.desired -= 0.1 * elapsed * def.bucket_jaw.sensitivity;
                }
                _ => {}
            }
        }
    }

    pub fn add(&mut self, other: &Self) {
        self.swing.desired += other.swing.desired;
        self.boom.desired = (self.boom.desired + other.boom.desired).clamp(0.0, 1.0);
        self.stick.desired = (self.stick.desired + other.stick.desired).clamp(0.0, 1.0);
        self.bucket_base.desired =
            (self.bucket_base.desired + other.bucket_base.desired).clamp(0.0, 1.0);
        self.bucket_jaw.desired =
            (self.bucket_jaw.desired + other.bucket_jaw.desired).clamp(0.0, 1.0);
    }
}
