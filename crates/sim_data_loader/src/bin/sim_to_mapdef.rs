use std::{env, fs};

use bevy_math::Vec3;
use ron::ser::PrettyConfig;
use shared_map::map_def::{MapDef, RockData};
use sim_data_loader::load_broken_rocks;

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
    let broken_rocks = load_broken_rocks(broken_rocks_path).expect("Could not load broken rocks.");
    let rocks_for_mapdef = broken_rocks
        .iter()
        .map(|rock| RockData {
            translation: Vec3::new(rock.x, rock.y, rock.z),
            metadata: rock.id,
        })
        .collect::<Vec<_>>();

    let rocks_for_mapdef = center_rocks(rocks_for_mapdef);

    if let Ok(mut existing_output) =
        ron::de::from_reader::<_, MapDef>(fs::File::open(&output_path).unwrap())
    {
        existing_output.rocks = rocks_for_mapdef;
        ron::ser::to_writer_pretty(
            fs::File::create(&output_path).unwrap(),
            &existing_output,
            PrettyConfig::default(),
        )
        .unwrap();
    }
}

/// Center the rocks translation, mostly to avoid floating point errors.
/// Sets the new minimum Z to be ~0.
// TODO: pass an AABB containing unbroken rocks.
fn center_rocks(rocks_for_mapdef: Vec<RockData>) -> Vec<RockData> {
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
