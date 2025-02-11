use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub struct UiGizmoToggle;

impl Plugin for UiGizmoToggle {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_toggle_gizmo);
    }
}

pub fn ui_toggle_gizmo(mut egui_ctx: EguiContexts, mut gizmo_store: ResMut<GizmoConfigStore>) {
    egui::Window::new("Gizmo debug").show(egui_ctx.ctx_mut(), |ui| {
        for (_, config, reflect) in gizmo_store.iter_mut() {
            ui.checkbox(
                &mut config.enabled,
                reflect.reflect_type_ident().unwrap_or("Unknown"),
            );
        }
    });
}
