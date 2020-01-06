#![deny(clippy::all)]

pub mod color;
pub mod error;
pub mod kit;
pub mod math;
pub mod rect;

#[cfg(feature = "renderer")]
pub mod core;
#[cfg(feature = "renderer")]
pub use wgpu;
