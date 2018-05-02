
use std::env;

fn main() {
    let arg1 = env::args().skip(1).next().unwrap();
    let f: f32 = arg1.parse::<f32>().unwrap();

    println!("f32: {}", f);
    println!("Next f32 up: {}", next_f32_up(f));
    println!("Next f32 down: {}", next_f32_down(f));
}

fn f32_to_bits(f: f32) -> u32 {
  unsafe { ::std::mem::transmute(f) }
}

fn bits_to_f32(u: u32) -> f32 {
  unsafe { ::std::mem::transmute(u) }
}

fn next_f32_up(f: f32) -> f32 {
  if f.is_infinite() && f > 0.0 {
    f
  } else if f == -0.0 {
    0.0
  } else {
    let mut u = f32_to_bits(f);
    if f>=0.0 { u+=1; }
    else { u-=1; }
    bits_to_f32(u)
  }
}

fn next_f32_down(f: f32) -> f32 {
  if f.is_infinite() && f < 0.0 {
    f
  } else if f == 0.0 {
    -0.0
  } else {
    let mut u = f32_to_bits(f);
    if f<=-0.0 { u+=1; }
    else { u-=1; }
    bits_to_f32(u)
  }
}
