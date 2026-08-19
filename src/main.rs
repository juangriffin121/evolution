mod mods;

use clap::Parser;
use mods::cli::Cli;
use mods::world::World;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::mods::cli::parse_command;
use crate::mods::constants::load_constants;

fn main() {
    println!("Hello, blobworld!");
    let cli = Cli::parse();
    let constants = load_constants();

    let mut rng = StdRng::seed_from_u64(constants.seed as u64);
    let (input_filename, output_filename) = parse_command(cli.command);
    let mut world = World::load_or_start(input_filename, &mut rng);
    world.evolve(&mut rng);
    world.if_save(output_filename);
}
