#![deny(clippy::all)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::should_implement_trait)]

pub mod color;
pub mod error;
pub mod kit;
pub mod math;
pub mod rect;

#[cfg(feature = "renderer")]
pub mod core;
#[cfg(feature = "renderer")]
pub use wgpu;
