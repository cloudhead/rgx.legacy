#![deny(clippy::all)]
extern crate ultraviolet;

pub mod color;
pub mod error;
pub mod kit;
pub mod math;
pub mod rect;

#[cfg(feature = "renderer")]
pub mod core;
#[cfg(feature = "renderer")]
pub use wgpu;
