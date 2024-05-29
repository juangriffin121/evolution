use serde::Deserialize;
use serde_json;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Constants {
    pub reproduction_distance: f32,
    pub step_size: f32,
    pub food_energy: f32,
}

impl Constants {
    pub fn from_file(path: &str) -> serde_json::Result<Constants> {
        let data = fs::read_to_string(path).expect("Unable to read file");
        let constants: Constants = serde_json::from_str(&data)?;
        Ok(constants)
    }
}

// Declare the constants variable
pub fn load_constants() -> Constants {
    Constants::from_file("constants.json").expect("Failed to load constants")
}
