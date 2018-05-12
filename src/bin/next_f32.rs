extern crate efloat;

use std::env;

fn main() {
    let arg1 = env::args().skip(1).next().unwrap();

    let f: f32 = arg1.parse::<f32>().unwrap();
    let i: i32 = unsafe { ::std::mem::transmute(f) };

    let up: f32 = efloat::next_f32_up(f);
    let upi: i32 = unsafe { ::std::mem::transmute(up) };

    let down: f32 = efloat::next_f32_down(f);
    let downi: i32 = unsafe { ::std::mem::transmute(down) };

    println!("f32: {} = 0x{:x}", f, i);
    println!("Next f32 up: {} = 0x{:x}", up, upi);
    println!("Next f32 down: {} = 0x{:x}", down, downi);
}
