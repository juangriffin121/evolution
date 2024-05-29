use plotters::prelude::*;
use rand::Rng;
use std::usize;

use super::{
    blobs::{self, Blob, BlobType},
    brains::Brain,
    constants::Constants,
};
use rayon::prelude::*;
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
            let (speed, angle) = (response[0], response[1]);
            blob.angle = angle;
            blob.step(speed, &self.shape, self.constants.step_size);
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
            if distance < 2.0 {
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
        let mut to_remove = Vec::new();
        for (predator_idx, prey_idx) in interactions {
            let predator: &mut Blob = predators[predator_idx];
            let prey: &mut Blob = preys[prey_idx];
            predator.add_energy(self.constants.food_energy * prey.energy);

            //horrible way of writting the prey_idx´th prey position in the blobs array
            to_remove.push(prey_indexes[prey_idx]);
        }
        //kill the prey to remove
        for idx in to_remove {
            self.blobs.remove(idx).die();
        }
    }

    fn reproduce_blobs(&mut self) {
        let mut to_reproduce = Vec::new();
        for (blob_idx, blob) in self.blobs.iter().enumerate() {
            if blob.energy >= 2.0 {
                to_reproduce.push(blob_idx);
            }
        }
        for blob_idx in to_reproduce {
            let blob = self.blobs.remove(blob_idx);
            let (child1, child2) = blob.reproduce(self.constants.reproduction_distance);
            self.blobs.push(child1);
            self.blobs.push(child2);
        }
    }

    fn graph(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
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
                1.0 * (1366.0 / self.shape.0).round(),
                color,
            )))?;
        }

        // Save the result to file
        drawing_area.present()?;
        Ok(())
    }

    pub fn actualize(&mut self, age: i32) {
        let (predator_indexes, prey_indexes) = self.get_indexes();

        let stimuli_list = self.gather_stimuli();

        let responses = self.gather_responses(stimuli_list);

        self.move_blobs(responses);

        let interactions = self.check_interactions(&predator_indexes, &prey_indexes);

        self.kills(interactions, &prey_indexes);

        self.reproduce_blobs();

        let filename = format!("./animation/frame{:04}.png", age);
        println!("{filename}");
        self.graph(&filename).expect("something wong with graphing")
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
