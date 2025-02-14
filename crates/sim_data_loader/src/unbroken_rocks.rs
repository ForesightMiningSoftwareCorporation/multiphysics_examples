use bevy_math::{UVec2, Vec3, Vec3Swizzles};
use core::f32;
use csv::ReaderBuilder;
use std::{error::Error, path::Path};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RecordUnBrokenRock {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
    // pub Unbroken: f32,
    // pub model_guid: String,
    // pub post_x: f32,
    // pub post_y: f32,
    // pub post_z: f32,
    pub id: u32,
}

pub fn load_unbroken_rocks(
    path: impl AsRef<Path>,
) -> Result<Vec<RecordUnBrokenRock>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new().from_path(path)?;
    let mut records = Vec::new();
    for result in reader.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record: RecordUnBrokenRock = result?;
        records.push(record);
    }
    Ok(records)
}

/// Returns the heightmap and it's dimensions.
pub fn generate_heightmap(
    blocks: &Vec<RecordUnBrokenRock>,
    sampling_interval: f32,
) -> (Vec<f32>, UVec2) {
    // Transform blocks into min/max pairs
    let blocks = blocks
        .iter()
        .map(|block| {
            (
                Vec3::new(
                    block.x - block.dx / 2.0,
                    block.y - block.dy / 2.0,
                    block.z - block.dz / 2.0,
                ),
                Vec3::new(
                    block.x + block.dx / 2.0,
                    block.y + block.dy / 2.0,
                    block.z + block.dz / 2.0,
                ),
            )
        })
        .collect::<Vec<_>>();
    // Get the min max of the entire model.
    let (mins, maxs) = blocks.iter().fold(
        (Vec3::INFINITY, Vec3::NEG_INFINITY),
        |(min_acc, max_acc), (block_min, block_max)| {
            (min_acc.min(*block_min), max_acc.max(*block_max))
        },
    );
    let model_extents = maxs - mins;
    let model_origin = mins;
    let dim = (model_extents.xy() / sampling_interval)
        .map(|e| e.ceil())
        .as_uvec2()
        + UVec2::new(1, 1);
    let mut result = vec![mins.z; (dim.x * dim.y) as usize];
    for (aabb_min, aabb_max) in blocks {
        let rel_min = aabb_min - model_origin;
        let rel_max = aabb_max - model_origin;
        let idx_min = (rel_min / sampling_interval).map(|e| e.floor()).as_uvec3();
        let idx_max = (rel_max / sampling_interval).map(|e| e.ceil()).as_uvec3();

        for i in idx_min.x..=idx_max.x {
            for j in idx_min.y..=idx_max.y {
                let h = &mut result[(i + j * dim.x) as usize];
                *h = h.max(aabb_max.z)
            }
        }
    }
    (result, dim)
}
