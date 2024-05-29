use crate::mods::brains::Brain;
use crate::mods::world::World;

use super::utils::visual_neuron_activation;
#[derive(Clone)]
pub struct Blob {
    pub brain: Brain,
    pub position: (f32, f32),
    pub angle: f32,
    pub blob_type: BlobType,
    pub energy: f32,
}
#[derive(Clone, Copy, PartialEq)]
pub enum BlobType {
    Prey,
    Predator,
}

impl Blob {
    pub fn new(
        brain: Brain,
        position: (f32, f32),
        angle: f32,
        blob_type: BlobType,
        init_energy: f32,
    ) -> Blob {
        Blob {
            brain,
            position,
            angle,
            blob_type,
            energy: init_energy,
        }
    }

    pub fn direction(&self) -> (f32, f32) {
        (self.angle.cos(), self.angle.sin())
    }

    pub fn reproduce(self, reproduction_distance: f32) -> (Blob, Blob) {
        let direction = self.direction();
        let child1 = Blob {
            brain: self.brain.make_child(),
            position: (
                self.position.0 + reproduction_distance * direction.0,
                self.position.1 + reproduction_distance * direction.1,
            ),
            angle: self.angle,
            blob_type: self.blob_type,
            energy: self.energy / 2.,
        };
        let child2 = Blob {
            brain: self.brain.make_child(),
            position: (
                self.position.0 + reproduction_distance * direction.0,
                self.position.1 + reproduction_distance * direction.1,
            ),
            angle: self.angle,
            blob_type: self.blob_type,
            energy: self.energy / 2.,
        };
        (child1, child2)
    }

    pub fn step(&mut self, speed: f32) {
        let direction = self.direction();
        self.position = (
            self.position.0 + speed * direction.0,
            self.position.1 + speed * direction.1,
        )
    }

    pub fn check_surroundings(&self, world: &World) -> Vec<f32> {
        let preys: Vec<&Blob> = world
            .blobs
            .iter()
            .filter(|&x| x.blob_type == BlobType::Prey)
            .collect();
        let predators: Vec<&Blob> = world
            .blobs
            .iter()
            .filter(|&x| x.blob_type == BlobType::Predator)
            .collect();
        let mut stimuli = Vec::new();
        match self.blob_type {
            BlobType::Predator => {
                for neuron in 0..self.brain.network_shape[0] {
                    let neuron_angle = self.brain.neuron_angles[neuron as usize] + self.angle;
                    let neuron_vec = (neuron_angle.cos(), neuron_angle.sin());
                    let activation = visual_neuron_activation(
                        &preys,
                        &self.position,
                        &neuron_vec,
                        self.brain.neuron_length,
                    );
                    stimuli.push(activation)
                }
            }
            BlobType::Prey => {
                for neuron in 0..self.brain.network_shape[0] {
                    let neuron_angle = self.brain.neuron_angles[neuron as usize] + self.angle;
                    let neuron_vec = (neuron_angle.cos(), neuron_angle.sin());
                    let activation = visual_neuron_activation(
                        &predators,
                        &self.position,
                        &neuron_vec,
                        self.brain.neuron_length,
                    );
                    stimuli.push(activation);
                }
            }
        }
        stimuli
    }

    pub fn add_energy(&mut self, energy: f32) {
        self.energy += energy;
    }

    pub fn die(self) {}
}
