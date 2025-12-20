#![no_std]
#![allow(non_snake_case)]
#![allow(warnings)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(clippy::all)]

pub mod cmacros;
pub use cmacros::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Prevent linking errors from the standard `test` library when running `cargo 3ds test --lib`.
#[cfg(test)]
extern crate shim_3ds;
