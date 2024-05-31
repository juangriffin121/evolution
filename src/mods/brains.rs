use crate::mods::utils::{matrix_prod, sum_weights};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::utils::sigmoid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Brain {
    pub network_shape: Vec<i32>,
    pub neuron_separation_radians: f32,
    pub weights: Vec<Vec<Vec<f32>>>,
    pub neuron_angles: Vec<f32>,
    pub neuron_length: f32,
}

impl Brain {
    pub fn new(
        network_shape: Vec<i32>,
        neuron_separation_radians: f32,
        neuron_length: f32,
        weights: Option<Vec<Vec<Vec<f32>>>>,
    ) -> Brain {
        let weights = weights.unwrap_or(Self::init_random(&network_shape));
        let mut neuron_angles = Vec::new();
        let input_neurons = network_shape[0];
        for i in 0..input_neurons {
            neuron_angles.push((i - input_neurons / 2) as f32 * neuron_separation_radians)
        }

        Brain {
            network_shape,
            neuron_separation_radians,
            weights,
            neuron_angles,
            neuron_length,
        }
    }

    pub fn init_random(network_shape: &Vec<i32>) -> Vec<Vec<Vec<f32>>> {
        let mut weights = Vec::new();
        let mut rng = rand::thread_rng();
        for layer in 0..network_shape.len() - 1 {
            let mut weight_matrix = Vec::new();
            for _post_synaptic_neuron in 0..network_shape[layer + 1] {
                let mut weight_vector = Vec::new();
                for _pre_synaptic_neuron in 0..network_shape[layer] {
                    weight_vector.push(rng.gen_range(-100.0..100.0));
                }
                weight_matrix.push(weight_vector);
            }
            weights.push(weight_matrix);
        }
        weights
    }

    pub fn delta(network_shape: &Vec<i32>, mutation_rate: f32) -> Vec<Vec<Vec<f32>>> {
        let mut weights = Vec::new();
        let mut rng = rand::thread_rng();
        for layer in 0..network_shape.len() - 1 {
            let mut weight_matrix = Vec::new();
            for _post_synaptic_neuron in 0..network_shape[layer + 1] {
                let mut weight_vector = Vec::new();
                for _pre_synaptic_neuron in 0..network_shape[layer] {
                    weight_vector.push(mutation_rate * rng.gen_range(-1.0..1.0));
                }
                weight_matrix.push(weight_vector);
            }
            weights.push(weight_matrix);
        }
        weights
    }

    pub fn make_child(&self, mutation_rate: f32) -> Brain {
        let weights = sum_weights(
            &self.weights,
            &Brain::delta(&self.network_shape, mutation_rate),
        );
        Brain::new(
            self.network_shape.clone(),
            self.neuron_separation_radians,
            self.neuron_length,
            Some(weights),
        )
    }

    pub fn synapse(&self, stimuli: &Vec<f32>) -> Vec<f32> {
        let mut input = stimuli.clone();
        for layer in 0..self.network_shape.len() - 1 {
            input = matrix_prod(&self.weights[layer], &input);
            input = input.iter().map(|&x| sigmoid(x)).collect();
        }
        input
    }
}
