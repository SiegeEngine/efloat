
extern crate efloat;

use std::env;

fn main() {
    let arg1 = env::args().skip(1).next().unwrap();
    let f: f32 = arg1.parse::<f32>().unwrap();

    println!("f32: {}", f);
    println!("Next f32 up: {}", efloat::next_f32_up(f));
    println!("Next f32 down: {}", efloat::next_f32_down(f));
}
