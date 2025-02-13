use std::{error::Error, path::Path};

use csv::ReaderBuilder;

#[derive(Debug, serde::Deserialize)]
pub struct RecordBrokenRock {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    // pub insitu_model_guid: String,
    // pub pre_dx: f32,
    // pub pre_dy: f32,
    // pub pre_dz: f32,
    // pub pre_x: f32,
    // pub pre_y: f32,
    // pub pre_z: f32,
    pub id: u32,
}

pub fn load_broken_rocks(path: impl AsRef<Path>) -> Result<Vec<RecordBrokenRock>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new().from_path(path)?;
    let mut records = Vec::new();
    for result in reader.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record: RecordBrokenRock = result?;
        records.push(record);
    }
    Ok(records)
}
