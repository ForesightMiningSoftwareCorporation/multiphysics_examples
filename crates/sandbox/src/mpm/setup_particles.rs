use bevy::asset::ron;
use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;
use bevy_rapier3d::geometry::RapierColliderHandle;
use bevy_rapier3d::plugin::ReadRapierContext;
use bevy_rapier3d::prelude::RapierContext;
use bevy_wgsparkl::components::MpmCouplingEnabled;
use bevy_wgsparkl::resources::{AppState, PhysicsContext};
use nalgebra::{point, RealField, Rotation3};
use nalgebra::{vector, Similarity3, Vector3};
use parry3d::bounding_volume::Aabb;
use shared_map::map_def::{MapDef, MapDefHandle};
use wgebra::GpuSim3;
use wgparry3d::parry::shape::{Cuboid, TriMesh};
use wgrapier3d::dynamics::body::{BodyCoupling, BodyCouplingEntry};
use wgrapier3d::dynamics::BodyDesc;
use wgsparkl3d::models::DruckerPrager;
use wgsparkl3d::rapier::dynamics::RigidBodySet;
use wgsparkl3d::rapier::geometry::Ray;
use wgsparkl3d::rapier::prelude::{ColliderBuilder, ColliderSet, RigidBodyBuilder};
use wgsparkl3d::{
    models::ElasticCoefficients,
    pipeline::MpmData,
    solver::{Particle, ParticlePhase, SimulationParams},
};
use wgsparkl3d::solver::ParticleDynamics;

pub fn setup_mpm_particles(
    mut commands: Commands,
    device: Res<RenderDevice>,
    mut app_state: ResMut<AppState>,
    rapier: ReadRapierContext,
    coupling: Query<&RapierColliderHandle, With<MpmCouplingEnabled>>,
    map_defs_handles: Query<Ref<MapDefHandle>>,
    map_defs: Res<Assets<MapDef>>,
) {
    if rapier.rapier_context.get_single().is_err() {
        return; // Rapier isn’t initialized yet.
    }

    let rapier = rapier.single();

    if rapier.colliders.colliders.is_empty() || coupling.iter().len() < 5 {
        return; // Rapier isn’t initialized yet.
    }

    let Ok(map_def_handle) = map_defs_handles.get_single() else {
        return;
    };

    if app_state.particles_initialized {
        return; // Already initialized.
    }

    let Some(map_def): Option<&MapDef> = map_defs.get(&map_def_handle.0) else {
        return;
    };

    app_state.particles_initialized = true;

    let coupling: Vec<_> = coupling
        .iter()
        .map(|co_handle| {
            let co = &rapier.colliders.colliders[co_handle.0];
            println!("Coupled collider: {:?}", co.shape().shape_type());
            println!(
                "Coupled collider pose: {:?}",
                co.position().translation.vector
            );
            let rb_handle = co.parent().unwrap();
            BodyCouplingEntry {
                body: rb_handle,
                collider: co_handle.0,
                mode: BodyCoupling::OneWay, // TODO: try out two-ways for the buldozer and the truck.
            }
        })
        .collect();

    let device = device.wgpu_device();

    if !app_state.restarting {
        app_state.num_substeps = 2;
        app_state.gravity_factor = 1.0;
    };

    let params = SimulationParams {
        gravity: vector![0.0, 0.0, -9.81] * app_state.gravity_factor,
        dt: (1.0 / 60.0) / (app_state.num_substeps as f32),
    };

    let cell_width = 0.5;
    let mut particles = vec![];

    'next_rock: for rock in &map_def.rocks {
        let mut position = vector![rock.translation.x, rock.translation.y, rock.translation.z];

        // HACK: remove any particle that starts below any mesh (and, in particular, the ground).
        for (_, collider) in rapier.colliders.colliders.iter() {
            if collider.shape().intersects_ray(
                collider.position(),
                &Ray::new(position.into(), Vector3::z()),
                f32::MAX,
            ) {
                // Discard any rock that starts below the topography.
                continue 'next_rock;
            }
        }

        let rock_size = vector![1.0, 1.0, 1.0];
        let rock_aabb = Aabb::from_half_extents(position.into(), rock_size / 2.0);

        for subrock in rock_aabb.split_at_center() {
            let subrock_size = subrock.extents();
            let volume = subrock_size.x * subrock_size.y * subrock_size.z;
            let density = 2700.0;
            let radius = volume.cbrt() / 2.0;
            particles.push(Particle {
                position: subrock.center().coords,
                dynamics: ParticleDynamics::with_density(radius, density),
                model: ElasticCoefficients::from_young_modulus(10_000_000.0, 0.2),
                plasticity: Some(DruckerPrager {
                    // TODO: tune these values.
                    h0: 75.0f32.to_radians(),
                    h1: 90.0f32.to_radians(),
                    h2: 0.6,
                    h3: 30.0f32.to_radians(),
                    ..DruckerPrager::new(10_000_000.0, 0.2)
                }),
                phase: None,
            });
        }
    }

    println!(
        "Num removed particles: {}/{} (simulated: {})",
        map_def.rocks.len() - particles.len() / 8,
        map_def.rocks.len(),
        particles.len()
    );

    println!("Coupled: {}", coupling.len());

    let data = MpmData::with_select_coupling(
        device,
        params,
        &particles,
        &rapier.rigidbody_set.bodies,
        &rapier.colliders.colliders,
        coupling,
        cell_width,
        60_000,
    );
    commands.insert_resource(PhysicsContext { data, particles });
}
