//! # efloat
//!
//! licensed under the MIT License (MIT)
//! Copyright (c) 2018 Michael Dilger
//!
//! This is a floating point type that remembers how far off it might be from the
//! actual precise value, based on it's history.  It keeps and upper and lower error
//! bound internally, and you can check those with function calls.
//!
//! Here are a few tips:
//! * Multiplication and division don't cause too much error.
//! * Addition is ok, but subtraction (or addition of differing signs) has
//!   a terrible error bound.
//! * Operate on small numbers first, working up, so that the larger errors don't
//!   propogate and grow as much.
//!
//! Logic taken from pbrt-v3: https://github.com/mmp/pbrt-v3  (efloat.h class)
//!   by Matt Pharr, Greg Humphreys, and Wenzel Jakob.


use std::ops::{Add, Sub, Mul, Div, Neg};

/// This is a floating point type that remembers how far off it might be from the
/// actual precise value, based on it's history.  It keeps and upper and lower error
/// bound internally, and you can check those with function calls.
#[derive(Debug, Clone)]
pub struct EFloat32 {
    v: f32,
    low: f32,
    high: f32,
    #[cfg(debug_assertions)]
    precise: f64,
}

impl EFloat32 {
    pub fn new(v: f32) -> EFloat32 {
        let ef = EFloat32 {
            v: v,
            low: v,
            high: v,
            #[cfg(debug_assertions)]
            precise: v as f64,
        };
        #[cfg(debug_assertions)] {
            ef.check();
        }
        ef
    }

    pub fn new_with_err(v: f32, err: f32) -> EFloat32 {
        let ef = EFloat32 {
            v: v,
            low: next_f32_down(v - err),
            high: next_f32_up(v + err),
            #[cfg(debug_assertions)]
            precise: v as f64,
        };
        #[cfg(debug_assertions)] {
            ef.check();
        }
        ef
    }

    #[cfg(debug_assertions)]
    pub fn new_with_precise_err(v: f32, p: f64, err: f32) -> EFloat32 {
        let mut ef = Self::new_with_err(v, err);
        ef.precise = p;
        ef.check();
        ef
    }

    #[inline]
    pub fn check(&self) {
        if !self.low.is_infinite() && !self.low.is_nan()
            && !self.high.is_infinite() && !self.high.is_nan()
        {
            assert!(self.low <= self.high);
        }
        #[cfg(debug_assertions)] {
            if !self.v.is_infinite() && !self.v.is_nan() {
                assert!(self.low as f64 <= self.precise);
                assert!(self.precise <= self.high as f64);
            }
        }
    }

    pub fn value(&self) -> f32 {
        self.v
    }

    pub fn upper_bound(&self) -> f32 {
        self.high
    }

    pub fn lower_bound(&self) -> f32 {
        self.low
    }

    pub fn absolute_error(&self) -> f32 {
        self.high - self.low
    }

    #[cfg(debug_assertions)]
    pub fn relative_error(&self) -> f32 {
        ((self.precise - self.v as f64) / self.precise).abs() as f32
    }

    #[cfg(debug_assertions)]
    pub fn precise(&self) -> f64 {
        self.precise
    }

    pub fn sqrt(&self) -> EFloat32 {
        let r = EFloat32 {
            v: self.v.sqrt(),
            low: next_f32_down(self.low.sqrt()),
            high: next_f32_up(self.high.sqrt()),
            #[cfg(debug_assertions)]
            precise: self.precise.sqrt(),
        };
        r.check();
        r
    }

    pub fn abs(&self) -> EFloat32 {
        if self.low >= 0.0 {
            // the entire interval is greater than zero, so we are done.
            return self.clone();
        } else if self.high <= 0.0 {
            // the entire interval is less than zero
            let r = EFloat32 {
                v: -self.v,
                low: -self.high,
                high: -self.low,
                #[cfg(debug_assertions)]
                precise: -self.precise,
            };
            r.check();
            return r;
        } else {
            let r = EFloat32 {
                v: self.v.abs(),
                low: 0.0,
                high: -self.low.max(self.high),
                #[cfg(debug_assertions)]
                precise: self.precise.abs(),
            };
            r.check();
            return r;
        }
    }
}

impl Add for EFloat32 {
    type Output = EFloat32;

    fn add(self, other: EFloat32) -> EFloat32 {
        let r = EFloat32 {
            v: self.v + other.v,
            // Interval arithemetic addition, with the result rounded away from
            // the value r.v in order to be conservative.
            low: next_f32_down(self.low + other.low),
            high: next_f32_up(self.high + other.high),
            #[cfg(debug_assertions)]
            precise: self.precise + other.precise
        };
        r.check();
        r
    }
}

impl Sub for EFloat32 {
    type Output = EFloat32;

    fn sub(self, other: EFloat32) -> EFloat32 {
        let r = EFloat32 {
            v: self.v - other.v,
            low: next_f32_down(self.low - other.high),
            high: next_f32_up(self.high - other.low),
            #[cfg(debug_assertions)]
            precise: self.precise - other.precise
        };
        r.check();
        r
    }
}

impl Mul for EFloat32 {
    type Output = EFloat32;

    fn mul(self, other: EFloat32) -> EFloat32 {
        let prod: [f32; 4] = [
            self.low * other.low,
            self.high * other.low,
            self.low * other.high,
            self.high * other.high
        ];

        let r = EFloat32 {
            v: self.v * other.v,
            low: next_f32_down(
                prod[0].min(prod[1]).min(prod[2].min(prod[3]))),
            high: next_f32_up(
                prod[0].max(prod[1]).max(prod[2].max(prod[3]))),
            #[cfg(debug_assertions)]
            precise: self.precise * other.precise
        };
        r.check();
        r
    }
}

impl Div for EFloat32 {
    type Output = EFloat32;

    fn div(self, other: EFloat32) -> EFloat32 {
        if other.low < 0.0 && other.high > 0.0 {
            // Bah. the interval we are dividing straddles zero, so just
            // return an interval of everything.
            return EFloat32 {
                v: self.v / other.v,
                low: -std::f32::INFINITY,
                high: std::f32::INFINITY,
                #[cfg(debug_assertions)]
                precise: self.precise / other.precise
            };
        }
        let prod: [f32; 4] = [
            self.low / other.low,
            self.high / other.low,
            self.low / other.high,
            self.high / other.high
        ];

        let r = EFloat32 {
            v: self.v / other.v,
            low: next_f32_down(
                prod[0].min(prod[1]).min(prod[2].min(prod[3]))),
            high: next_f32_up(
                prod[0].max(prod[1]).max(prod[2].max(prod[3]))),
            #[cfg(debug_assertions)]
            precise: self.precise / other.precise
        };
        r.check();
        r
    }
}

impl Neg for EFloat32 {
    type Output = EFloat32;

    fn neg(self) -> EFloat32 {
        let r = EFloat32 {
            v: -self.v,
            low: -self.high,
            high: -self.low,
            #[cfg(debug_assertions)]
            precise: -self.precise
        };
        r.check();
        r
    }
}

impl PartialEq for EFloat32 {
    fn eq(&self, other: &EFloat32) -> bool {
        self.v == other.v
    }
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

// Higham (2002, sect 3.1)
//pub const MACHINE_EPSILON: f32 = ::std::f32::EPSILON * 0.5;
//fn gamma(n: i32) -> f32 {
//    (n as f32 * MACHINE_EPSILON) / (1.0 - n as f32 * MACHINE_EPSILON)
//}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        let x = EFloat32::new(0.87234);
        let y = EFloat32::new(0.2348709);
        let z = x * y;
        let w = EFloat32::new(1.0) - z;
        println!("value={} upper={} lower={} absolute_error={} relative_error={} precise={}",
                 w.value(), w.upper_bound(), w.lower_bound(),
                 w.absolute_error(), w.relative_error(), w.precise());
    }
}
