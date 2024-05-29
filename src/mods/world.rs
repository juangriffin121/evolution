use std::usize;

use super::{
    blobs::{Blob, BlobType},
    constants::Constants,
};
use rayon::prelude::*;
pub struct World {
    pub blobs: Vec<Blob>,
    pub shape: (f32, f32),
    pub constants: Constants,
}

impl World {
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
            blob.step(speed);
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

    pub fn actualize(&mut self) {
        //get list of predator and prey positions in the blobs vec
        let (predator_indexes, prey_indexes) = self.get_indexes();

        let stimuli_list = self.gather_stimuli();

        let responses = self.gather_responses(stimuli_list);

        self.move_blobs(responses);

        let interactions = self.check_interactions(&predator_indexes, &prey_indexes);

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

        //reproduce
        //graph
    }
}
