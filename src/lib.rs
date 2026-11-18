#![doc = include_str!("../README.md")]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(private_interfaces)]
#![deny(clippy::missing_safety_doc)]
#![cfg_attr(feature = "nightly", feature(optimize_attribute, slice_range))]
#![no_std]
extern crate alloc as alloc_crate;

mod extract_if;
mod into_iter;
mod slice_range;
mod vec;
mod zst;

pub use extract_if::*;
pub use into_iter::*;
pub use vec::*;
