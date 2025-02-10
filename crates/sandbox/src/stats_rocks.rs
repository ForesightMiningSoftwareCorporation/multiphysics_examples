use bevy_egui::{egui, EguiContexts};

use bevy::prelude::*;
use shared_map::rock::Rock;

pub struct StatsRocksPlugin;

impl Plugin for StatsRocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_rock_count);
    }
}

pub fn ui_rock_count(mut contexts: EguiContexts, q_rocks: Query<&Rock>) {
    let rock_count = q_rocks.iter().count();
    egui::Window::new("Rocks Count").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Rocks: {}", rock_count));
    });
}
