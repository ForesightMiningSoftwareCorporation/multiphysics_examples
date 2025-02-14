use bevy_math::prelude::*;
use parry3d::{math::Vector, na::DMatrix};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RockData {
    pub translation: Vec3,
    pub size: Vec3,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MapDef {
    pub rocks: Vec<RockData>,
    pub floor_vtx: Vec<Vec3>,
    pub floor_idx: Vec<[u32; 3]>,
}

pub fn to_mapdef_alternative(
    rocks_for_mapdef: &[shared_map::map_def::RockData],
    height_map: &[f32],
    height_map_dim: &bevy_math::UVec2,
) -> MapDef {
    let height_map = (height_map, height_map_dim);
    let matrix =
        DMatrix::<f32>::from_fn(height_map.1.x as usize, height_map.1.y as usize, |x, y| {
            height_map.0[x + y * height_map.1.x as usize]
        });
    let parry_heightfield = parry3d::shape::HeightField::new(matrix, Vector::new(1.0, 1.0, 1.0));
    let trimesh = parry_heightfield.to_trimesh();

    let data_alternative: MapDef = MapDef {
        rocks: rocks_for_mapdef
            .iter()
            .map(|rock| RockData {
                translation: rock.translation,
                size: Vec3::ONE,
            })
            .collect(),
        floor_vtx: trimesh.0.iter().map(|v| Vec3::new(v.x, v.y, v.z)).collect(),
        floor_idx: trimesh.1,
    };
    data_alternative
}
