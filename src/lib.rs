#![allow(clippy::needless_range_loop)]
#![allow(clippy::useless_format)]
#![allow(clippy::single_match)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![warn(rust_2018_idioms)]
use std::alloc::System;

#[macro_use]
extern crate log;

pub mod alloc;
pub mod application;
pub mod clock;
pub mod gfx;
pub mod logger;
pub mod math;
pub mod platform;
pub mod timer;
pub mod ui;

pub use application::Application;
pub use gfx::color;
pub use ui::Widget;

pub use math::rect::{Rect, Region};
pub use math::{
    Offset, Origin, Ortho, Point, Point2D, Size, Transform, Transform3D, Vector, Vector2D,
    Vector3D, Vector4D, Zero,
};

/// Program version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[global_allocator]
pub static ALLOCATOR: alloc::Allocator = alloc::Allocator::new(System);
