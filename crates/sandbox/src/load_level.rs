use crate::{controls::CurrentSelection, muck_pile::SpawnMuckPileCommand};

use bevy::{prelude::*, render::view::NoFrustumCulling};
use bevy_editor_cam::prelude::*;
use bevy_math::Vec3A;
use bevy_rapier3d::{prelude::*, rapier::control::WheelTuning};
use shared_map::{
    map_def::{MapDef, MapDefHandle, CONTACT_SKIN},
    rock::SpawnRockCommand,
};
use shared_vehicle::{
    accessory_controls::{
        excavator::{
            controls::{ExcavatorControls, ExcavatorControlsMapping},
            ExcavatorDef, ExcavatorDefHandle,
        },
        truck::{controls::TruckControls, TruckDef, TruckDefHandle},
    },
    rapier_vehicle_controller::VehicleControllerParameters,
    vehicle_spawner::{
        self, bulldozer::BULLDOZER_PATH, excavator::EXCAVATOR_PATH, truck::TRUCK_PATH, VehicleType,
    },
};

#[derive(Resource, Debug, Reflect)]
pub struct LevelResources {
    pub map_def_handle: Handle<MapDef>,
    pub excavator_def: Handle<ExcavatorDef>,
    pub truck_def: Handle<TruckDef>,

    pub bulldozer_model: Handle<Scene>,
    pub excavator_model: Handle<Scene>,
    pub truck_model: Handle<Scene>,

    pub diffuse_map: Handle<Image>,
    pub specular_map: Handle<Image>,
}

pub fn load_level_resources(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LevelResources {
        //map_def_handle: asset_server.load("private/Sim data/transformed/imported_cubes.mapdef.ron"),
        map_def_handle: asset_server.load("mapdef/final.mapdef.ron"),
        excavator_def: asset_server.load("vehicledef/excavator.excavatordef.ron"),
        truck_def: asset_server.load("vehicledef/truck.truckdef.ron"),
        diffuse_map: asset_server.load("environment_maps/diffuse_rgb9e5_zstd.ktx2"),
        specular_map: asset_server.load("environment_maps/specular_rgb9e5_zstd.ktx2"),
        bulldozer_model: asset_server.load(GltfAssetLabel::Scene(0).from_asset(BULLDOZER_PATH)),
        excavator_model: asset_server.load(GltfAssetLabel::Scene(0).from_asset(EXCAVATOR_PATH)),
        truck_model: asset_server.load(GltfAssetLabel::Scene(0).from_asset(TRUCK_PATH)),
    });
}

pub fn spawn_level(
    mut commands: Commands,
    resources: Res<LevelResources>,
    asset_server: Res<AssetServer>,
) {
    let map_handle = resources.map_def_handle.clone();
    // Ground
    let mut map = commands.spawn(MapDefHandle(resources.map_def_handle.clone()));

    map.insert((
        Transform::default().with_rotation(
            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        ),
        CollisionGroups::new(Group::GROUP_2, Group::ALL),
    ));
}

pub fn setup_vehicles(
    mut commands: Commands,
    mut map_def_instances: Query<(Entity, &GlobalTransform, &Transform, &MapDefHandle)>,
    resources: Res<LevelResources>,
    asset_server: Res<AssetServer>,
    map_defs: Res<Assets<MapDef>>,
    mut initialized: Local<bool>,
) {
    for (e, t, tt, map_def_handle) in map_def_instances.iter_mut() {
        let Some(map_def): Option<&MapDef> = map_defs.get(&map_def_handle.0) else {
            continue;
        };

        if *initialized {
            return;
        }
        spawn_vehicles(
            commands.reborrow(),
            &resources,
            &asset_server,
            dbg!(
                map_def.spawn_point.unwrap_or(Vec3::new(0.0, 20.0, 24.0))
                    + Vec3::new(map_def.scale.z / 2.0, map_def.scale.x / 2.0, 0.0)
            ),
        );
        *initialized = true;
    }
}
pub fn spawn_vehicles(
    mut commands: Commands,
    resources: &Res<LevelResources>,
    asset_server: &Res<AssetServer>,
    spawn_point: Vec3,
) {
    // Vehicles

    let wheel_tuning = WheelTuning {
        suspension_stiffness: 80.0,
        suspension_damping: 10.0,
        ..WheelTuning::default()
    };
    // TODO: consider changing the engine speed (or mass of objects) depending on the chosen scale.
    //
    let scale = 2.5f32;
    let mut bulldozer_parameters = VehicleControllerParameters::empty()
        .with_wheel_positions_for_half_size(Vec3::new(0.5, 1.5, 0.4), Vec3::Z * -CONTACT_SKIN)
        .with_wheel_tuning(wheel_tuning)
        .with_crawler(true);
    bulldozer_parameters.wheel_radius *= scale;
    bulldozer_parameters.engine_force = 100.0 * scale * scale;
    bulldozer_parameters
        .wheel_brake
        .iter_mut()
        .for_each(|w| *w = 14.0 * scale);
    bulldozer_parameters
        .wheel_positions
        .iter_mut()
        .for_each(|w| {
            *w *= scale;
        });
    let bulldozer_entity = vehicle_spawner::spawn(
        VehicleType::Bulldozer,
        &mut commands,
        &asset_server,
        Transform::from_translation(spawn_point + Vec3::Z)
            .with_rotation(Quat::from_rotation_z(180f32.to_radians()))
            .with_scale(Vec3::splat(scale)),
    )
    .insert(bulldozer_parameters)
    .id();
    commands.insert_resource(CurrentSelection {
        entity: Some(bulldozer_entity),
    });

    let excavator_def = ExcavatorDefHandle(resources.excavator_def.clone());
    let mut excavator_parameters = VehicleControllerParameters::empty()
        .with_wheel_positions_for_half_size(Vec3::new(0.7, 1.0, 0.4), Vec3::Z * -CONTACT_SKIN)
        .with_wheel_tuning(wheel_tuning)
        .with_crawler(true);
    excavator_parameters.engine_force = 40.0 * scale * scale;
    excavator_parameters.wheel_radius *= scale;
    excavator_parameters
        .wheel_brake
        .iter_mut()
        .for_each(|w| *w = 2.0 * scale);
    excavator_parameters
        .wheel_positions
        .iter_mut()
        .for_each(|w| {
            *w *= scale;
        });
    vehicle_spawner::spawn(
        VehicleType::Excavator,
        &mut commands,
        &asset_server,
        Transform::from_translation(spawn_point + Vec3::new(10.0, 0.0, 0.0))
            .with_rotation(Quat::from_rotation_z(180f32.to_radians()))
            .with_scale(Vec3::splat(scale)),
    )
    .insert(excavator_parameters)
    .insert(excavator_def)
    .insert(ExcavatorControls::default());

    let mut truck_controller_parameters = VehicleControllerParameters {
        wheel_tuning,
        // truck has more mass and uses only 2 power wheels so more powerful wheels.
        engine_force: 400f32 * scale * scale,
        // rear wheel is always braking
        wheel_brake: [10.5f32 * scale * scale, 1.6f32 * scale * scale],
        wheel_positions: [
            Vec3::new(-1.3, 1.6, 0.3 - CONTACT_SKIN),
            Vec3::new(1.3, 1.6, 0.3 - CONTACT_SKIN),
            Vec3::new(-1.3, -1.2, 0.3 - CONTACT_SKIN),
            Vec3::new(1.3, -1.2, 0.3 - CONTACT_SKIN),
        ],
        wheel_radius: 0.7 * scale,
        ..VehicleControllerParameters::empty()
    };
    truck_controller_parameters
        .wheel_positions
        .iter_mut()
        .for_each(|w| {
            *w *= scale;
        });
    let truck_def = TruckDefHandle(resources.truck_def.clone());
    vehicle_spawner::spawn(
        VehicleType::Truck,
        &mut commands,
        &asset_server,
        Transform::from_translation(spawn_point - Vec3::new(10.0, 0.0, 0.0))
            .with_rotation(Quat::from_rotation_z(180f32.to_radians()))
            .with_scale(Vec3::splat(scale)),
    )
    .insert(truck_controller_parameters)
    .insert(truck_def)
    .insert(TruckControls::default())
    .with_children(|child_builder| {
        // muck pile in the truck
        child_builder.spawn(
            SpawnMuckPileCommand {
                local_aabb: bevy::render::primitives::Aabb {
                    center: Vec3A::new(0.0, 0.0, 1.5),
                    half_extents: Vec3A::new(3.0, 3.0, 2.0),
                },
                name: "Truck pile".to_string(),
                position: Isometry3d::default(),
            }
            .to_bundle_minimal(),
        );
    });

    // Muck piles
    // FIXME: This is not really useful anymore as `CountRocksInZone` doesn't support wgsparkl.
    //        But this serves as a visual objective.
    commands.queue(SpawnMuckPileCommand {
        local_aabb: bevy::render::primitives::Aabb {
            center: Vec3A::new(0.0, 0.0, 0.5),
            half_extents: Vec3A::new(15.0, 15.0, 15.0),
        },
        name: "Muck Pile".to_string(),
        position: Isometry3d::from_translation(spawn_point + Vec3::new(-60.0, 0.0, 0.01 - 6.0)),
    });

    // Camera, light
    let diffuse_map = resources.diffuse_map.clone();
    commands.spawn((
        Camera3d::default(),
        EditorCam {
            orbit_constraint: OrbitConstraint::Fixed {
                up: Vec3::Z,
                can_pass_tdc: false,
            },
            ..EditorCam::default()
        },
        EnvironmentMapLight {
            diffuse_map: diffuse_map.clone(),
            specular_map: resources.specular_map.clone(),
            intensity: 500.0,
            ..default()
        },
        bevy_editor_cam::extensions::independent_skybox::IndependentSkybox::new(
            diffuse_map,
            1000.0,
            default(),
        ),
        Transform::from_translation(spawn_point + Vec3::new(-63.0, 15.0, 58.0))
            .looking_at(spawn_point + Vec3::new(0.0, 10.0, 0.3), Vec3::Z),
    ));
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(1.0, -1.0, -1.0), Vec3::Z),
    ));
}

pub fn add_muck_pile_for_excavator(
    mut commands: Commands,
    excavator_mapping: Query<(&Name, &ExcavatorControlsMapping), Added<ExcavatorControlsMapping>>,
) {
    for (name, mapping) in excavator_mapping.iter() {
        commands
            .spawn((
                SpawnMuckPileCommand {
                    local_aabb: bevy::render::primitives::Aabb {
                        // 300 because excavator has been rescaled..
                        center: Vec3A::new(0.0, 0.0, 300.0),
                        half_extents: Vec3A::new(0.75, 0.75, 0.75),
                    },
                    name: format!("{} pile", name.as_str()),
                    position: Isometry3d::default(),
                }
                .to_bundle_minimal(),
                NoFrustumCulling,
            ))
            .set_parent(mapping.bucket_base);
    }
}
