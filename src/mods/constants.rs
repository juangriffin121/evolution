use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Constants {
    pub reproduction_distance: f32,
    pub step_size: f32,
    pub food_energy: f32,
    pub neuron_length: f32,
    pub world_shape: (f32, f32),
    pub input_neurons_num: i32,
    pub motion_energy_cost: f32,
    pub prey_base_energy_gain: f32,
    pub predator_base_energy_loss: f32,
    pub mutation_rate: f32,
    pub ages: i32,
    pub num_predators: i32,
    pub num_prey: i32,
    pub max_speed: f32,
    pub max_angle_diff: f32,
    pub graph_neurons: bool,
    pub activation: String,
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
