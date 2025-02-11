use bevy_egui::{egui, EguiContexts};

use bevy::{color::palettes, prelude::*, render::primitives::Aabb};
use bevy_math::bounding::Aabb3d;
use bevy_rapier3d::plugin::ReadDefaultRapierContext;
use shared_map::rock::Rock;

pub struct StatsRocksPlugin;

#[derive(Debug, Component, Reflect)]
#[require(Aabb)]
pub struct CountRocksInZone(pub usize);

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct RockStatsGizmos;

impl Plugin for StatsRocksPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CountRocksInZone>();
        app.register_type::<RockStatsGizmos>();
        app.init_gizmo_group::<RockStatsGizmos>();
        app.world_mut()
            .get_resource_mut::<GizmoConfigStore>()
            .unwrap()
            .config_mut::<RockStatsGizmos>()
            .0
            .enabled = false;
        app.add_systems(Update, (count_rocks, ui_rock_count).chain());
        app.add_systems(Update, debug_visual_count_rocks_in_zone);
    }
}

pub fn ui_rock_count(
    mut contexts: EguiContexts,
    q_rocks: Query<&Rock>,
    q_piles: Query<(&Name, &CountRocksInZone)>,
) {
    let rock_count = q_rocks.iter().count();
    egui::Window::new("Rocks Count").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Total Rocks: {}", rock_count));
        ui.label(format!("Piles:"));
        for (name, count) in q_piles.iter() {
            ui.label(format!("{}: {}", name.to_string(), count.0));
        }
    });
}

pub fn count_rocks(
    rapier_context: ReadDefaultRapierContext,
    mut q_zones: Query<(&GlobalTransform, &mut CountRocksInZone, &Aabb)>,
    q_rocks: Query<Entity, With<Rock>>,
) {
    let context = rapier_context.single();
    for (gt, mut zone, aabb) in q_zones.iter_mut() {
        let mut amount = 0;
        // Using a query to get the rocks in the zone.
        // Using a sensor and a cache may be better for performance,
        // but this implementation is simpler.
        context.colliders_with_aabb_intersecting_aabb(
            Aabb3d::new(
                gt.transform_point(Vec3::from(aabb.center)),
                aabb.half_extents,
            ),
            |e| {
                // check if that's a rock.
                if q_rocks.get(e).is_ok() {
                    amount += 1;
                }
                return true;
            },
        );
        zone.0 = amount;
    }
}

pub fn debug_visual_count_rocks_in_zone(
    mut gizmos: Gizmos<RockStatsGizmos>,
    q_zones: Query<(&GlobalTransform, &Aabb), With<CountRocksInZone>>,
) {
    for (gt, aabb) in q_zones.iter() {
        gizmos.cuboid(
            Transform::from_translation(gt.transform_point(Vec3::from(aabb.center)))
                .with_scale(Vec3::from(aabb.half_extents) * 2.0),
            palettes::css::BROWN,
        );
    }
}
