use num_traits::cast::{NumCast, ToPrimitive};
use num_traits::{Float, Num, One, ParseFloatError, Zero};
use std::cmp::Ordering;
use std::num::FpCategory;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

/// This is a floating point type that remembers how far off it might be from the
/// actual precise value, based on it's history.  It keeps and upper and lower error
/// bound internally, and you can check those with function calls.
#[derive(Debug, Clone, Copy)]
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
        #[cfg(debug_assertions)]
        {
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
        #[cfg(debug_assertions)]
        {
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
        if !self.low.is_infinite() && !self.low.is_nan() && !self.high.is_infinite()
            && !self.high.is_nan()
        {
            assert!(self.low <= self.high);
        }
        #[cfg(debug_assertions)]
        {
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
            precise: self.precise + other.precise,
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
            precise: self.precise - other.precise,
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
            self.high * other.high,
        ];

        let r = EFloat32 {
            v: self.v * other.v,
            low: next_f32_down(prod[0].min(prod[1]).min(prod[2].min(prod[3]))),
            high: next_f32_up(prod[0].max(prod[1]).max(prod[2].max(prod[3]))),
            #[cfg(debug_assertions)]
            precise: self.precise * other.precise,
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
                low: -::std::f32::INFINITY,
                high: ::std::f32::INFINITY,
                #[cfg(debug_assertions)]
                precise: self.precise / other.precise,
            };
        }
        let prod: [f32; 4] = [
            self.low / other.low,
            self.high / other.low,
            self.low / other.high,
            self.high / other.high,
        ];

        let r = EFloat32 {
            v: self.v / other.v,
            low: next_f32_down(prod[0].min(prod[1]).min(prod[2].min(prod[3]))),
            high: next_f32_up(prod[0].max(prod[1]).max(prod[2].max(prod[3]))),
            #[cfg(debug_assertions)]
            precise: self.precise / other.precise,
        };
        r.check();
        r
    }
}

impl Rem for EFloat32 {
    type Output = EFloat32;

    fn rem(self, other: EFloat32) -> EFloat32 {
        if other.low < 0.0 && other.high > 0.0 {
            // Bah. the interval we are dividing straddles zero, so just
            // return an interval of everything.
            return EFloat32 {
                v: self.v / other.v,
                low: -::std::f32::INFINITY,
                high: ::std::f32::INFINITY,
                #[cfg(debug_assertions)]
                precise: self.precise / other.precise,
            };
        }
        let prod: [f32; 4] = [
            self.low % other.low,
            self.high % other.low,
            self.low % other.high,
            self.high % other.high,
        ];

        let r = EFloat32 {
            v: self.v % other.v,
            low: next_f32_down(prod[0].min(prod[1]).min(prod[2].min(prod[3]))),
            high: next_f32_up(prod[0].max(prod[1]).max(prod[2].max(prod[3]))),
            #[cfg(debug_assertions)]
            precise: self.precise / other.precise,
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
            precise: -self.precise,
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

impl PartialOrd for EFloat32 {
    fn partial_cmp(&self, other: &EFloat32) -> Option<Ordering> {
        self.v.partial_cmp(&other.v)
    }
}

impl Zero for EFloat32 {
    fn zero() -> EFloat32 {
        EFloat32 {
            v: 0.0,
            low: 0.0,
            high: 0.0,
            #[cfg(debug_assertions)]
            precise: 0.0,
        }
    }

    fn is_zero(&self) -> bool {
        self.low <= 0.0 && self.high >= 0.0
    }
}

impl One for EFloat32 {
    fn one() -> EFloat32 {
        EFloat32 {
            v: 1.0,
            low: 1.0,
            high: 1.0,
            #[cfg(debug_assertions)]
            precise: 1.0,
        }
    }

    fn is_one(&self) -> bool {
        self.low <= 1.0 && self.high >= 1.0
    }
}

impl Num for EFloat32 {
    type FromStrRadixErr = ParseFloatError;

    fn from_str_radix(src: &str, radix: u32) -> Result<EFloat32, ParseFloatError> {
        let f = f32::from_str_radix(src, radix)?;
        Ok(EFloat32 {
            v: f,
            low: f,
            high: f,
            #[cfg(debug_assertions)]
            precise: f as f64,
        })
    }
}

impl ToPrimitive for EFloat32 {
    fn to_i64(&self) -> Option<i64> {
        self.v.to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        self.v.to_u64()
    }

    fn to_isize(&self) -> Option<isize> {
        self.v.to_isize()
    }

    fn to_i8(&self) -> Option<i8> {
        self.v.to_i8()
    }

    fn to_i16(&self) -> Option<i16> {
        self.v.to_i16()
    }

    fn to_i32(&self) -> Option<i32> {
        self.v.to_i32()
    }

    fn to_usize(&self) -> Option<usize> {
        self.v.to_usize()
    }

    fn to_u8(&self) -> Option<u8> {
        self.v.to_u8()
    }

    fn to_u16(&self) -> Option<u16> {
        self.v.to_u16()
    }

    fn to_u32(&self) -> Option<u32> {
        self.v.to_u32()
    }

    fn to_f32(&self) -> Option<f32> {
        self.v.to_f32()
    }

    fn to_f64(&self) -> Option<f64> {
        self.v.to_f64()
    }
}

impl NumCast for EFloat32 {
    #[inline]
    fn from<T: ToPrimitive>(n: T) -> Option<EFloat32> {
        n.to_f32().map(|f| EFloat32 {
            v: f,
            low: f,
            high: f,
            #[cfg(debug_assertions)]
            precise: f as f64,
        })
    }
}

macro_rules! fn_noparams_self {
    ($fn:ident) => {
        fn $fn() -> Self {
            let f = f32::$fn();
            EFloat32 {
                v: f,
                low: f,
                high: f,
                #[cfg(debug_assertions)]
                precise: f as f64,
            }
        }
    };
}
macro_rules! fn_self {
    ($fn:ident, $out:ty) => {
        fn $fn(self) -> $out {
            self.v.$fn()
        }
    };
}
macro_rules! fn_self_self {
    ($fn:ident) => {
        fn $fn(self) -> EFloat32 {
            let r = EFloat32 {
                v: self.v.$fn(),
                low: next_f32_down(self.low.$fn()),
                high: next_f32_up(self.high.$fn()),
                #[cfg(debug_assertions)]
                precise: self.precise.$fn(),
            };
            r.check();
            r
        }
    };
}
macro_rules! fn_self_unimpl {
    ($fn:ident, $out:ty) => {
        fn $fn(self) -> $out {
            unimplemented!()
        }
    };
}

impl Float for EFloat32 {
    fn_noparams_self!(nan);
    fn_noparams_self!(infinity);
    fn_noparams_self!(neg_infinity);
    fn_noparams_self!(neg_zero);
    fn_noparams_self!(min_value);
    fn_noparams_self!(min_positive_value);
    fn_noparams_self!(max_value);

    fn_self!(is_nan, bool);
    fn_self!(is_infinite, bool); // maybe also true if low/high is infinite?
    fn_self!(is_finite, bool); // maybe also true if low/high is finite?
    fn_self!(is_normal, bool); // maybe also true if low/high is normal?
    fn_self!(classify, FpCategory);

    fn_self_self!(floor);
    fn_self_self!(ceil);
    fn_self_self!(round);
    fn_self_self!(trunc);

    fn fract(self) -> EFloat32 {
        let r = if self.low.trunc() != self.high.trunc() {
            // The range straddles an integer. We know that we are within
            // two ranges now. However, we can't represent that, so we
            // have to take the entire [0,1).
            EFloat32 {
                v: self.v.fract(),
                low: 0.0,
                high: next_f32_down(1.0),
                #[cfg(debug_assertions)]
                precise: self.precise.fract(),
            }
        } else {
            EFloat32 {
                v: self.v.fract(),
                low: self.low.fract(),
                high: self.high.fract(),
                #[cfg(debug_assertions)]
                precise: self.precise.fract(),
            }
        };
        r.check();
        r
    }

    fn abs(self) -> EFloat32 {
        let r = EFloat32 {
            v: self.v.abs(),
            low: if self.low < 0.0 && self.high > 0.0 {
                0.0
            } else {
                next_f32_down(self.low.abs().min(self.high.abs()))
            },
            high: next_f32_up(self.low.abs().max(self.high.abs())),
            #[cfg(debug_assertions)]
            precise: self.precise.abs(),
        };
        r.check();
        r
    }

    fn signum(self) -> EFloat32 {
        let r = EFloat32 {
            v: self.v.signum(),
            low: self.low.signum(),
            high: self.high.signum(),
            #[cfg(debug_assertions)]
            precise: self.precise.signum()
        };
        r.check();
        r
    }

    fn is_sign_positive(self) -> bool {
        // we can't give a singular answer for a range, so we just
        // use the 'v' value itself
        self.v.is_sign_positive()
    }

    fn is_sign_negative(self) -> bool {
        // we can't give a singular answer for a range, so we just
        // use the 'v' value itself
        self.v.is_sign_negative()
    }

    fn mul_add(self, a: Self, b: Self) -> Self {
        let prod: [f32; 8] = [
            self.low.mul_add(a.low, b.low),
            self.low.mul_add(a.low, b.high),
            self.low.mul_add(a.high, b.low),
            self.low.mul_add(a.high, b.high),
            self.high.mul_add(a.low, b.low),
            self.high.mul_add(a.low, b.high),
            self.high.mul_add(a.high, b.low),
            self.high.mul_add(a.high, b.high)
        ];
        let cmp = |a: &&f32, b: &&f32| {
            if **a<**b { Ordering::Less }
            else if **a>**b { Ordering::Greater }
            else { Ordering::Equal }
        };
        let r = EFloat32 {
            v: self.v.mul_add(a.v, b.v),
            low: next_f32_down(*prod.iter().min_by(cmp).unwrap()),
            high: next_f32_up(*prod.iter().max_by(cmp).unwrap()),
            #[cfg(debug_assertions)]
            precise: self.precise.mul_add(a.precise, b.precise),
        };
        r.check();
        r
    }

    fn recip(self) -> Self {
        let f = EFloat32 {
            v: self.v.recip(),
            low: next_f32_down(self.low.recip().min(self.high.recip())),
            high: next_f32_up(self.low.recip().max(self.high.recip())),
            #[cfg(debug_assertions)]
            precise: self.precise.recip(),
        };
        f.check();
        f
    }

    fn powi(self, n: i32) -> Self {
        unimplemented!()
    }
    fn powf(self, n: Self) -> Self {
        unimplemented!()
    }
    fn sqrt(self) -> Self {
        unimplemented!()
    }
    fn exp(self) -> Self{
        unimplemented!()
    }
    fn exp2(self) -> Self{
        unimplemented!()
    }
    fn ln(self) -> Self{
        unimplemented!()
    }
    fn log(self, base: Self) -> Self{
        unimplemented!()
    }
    fn log2(self) -> Self{
        unimplemented!()
    }
    fn log10(self) -> Self{
        unimplemented!()
    }
    fn max(self, other: Self) -> Self{
        unimplemented!()
    }
    fn min(self, other: Self) -> Self{
        unimplemented!()
    }
    fn abs_sub(self, other: Self) -> Self{
        unimplemented!()
    }
    fn cbrt(self) -> Self{
        unimplemented!()
    }
    fn hypot(self, other: Self) -> Self{
        unimplemented!()
    }
    fn sin(self) -> Self{
        unimplemented!()
    }
    fn cos(self) -> Self{
        unimplemented!()
    }
    fn tan(self) -> Self{
        unimplemented!()
    }
    fn asin(self) -> Self{
        unimplemented!()
    }
    fn acos(self) -> Self{
        unimplemented!()
    }
    fn atan(self) -> Self{
        unimplemented!()
    }
    fn atan2(self, other: Self) -> Self{
        unimplemented!()
    }
    fn sin_cos(self) -> (Self, Self){
        unimplemented!()
    }
    fn exp_m1(self) -> Self{
        unimplemented!()
    }
    fn ln_1p(self) -> Self{
        unimplemented!()
    }
    fn sinh(self) -> Self{
        unimplemented!()
    }
    fn cosh(self) -> Self{
        unimplemented!()
    }
    fn tanh(self) -> Self{
        unimplemented!()
    }
    fn asinh(self) -> Self{
        unimplemented!()
    }
    fn acosh(self) -> Self{
        unimplemented!()
    }
    fn atanh(self) -> Self{
        unimplemented!()
    }
    fn integer_decode(self) -> (u64, i16, i8) {
        unimplemented!()
    }

    fn epsilon() -> EFloat32 {
        let e = f32::epsilon();
        EFloat32 {
            v: e,
            low: e,
            high: e,
            #[cfg(debug_assertions)]
            precise: f64::epsilon(),
        }
    }

    //fn to_degrees(self) -> Self { ... }
    //fn to_radians(self) -> Self { ... }
}

fn f32_to_bits(f: f32) -> u32 {
    unsafe { ::std::mem::transmute(f) }
}

fn bits_to_f32(u: u32) -> f32 {
    unsafe { ::std::mem::transmute(u) }
}

pub fn next_f32_up(f: f32) -> f32 {
    if f.is_infinite() && f > 0.0 {
        f
    } else if f == -0.0 && f.is_sign_negative() {
        0.0
    } else {
        let mut u = f32_to_bits(f);
        if f >= 0.0 {
            u += 1;
        } else {
            u -= 1;
        }
        bits_to_f32(u)
    }
}

pub fn next_f32_down(f: f32) -> f32 {
    if f.is_infinite() && f < 0.0 {
        f
    } else if f == 0.0 && f.is_sign_positive() {
        -0.0
    } else {
        let mut u = f32_to_bits(f);
        if f <= -0.0 {
            u += 1;
        } else {
            u -= 1;
        }
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
        println!(
            "value={} upper={} lower={} absolute_error={} relative_error={} precise={}",
            w.value(),
            w.upper_bound(),
            w.lower_bound(),
            w.absolute_error(),
            w.relative_error(),
            w.precise()
        );
    }
}
