use crate::instancing3d::{InstanceBuffer, InstanceData, InstanceMaterialData};
use crate::prep_vertex_buffer::{GpuRenderConfig, RenderConfig, RenderMode, WgPrepVertexBuffer};
use crate::resources::{AppState, PhysicsContext, RunState, Timestamps};
use crate::step::TimestampChannel;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::math::{Vec3, Vec4};
use bevy::pbr::AmbientLight;
use bevy::prelude::*;
use bevy::render::render_resource::BufferUsages;
use bevy::render::renderer::RenderDevice;
use bevy::render::view::NoFrustumCulling;
use nalgebra::point;
use std::sync::Arc;
use wgcore::hot_reloading::HotReloadState;
use wgcore::tensor::GpuVector;
use wgcore::timestamps::GpuTimestamps;
use wgcore::Shader;
use wgpu::Features;
use wgsparkl3d::pipeline::MpmPipeline;
use wgsparkl3d::rapier::parry::math::Isometry;

/// set up a simple 3D scene
pub fn setup_app(mut commands: Commands, device: Res<RenderDevice>) {
    // app state
    let render_config = RenderConfig::new(RenderMode::Default);
    let gpu_render_config = GpuRenderConfig::new(device.wgpu_device(), render_config);
    let prep_vertex_buffer = WgPrepVertexBuffer::from_device(device.wgpu_device()).unwrap();

    let mut hot_reload = HotReloadState::new().unwrap();
    let pipeline = MpmPipeline::new(device.wgpu_device()).unwrap();
    pipeline.init_hot_reloading(&mut hot_reload);

    commands.insert_resource(AppState {
        render_config,
        gpu_render_config,
        prep_vertex_buffer,
        pipeline,
        run_state: RunState::Running,
        num_substeps: 1,
        gravity_factor: 1.0,
        restarting: false,
        selected_scene: 0,
        hot_reload,
        particles_initialized: false,
    });

    let (snd, rcv) = async_channel::unbounded();
    commands.insert_resource(TimestampChannel { snd, rcv });

    let features = device.features();
    let timestamps = features
        .contains(Features::TIMESTAMP_QUERY)
        .then(|| GpuTimestamps::new(device.wgpu_device(), 512));
    commands.insert_resource(Timestamps {
        timestamps,
        ..Default::default()
    });
}

pub fn setup_graphics(
    mut commands: Commands,
    device: Res<RenderDevice>,
    physics: Option<Res<PhysicsContext>>,
    mut meshes: ResMut<Assets<Mesh>>,
    inited_particles: Query<Entity, With<InstanceMaterialData>>,
) {
    let Some(physics) = physics else {
        return;
    };

    if !inited_particles.is_empty() {
        return; // The render particles are already initialized.
    }

    setup_particles_graphics(&mut commands, &device, &*physics, &mut meshes);
}

fn setup_particles_graphics(
    commands: &mut Commands,
    device: &RenderDevice,
    physics: &PhysicsContext,
    meshes: &mut Assets<Mesh>,
) {
    let device = device.wgpu_device();
    let colors = [
        Color::srgb_u8(234, 208, 168),
        Color::srgb_u8(182, 159, 102),
        Color::srgb_u8(107, 84, 40),
        Color::srgb_u8(118, 85, 43),
        Color::srgb_u8(64, 41, 5),
        Color::srgb_u8(89, 58, 14),
    ];
    let radius = physics.particles[0].volume.init_radius();
    let cube = meshes.add(Cuboid {
        half_size: Vec3::splat(radius),
    });

    let mut instances = vec![];
    for (rb_id, particle) in physics.particles.iter().enumerate() {
        let base_color = colors[rb_id % colors.len()].to_linear().to_f32_array();
        instances.push(InstanceData {
            deformation: [Vec4::X, Vec4::Y, Vec4::Z],
            position: Vec4::new(
                particle.position.x,
                particle.position.y,
                particle.position.z,
                0.0,
            ),
            base_color,
            color: base_color,
        });
    }

    let instances_buffer = GpuVector::init(
        device,
        &instances,
        BufferUsages::STORAGE | BufferUsages::VERTEX,
    );

    let num_instances = instances.len();
    commands.spawn((
        Mesh3d(cube),
        SpatialBundle::INHERITED_IDENTITY,
        InstanceMaterialData {
            data: instances,
            buffer: InstanceBuffer {
                buffer: Arc::new(instances_buffer.into_inner().into()),
                length: num_instances,
            },
        },
        NoFrustumCulling,
    ));
}
