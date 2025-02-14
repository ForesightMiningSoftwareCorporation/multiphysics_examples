use std::path::Path;

use bevy_math::Vec3;
use broken_rocks::load_broken_rocks;
use shared_map::map_def::RockData;
use unbroken_rocks::{load_unbroken_rocks, RecordUnBrokenRock};

pub mod broken_rocks;
pub mod seb_data;
pub mod unbroken_rocks;

pub fn load_all_rocks(
    unbroken_rocks_path: impl AsRef<Path>,
    broken_rocks_path: impl AsRef<Path>,
) -> (Vec<RockData>, Vec<RecordUnBrokenRock>) {
    // load broken rocks
    let broken_rocks = load_broken_rocks(broken_rocks_path).expect("Could not load broken rocks.");
    let mut rocks_for_mapdef = broken_rocks
        .iter()
        .map(|rock| RockData {
            translation: Vec3::new(rock.x, rock.y, rock.z),
            metadata: rock.id,
        })
        .collect::<Vec<_>>();

    // load unbroken rocks
    let mut unbroken_rocks =
        load_unbroken_rocks(unbroken_rocks_path).expect("Could not load unbroken rocks.");

    // center everything depending on heightmap
    let min_max_bounds = get_min_max_bounds(&unbroken_rocks);
    unbroken_rocks.iter_mut().for_each(|rock| {
        rock.z -= min_max_bounds.0.z;
    });
    rocks_for_mapdef.iter_mut().for_each(|rock| {
        rock.translation -= min_max_bounds.0;
    });
    (rocks_for_mapdef, unbroken_rocks)
}

fn get_min_max_bounds(unbroken_rocks: &Vec<RecordUnBrokenRock>) -> (Vec3, Vec3) {
    let (min_coords, max_coords) = unbroken_rocks.iter().fold(
        (
            Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        ),
        |(min_coords, max_coords), rock| {
            let translation = Vec3::new(rock.x, rock.y, rock.z);
            (min_coords.min(translation), max_coords.max(translation))
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
