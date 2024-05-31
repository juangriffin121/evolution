use plotters::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, usize};

use super::{
    blobs::{Blob, BlobType},
    brains::Brain,
    constants::Constants,
};
use bincode;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct World {
    pub blobs: Vec<Blob>,
    pub shape: (f32, f32),
    pub constants: Constants,
}

impl World {
    pub fn new(blobs: Vec<Blob>, constants: Constants) -> World {
        World {
            blobs,
            shape: constants.world_shape,
            constants,
        }
    }
    fn get_indexes(&self) -> (Vec<usize>, Vec<usize>) {
        let prey_indexes = self
            .blobs
            .iter()
            .enumerate()
            .filter(|(_, blob)| blob.blob_type == BlobType::Prey)
            .map(|(i, _)| i)
            .collect();
        let predator_indexes = self
            .blobs
            .iter()
            .enumerate()
            .filter(|(_, blob)| blob.blob_type == BlobType::Predator)
            .map(|(i, _)| i)
            .collect();
        (predator_indexes, prey_indexes)
    }
    fn gather_stimuli(&self) -> Vec<Vec<f32>> {
        // Use par_iter to iterate over blobs in parallel, apply check_surroundings, and collect results
        let stimuli_list: Vec<(usize, Vec<f32>)> = self
            .blobs
            .par_iter()
            .enumerate()
            .map(|(i, blob)| (i, blob.check_surroundings(self)))
            .collect();
        let mut sorted_stimuli_list = stimuli_list;
        sorted_stimuli_list.sort_by_key(|(i, _)| *i);
        sorted_stimuli_list
            .into_iter()
            .map(|(_, stimuli)| stimuli)
            .collect()
    }

    fn gather_responses(&self, stimuli_list: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
        let output_list: Vec<(usize, Vec<f32>)> = self
            .blobs
            .par_iter()
            .enumerate()
            .map(|(i, blob)| (i, blob.brain.synapse(&stimuli_list[i])))
            .collect();
        let mut sorted_output_list = output_list;
        sorted_output_list.sort_by_key(|(i, _)| *i);
        sorted_output_list
            .into_iter()
            .map(|(_, stimuli)| stimuli)
            .collect()
    }

    fn move_blobs(&mut self, responses: Vec<Vec<f32>>) {
        self.blobs.par_iter_mut().enumerate().for_each(|(i, blob)| {
            let response = &responses[i];

            let (speed, angle_diff) = (
                self.constants.max_speed * response[0],
                2.0 * self.constants.max_angle_diff * (response[1] - 0.5),
            );
            blob.angle += angle_diff;
            blob.step(
                speed,
                &self.shape,
                self.constants.step_size,
                self.constants.motion_energy_cost,
            );
        });
    }

    fn check_pred_interactions(
        (i, predator): (usize, &Blob),
        preys: &Vec<&Blob>,
    ) -> Vec<(usize, usize)> {
        let mut interactions = Vec::new();
        let predator_position = predator.position;
        for (j, prey) in preys.iter().enumerate() {
            let distance = (predator_position.0 - prey.position.0).powi(2)
                + (predator_position.1 - prey.position.1).powi(2);
            if distance <= prey.radius() + predator.radius() {
                interactions.push((i, j))
            }
        }
        interactions
    }

    fn check_interactions(
        &self,
        predator_indexes: &Vec<usize>,
        prey_indexes: &Vec<usize>,
    ) -> Vec<(usize, usize)> {
        let preys: Vec<&Blob> = prey_indexes.iter().map(|&i| &self.blobs[i]).collect();
        let predators: Vec<&Blob> = predator_indexes.iter().map(|&i| &self.blobs[i]).collect();
        let interactions: Vec<(usize, usize)> = predators
            .par_iter()
            .enumerate()
            .map(|(i, predator)| Self::check_pred_interactions((i, predator), &preys))
            .flatten()
            .collect();
        interactions
    }

    fn kills(&mut self, interactions: Vec<(usize, usize)>, prey_indexes: &Vec<usize>) {
        //get vecs of mutable refferences to the preys and predators
        let mut preys = Vec::new();
        let mut predators = Vec::new();
        for blob in self.blobs.iter_mut() {
            match blob.blob_type {
                BlobType::Prey => preys.push(blob),
                BlobType::Predator => predators.push(blob),
            }
        }

        //run through the interactions and feed the predators and mark the prey to kill
        let mut to_remove = HashSet::new();
        for (predator_idx, prey_idx) in interactions {
            let predator: &mut Blob = predators[predator_idx];
            let prey: &mut Blob = preys[prey_idx];
            predator.add_energy(self.constants.food_energy * prey.energy);

            //horrible way of writting the prey_idx´th prey position in the blobs array
            to_remove.insert(prey_indexes[prey_idx]);
        }

        let mut to_remove: Vec<_> = to_remove.into_iter().collect();
        //sort the removing indexes so i dont fuck up the blobs indexes while looping over it
        to_remove.sort_unstable_by(|a, b| b.cmp(a));
        //kill the prey to remove
        for idx in &to_remove {
            if *idx >= self.blobs.len() {
                println!("to remove: {:?}", to_remove)
            }
            self.blobs.remove(*idx).die();
        }
    }

    fn reproduce_blobs(&mut self) {
        let mut to_reproduce = Vec::new();
        for (blob_idx, blob) in self.blobs.iter().enumerate() {
            if blob.energy >= 2.0 {
                to_reproduce.push(blob_idx);
            }
        }
        for blob_idx in to_reproduce.iter().rev() {
            let blob = self.blobs.remove(*blob_idx);
            let (child1, child2) = blob.reproduce(
                self.constants.reproduction_distance,
                self.constants.mutation_rate,
            );
            self.blobs.push(child1);
            self.blobs.push(child2);
        }
    }

    fn base_energy(&mut self) {
        self.blobs.par_iter_mut().for_each(|blob| {
            let energy = match blob.blob_type {
                BlobType::Prey => self.constants.prey_base_energy_gain,
                BlobType::Predator => -self.constants.predator_base_energy_loss,
            };
            blob.add_energy(energy);
        })
    }

    fn starved(&mut self) {
        let mut starved_blobs_idxs = Vec::new();
        for (i, blob) in self.blobs.iter().enumerate() {
            if blob.energy < 0.0 {
                starved_blobs_idxs.push(i);
            }
        }
        for i in starved_blobs_idxs.iter().rev() {
            self.blobs.remove(*i).die();
        }
    }

    fn graph(&self, filename: &str, graph_neurons: bool) -> Result<(), Box<dyn std::error::Error>> {
        let screen_shape = (1366, 768);
        let drawing_area = BitMapBackend::new(filename, screen_shape).into_drawing_area();

        // Fill the background with white color
        drawing_area.fill(&WHITE)?;

        // Define the coordinate system
        let mut chart = ChartBuilder::on(&drawing_area)
            .build_cartesian_2d(-0.0_f32..self.shape.0, 0.0_f32..self.shape.1)?;

        // Draw the mesh (the grid lines and labels)
        chart.configure_mesh().draw()?;

        // Draw the circles
        for blob in &self.blobs {
            let color = match blob.blob_type {
                BlobType::Prey => GREEN.mix(0.9).filled(),
                BlobType::Predator => RED.mix(0.9).filled(),
            };
            chart.draw_series(std::iter::once(Circle::new(
                blob.position,
                blob.radius() * (1366.0 / self.shape.0).round(),
                color,
            )))?;

            if graph_neurons {
                let neuron_count = self.constants.input_neurons_num; // Assume `neuron_count` is defined in blob
                let neuron_length = self.constants.neuron_length; // Assume `neuron_length` is defined in blob
                let blob_angle = blob.angle; // Assume `direction` is the angle in radians

                let half_neurons = neuron_count / 2;
                let angle_step = blob.brain.neuron_separation_radians;

                for i in 0..neuron_count {
                    let angle = if i < half_neurons {
                        blob_angle - (i as f32 * angle_step)
                    } else {
                        blob_angle + ((i - half_neurons) as f32 * angle_step)
                    };

                    let end_x = blob.position.0 + angle.cos() * neuron_length;
                    let end_y = blob.position.1 + angle.sin() * neuron_length;

                    chart.draw_series(std::iter::once(PathElement::new(
                        vec![blob.position, (end_x, end_y)],
                        &BLACK,
                    )))?;
                }
            }
        }

        // Save the result to file
        drawing_area.present()?;
        Ok(())
    }

    pub fn update(&mut self, age: i32) {
        let (predator_indexes, prey_indexes) = self.get_indexes();

        let stimuli_list = self.gather_stimuli();
        // println!("{stimuli_list:?}");

        let responses = self.gather_responses(stimuli_list);
        // println!("{responses:?}");

        self.move_blobs(responses);

        let interactions = self.check_interactions(&predator_indexes, &prey_indexes);

        self.kills(interactions, &prey_indexes);

        self.base_energy();

        self.starved();

        self.reproduce_blobs();

        let filename = format!("./animation/frame{:04}.png", age);
        println!("{filename}");
        self.graph(&filename, self.constants.graph_neurons)
            .expect("something wong with graphing")
    }

    pub fn evolve(&mut self) {
        for age in 0..self.constants.ages {
            self.update(age);
            let blobs_count = self.blobs.len();
            let predators = self
                .blobs
                .iter()
                .filter(|blob| blob.blob_type == BlobType::Predator)
                .count();
            let preys = self
                .blobs
                .iter()
                .filter(|blob| blob.blob_type == BlobType::Prey)
                .count();
            let log = format!(
                "all: {}, prey: {}, predators: {}",
                blobs_count, preys, predators
            );
            println!("{log}")
        }
    }

    pub fn save_to_file(&self, filename: &str) -> io::Result<()> {
        let encoded: Vec<u8> = bincode::serialize(self).expect("Failed to serialize");
        let mut file = File::create(filename)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    pub fn load_from_file(filename: &str) -> io::Result<World> {
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let world: World = bincode::deserialize(&buffer).expect("Failed to deserialize");
        Ok(world)
    }
}

pub fn make_world(
    num_prey: i32,
    num_predators: i32,
    network_shape: Vec<i32>,
    constants: Constants,
) -> World {
    let mut rng = rand::thread_rng();
    let mut blobs = Vec::new();
    for _ in 0..num_prey {
        let position = (
            rng.gen_range(0.0..constants.world_shape.0),
            rng.gen_range(0.0..constants.world_shape.1),
        );

        let angle = rng.gen_range(0.0..std::f32::consts::TAU);

        let brain = Brain::new(network_shape.clone(), 0.1, constants.neuron_length, None);
        blobs.push(Blob::new(brain, position, angle, BlobType::Prey, 1.0));
    }
    for _ in 0..num_predators {
        let position = (
            rng.gen_range(0.0..constants.world_shape.0),
            rng.gen_range(0.0..constants.world_shape.1),
        );

        let angle = rng.gen_range(0.0..std::f32::consts::TAU);

        let brain = Brain::new(network_shape.clone(), 0.1, constants.neuron_length, None);
        blobs.push(Blob::new(brain, position, angle, BlobType::Predator, 1.0));
    }

    World::new(blobs, constants)
}
