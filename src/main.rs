mod mods;

use clap::Parser;
use mods::cli::Cli;
use mods::world::World;

use crate::mods::cli::parse_command;

fn main() {
    println!("Hello, blobworld!");
    let cli = Cli::parse();
    let mut rng = rand::thread_rng();
    let (input_filename, output_filename) = parse_command(cli.command);
    let mut world = World::load_or_start(input_filename, &mut rng);
    world.evolve(&mut rng);
    world.if_save(output_filename);
}
