use bevy::{input::keyboard::KeyboardInput, prelude::*, utils::HashMap};
use bevy_editor_cam::prelude::*;
use bevy_math::VectorSpace;
use sim_data_loader::unbroken_rocks::{generate_heightmap, load_unbroken_rocks};

pub fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DefaultEditorCamPlugins))
        .add_systems(Startup, setup)
        .add_systems(Update, show_heighthashmap)
        .run();
}

#[derive(Resource, Debug)]
pub struct HeightMap {
    pub heightmap: Vec<f32>,
    pub dim: UVec2,
}

fn setup(mut commands: Commands) {
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
    let mut unbroken_rocks = load_unbroken_rocks("assets/private/Sim data/Unbroken rock.csv")
        .expect("Could not load unbroken rocks.");
    // lower the unbroken rocks by min_y from broken rocks.
    unbroken_rocks.iter_mut().for_each(|rock| {
        rock.x -= 86100.0;
        rock.y -= 36100.0;
        rock.z -= 800.0;
    });
    let height_hash_map = generate_heightmap(&unbroken_rocks, 2f32);
    commands.insert_resource(HeightMap {
        heightmap: height_hash_map.0,
        dim: height_hash_map.1,
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
