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

extern crate num_traits;

mod efloat32;
pub use self::efloat32::*;
