use std::{env, fs};

use bevy_math::Vec3;
use ron::ser::PrettyConfig;
use shared_map::map_def::{MapDef, RockData};
use sim_data_loader::{
    broken_rocks::load_broken_rocks,
    seb_data,
    unbroken_rocks::{generate_heightmap, load_unbroken_rocks},
};

fn main() {
    let mut args = env::args();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} UNBROKEN_ROCKS_FILE BROKEN_ROCKS_FILE OUTPUT_FILE",
            args.next().unwrap()
        );
        std::process::exit(1);
    }
    args.next();
    let unbroken_rocks_path = args.next().unwrap();
    let broken_rocks_path = args.next().unwrap();
    let output_path = args.next().unwrap();
    // load broken rocks
    let broken_rocks = load_broken_rocks(broken_rocks_path).expect("Could not load broken rocks.");
    let rocks_for_mapdef = broken_rocks
        .iter()
        .map(|rock| RockData {
            translation: Vec3::new(rock.x, rock.y, rock.z),
            metadata: rock.id,
        })
        .collect::<Vec<_>>();
    let bounds_broken_rocks = get_min_max_bounds(&rocks_for_mapdef);
    let rocks_for_mapdef = center_rocks(rocks_for_mapdef, bounds_broken_rocks);

    // load unbroken rocks
    let mut unbroken_rocks =
        load_unbroken_rocks(unbroken_rocks_path).expect("Could not load unbroken rocks.");
    // lower the unbroken rocks by min_y from broken rocks.
    unbroken_rocks.iter_mut().for_each(|rock| {
        rock.z -= bounds_broken_rocks.0.z;
    });
    let sampling = 1f32;
    let height_map = generate_heightmap(&unbroken_rocks, sampling);

    //let rocks_for_mapdef = center_rocks(rocks_for_mapdef);

    // write the broken rocks

    let mut existing_output = if let Ok(mut existing_output) =
        ron::de::from_reader::<_, MapDef>(fs::File::open(&output_path).unwrap())
    {
        existing_output
    } else {
        MapDef::default()
    };
    existing_output.rocks = rocks_for_mapdef.clone();
    //existing_output.rocks = vec![];
    existing_output.height_map = height_map.0.clone();
    existing_output.scale = Vec3::new(
        height_map.1.x as f32 / sampling,
        1.0,
        height_map.1.y as f32 / sampling,
    );
    existing_output.vertices_width = height_map.1.x as usize;
    existing_output.vertices_length = height_map.1.y as usize;

    ron::ser::to_writer_pretty(
        fs::File::create(&output_path).unwrap(),
        &existing_output,
        PrettyConfig::default(),
    )
    .unwrap();
    let mut output_path_alternative = output_path.clone();
    output_path_alternative.push_str(".seb.ron");

    let data_alternative =
        seb_data::to_mapdef_alternative(&rocks_for_mapdef, &height_map.0, &height_map.1);

    ron::ser::to_writer_pretty(
        fs::File::create(&output_path_alternative).unwrap(),
        &data_alternative,
        PrettyConfig::default(),
    )
    .unwrap();
}

fn get_min_max_bounds(rocks_for_mapdef: &Vec<RockData>) -> (Vec3, Vec3) {
    let (min_coords, max_coords) = rocks_for_mapdef.iter().fold(
        (
            Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        ),
        |(min_coords, max_coords), rock| {
            (
                min_coords.min(rock.translation),
                max_coords.max(rock.translation),
            )
        },
    );
    (min_coords, max_coords)
}

/// Center the rocks translation, mostly to avoid floating point errors.
/// Sets the new minimum Z to be ~0.
// TODO: pass an AABB containing unbroken rocks, as both rocks and unbroken rocks should be centered together.
fn center_rocks(
    rocks_for_mapdef: Vec<RockData>,
    (min_coords, max_coords): (Vec3, Vec3),
) -> Vec<RockData> {
    let center = (min_coords + max_coords) / 2.0;
    let rocks_for_mapdef = rocks_for_mapdef
        .iter()
        .map(|rock| RockData {
            translation: Vec3 {
                x: rock.translation.x - center.x,
                y: rock.translation.y - center.y,
                z: rock.translation.z - min_coords.z,
            },
            metadata: rock.metadata,
        })
        .collect::<Vec<_>>();
    rocks_for_mapdef
}
