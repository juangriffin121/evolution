mod mods;

use clap::Parser;
use mods::cli::{Cli, Commands};
use mods::constants::load_constants;
use mods::world::make_world;

use crate::mods::world::World;

fn main() {
    println!("Hello, blobworld!");
    let cli = Cli::parse();

    let (input_filename, output_filename) = if let Some(command) = &cli.command {
        match command {
            Commands::Save { filename } => (None, Some(filename)),
            Commands::Load { filename } => (Some(filename), None),
            Commands::LoadAndSave {
                input_filename,
                output_filename,
            } => (Some(input_filename), Some(output_filename)),
        }
    } else {
        (None, None)
    };

    let mut world = if let Some(filename) = input_filename {
        println!("loading world");
        let world = World::load_from_file(filename).expect("something wong loading world");
        world
    } else {
        println!("generating world");
        let constants = load_constants();
        let network_shape = vec![constants.input_neurons_num, 10, 2];
        let world = make_world(
            constants.num_prey,
            constants.num_predators,
            network_shape,
            constants,
        );
        world
    };

    world.evolve();

    if let Some(filename) = output_filename {
        println!("saving world");
        world
            .save_to_file(filename)
            .expect("something wong saving world");
    }
}
