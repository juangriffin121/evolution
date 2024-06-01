use super::activations::Activation;
use crate::mods::utils::{matrix_prod, sum_weights};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Brain {
    pub network_shape: Vec<i32>,
    pub neuron_separation_radians: f32,
    pub weights: Vec<Vec<Vec<f32>>>,
    pub neuron_angles: Vec<f32>,
    pub neuron_length: f32,
    pub activation: Activation,
}

impl Brain {
    pub fn new(
        network_shape: Vec<i32>,
        neuron_separation_radians: f32,
        neuron_length: f32,
        weights: Option<Vec<Vec<Vec<f32>>>>,
        activation: Activation,
        rng: &mut impl Rng,
    ) -> Brain {
        let weights = weights.unwrap_or(Self::init_random(&network_shape, rng));
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
            activation,
        }
    }

    pub fn init_random(network_shape: &Vec<i32>, rng: &mut impl Rng) -> Vec<Vec<Vec<f32>>> {
        let mut weights = Vec::new();
        for layer in 0..network_shape.len() - 1 {
            let mut weight_matrix = Vec::new();
            for _post_synaptic_neuron in 0..network_shape[layer + 1] {
                let mut weight_vector = Vec::new();
                for _pre_synaptic_neuron in 0..network_shape[layer] {
                    weight_vector.push(rng.gen_range(-1.0..1.0));
                }
                weight_matrix.push(weight_vector);
            }
            weights.push(weight_matrix);
        }
        weights
    }

    pub fn delta(
        network_shape: &Vec<i32>,
        mutation_rate: f32,
        rng: &mut impl Rng,
    ) -> Vec<Vec<Vec<f32>>> {
        let mut weights = Vec::new();
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

    pub fn make_child(&self, mutation_rate: f32, rng: &mut impl Rng) -> Brain {
        let weights = sum_weights(
            &self.weights,
            &Brain::delta(&self.network_shape, mutation_rate, rng),
        );
        let new_separation = 0.0_f32.max(
            std::f32::consts::TAU.min(self.neuron_separation_radians + rng.gen_range(-0.01..0.01)),
        );
        Brain::new(
            self.network_shape.clone(),
            new_separation,
            self.neuron_length,
            Some(weights),
            self.activation.clone(),
            rng,
        )
    }

    pub fn synapse(&self, stimuli: &Vec<f32>) -> Vec<f32> {
        let mut input = stimuli.clone();
        for layer in 0..self.network_shape.len() - 1 {
            input = matrix_prod(&self.weights[layer], &input);
            if layer != self.network_shape.len() - 2 {
                input = input.iter().map(|&x| self.activation.apply(x)).collect();
            }
        }
        input
    }
}
