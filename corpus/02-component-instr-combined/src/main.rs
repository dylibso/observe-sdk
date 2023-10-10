cargo_component_bindings::generate!();

use bindings::component_instr_component::hello_world;

fn main() {
    println!("calling component: \"{}\"", hello_world());
}
