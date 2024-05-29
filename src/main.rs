mod mods;
use mods::constants::load_constants;
use mods::world::make_world;

fn main() {
    let constants = load_constants();
    let network_shape = vec![constants.input_neurons_num, 10, 3];
    let mut world = make_world(100, 70, network_shape, constants);
    for age in 0..100 {
        world.actualize(age);
    }
}
