use bevy_math::prelude::*;
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
