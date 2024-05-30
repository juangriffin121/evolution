mod mods;

use crate::mods::blobs::BlobType;
use mods::constants::load_constants;
use mods::world::make_world;

fn main() {
    let constants = load_constants();
    println!("{constants:?}");
    let network_shape = vec![constants.input_neurons_num, 10, 3];
    let mut world = make_world(
        constants.num_prey,
        constants.num_predators,
        network_shape,
        constants,
    );
    for age in 0..world.constants.ages {
        world.update(age);
        let blobs_count = world.blobs.len();
        let predators = world
            .blobs
            .iter()
            .filter(|blob| blob.blob_type == BlobType::Predator)
            .count();
        let preys = world
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
