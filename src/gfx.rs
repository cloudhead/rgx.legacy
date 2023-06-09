#![allow(clippy::many_single_char_names)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::too_many_arguments)]

pub mod backends;
pub mod color;
pub mod pixels;
pub mod renderer;
pub mod shape2d;
pub mod sprite2d;

pub mod prelude {
    use super::*;

    pub use super::{Axis, Repeat, ZDepth};
    pub use color::{Color, Image, Rgb8, Rgba, Rgba8};
    pub use renderer::*;
    pub use shape2d::{
        circle, line, rectangle, Fill, IntoShape, Rectangle, Rotation, Shape, Stroke,
    };
}

pub use prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Repeat {
    pub x: f32,
    pub y: f32,
}

impl Repeat {
    pub fn new(x: f32, y: f32) -> Self {
        Repeat { x, y }
    }
}

impl Default for Repeat {
    fn default() -> Self {
        Repeat { x: 1.0, y: 1.0 }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub struct ZDepth(pub f32);

impl ZDepth {
    pub const ZERO: Self = ZDepth(0.0);
}

impl From<f32> for ZDepth {
    fn from(other: f32) -> Self {
        ZDepth(other)
    }
}

impl Default for ZDepth {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Deref for ZDepth {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
