use bevy::prelude::Resource;
use wgcore::hot_reloading::HotReloadState;
use wgcore::timestamps::GpuTimestamps;
use wgsparkl3d::pipeline::{MpmData, MpmPipeline};
use wgsparkl3d::rapier::prelude::{CCDSolver, ColliderSet, DefaultBroadPhase, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase, PhysicsPipeline, RigidBodySet};
use wgsparkl3d::solver::Particle;
use crate::prep_vertex_buffer::{GpuRenderConfig, RenderConfig, WgPrepVertexBuffer};

#[derive(Resource)]
pub struct AppState {
    pub run_state: RunState,
    pub render_config: RenderConfig,
    pub gpu_render_config: GpuRenderConfig,
    pub pipeline: MpmPipeline,
    pub prep_vertex_buffer: WgPrepVertexBuffer,
    pub num_substeps: usize,
    pub gravity_factor: f32,
    pub restarting: bool,
    pub selected_scene: usize,
    pub hot_reload: HotReloadState,
    pub particles_initialized: bool,
}

#[derive(Resource)]
pub struct PhysicsContext {
    pub data: MpmData,
    pub particles: Vec<Particle>,
}

// #[derive(Resource, Default)]
// pub struct RenderContext {
//     pub instanced_materials: InstancedMaterials,
//     pub prefab_meshes: HashMap<ShapeType, Handle<Mesh>>,
//     pub rigid_entities: Vec<EntityWithGraphics>,
// }

#[derive(Resource, Default)]
pub struct Timestamps {
    pub timestamps: Option<GpuTimestamps>,
    pub grid_sort: f64,
    pub grid_update_cdf: f64,
    pub p2g_cdf: f64,
    pub g2p_cdf: f64,
    pub p2g: f64,
    pub grid_update: f64,
    pub g2p: f64,
    pub particles_update: f64,
    pub integrate_bodies: f64,
}

impl Timestamps {
    pub fn total_time(&self) -> f64 {
        self.grid_sort
            + self.grid_update_cdf
            + self.p2g_cdf
            + self.g2p_cdf
            + self.p2g
            + self.grid_update
            + self.g2p
            + self.particles_update
            + self.integrate_bodies
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RunState {
    Running,
    Paused,
    Step,
}
