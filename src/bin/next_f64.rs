
use std::env;

fn main() {
    let arg1 = env::args().skip(1).next().unwrap();
    let f: f64 = arg1.parse::<f64>().unwrap();

    println!("f64: {}", f);
    println!("Next f64 up: {}", next_f64_up(f));
    println!("Next f64 down: {}", next_f64_down(f));
}

fn f64_to_bits(f: f64) -> u64 {
  unsafe { ::std::mem::transmute(f) }
}

fn bits_to_f64(u: u64) -> f64 {
  unsafe { ::std::mem::transmute(u) }
}

fn next_f64_up(f: f64) -> f64 {
  if f.is_infinite() && f > 0.0 {
    f
  } else if f == -0.0 {
    0.0
  } else {
    let mut u = f64_to_bits(f);
    if f>=0.0 { u+=1; }
    else { u-=1; }
    bits_to_f64(u)
  }
}

fn next_f64_down(f: f64) -> f64 {
  if f.is_infinite() && f < 0.0 {
    f
  } else if f == 0.0 {
    -0.0
  } else {
    let mut u = f64_to_bits(f);
    if f<=-0.0 { u+=1; }
    else { u-=1; }
    bits_to_f64(u)
  }
}
