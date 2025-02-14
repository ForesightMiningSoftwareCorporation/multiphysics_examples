use bevy::{asset::RenderAssetUsages, prelude::*, render::mesh::Indices};
use bevy_editor_cam::prelude::*;
use shared_map::map_def::RockData;
use sim_data_loader::{
    broken_rocks::load_broken_rocks,
    load_all_rocks,
    unbroken_rocks::{generate_heightmap, load_unbroken_rocks},
};

pub fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DefaultEditorCamPlugins))
        .add_systems(Startup, setup)
        .add_systems(Update, show_heighthashmap)
        .add_systems(Update, show_rocks)
        .run();
}

#[derive(Resource, Debug)]
pub struct HeightMap {
    pub heightmap: Vec<f32>,
    pub dim: UVec2,
}

#[derive(Resource, Debug)]
pub struct Rocks {
    pub rocks_data: Vec<RockData>,
}

#[derive(Resource, Debug)]
pub struct HeightMapAlt(pub sim_data_loader::seb_data::MapDef);

fn setup(
    mut commands: Commands,
    // to show alternative data format
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(1.0, -1.0, -1.0), Vec3::Z),
    ));
    commands.spawn((
        Camera3d::default(),
        EditorCam {
            orbit_constraint: OrbitConstraint::Fixed {
                up: Vec3::Z,
                can_pass_tdc: false,
            },
            ..EditorCam::default()
        },
        Transform::from_translation(Vec3::new(10.0, 10.0, 10.0)).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    let (broken_rocks, unbroken_rocks) = load_all_rocks(
        "assets/private/Sim data/Unbroken rock.csv",
        "assets/private/Sim data/Broken rock.csv",
    );
    // lower the unbroken rocks by min_y from broken rocks.
    // unbroken_rocks.iter_mut().for_each(|rock| {
    //     rock.x -= 86100.0;
    //     rock.y -= 36100.0;
    //     rock.z -= 800.0;
    // });
    let height_map = generate_heightmap(&unbroken_rocks, 1f32);
    let alternative =
        sim_data_loader::seb_data::to_mapdef_alternative(&[], &height_map.0, &height_map.1);
    commands.insert_resource(HeightMap {
        heightmap: height_map.0,
        dim: height_map.1,
    });

    let mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(
        alternative.floor_idx.into_iter().flatten().collect(),
    ));
    let mut mesh = mesh.with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        alternative
            .floor_vtx
            .iter()
            .map(|pos| [pos.x, pos.y, pos.z])
            .collect::<Vec<_>>(),
    );
    mesh.compute_normals();

    let mesh = meshes.add(mesh);
    commands.spawn((Mesh3d(mesh), MeshMaterial3d(materials.add(Color::WHITE))));

    // broken rocks

    commands.insert_resource(Rocks {
        rocks_data: broken_rocks.clone(),
    });
}

fn show_heighthashmap(
    mut gizmos: Gizmos,
    height_map: Res<HeightMap>,
    input: Res<ButtonInput<KeyCode>>,
) {
    // x lines
    for x in 0..height_map.dim.x - 1 {
        for y in 0..height_map.dim.y {
            let h0 = height_map.heightmap[(x + y * height_map.dim.x) as usize];
            let h1 = height_map.heightmap[(x + 1 + y * height_map.dim.x) as usize];
            let x = x as f32;
            let y = y as f32;
            gizmos.line_gradient(
                Vec3::new(x, y, h0),
                if !input.pressed(KeyCode::Space) {
                    Vec3::new(x + 1f32, y, h1)
                } else {
                    Vec3::new(x, y, 0.0)
                },
                Color::hsl((h0 / 10.0).sin(), (x as f32 / 5f32).sin(), 0.8),
                Color::hsl((h1 / 10.0).sin(), ((x + 1f32) as f32 / 5f32).sin(), 0.8),
            );
        }
    }
    // y lines
    for y in 0..height_map.dim.y - 1 {
        for x in 0..height_map.dim.x {
            let h0 = height_map.heightmap[(x + y * height_map.dim.x) as usize];
            let h1 = height_map.heightmap[(x + (y + 1) * height_map.dim.x) as usize];
            let x = x as f32;
            let y = y as f32;
            gizmos.line_gradient(
                Vec3::new(x, y, h0),
                if !input.pressed(KeyCode::Space) {
                    Vec3::new(x, y + 1f32, h1)
                } else {
                    Vec3::new(x, y, 0.0)
                },
                Color::hsl((h0 / 10.0).sin(), (x as f32 / 5f32).sin(), 0.8),
                Color::hsl((h1 / 10.0).sin(), ((x + 1f32) as f32 / 5f32).sin(), 0.8),
            );
        }
    }
}

fn show_rocks(mut gizmos: Gizmos, rocks: Res<Rocks>) {
    // rocks
    for r in rocks.rocks_data.iter() {
        gizmos.sphere(
            Vec3::new(r.translation.x, r.translation.y, r.translation.z),
            0.5,
            Color::hsl(
                (r.translation.x / 5.0).sin(),
                (r.translation.z / 10f32).sin(),
                0.8,
            ),
        );
    }
}
