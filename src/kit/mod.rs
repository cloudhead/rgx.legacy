#![allow(dead_code)]
use crate::core::VertexLayout;

pub use crate::core;
pub use crate::core::{Rgba, Rgba8};

pub mod shape2d;
pub mod sprite2d;

use cgmath::{Matrix4, Ortho};

#[derive(Clone)]
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

///////////////////////////////////////////////////////////////////////////
// Animation
///////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub enum AnimationState {
    Playing(u32, f64),
    Paused(u32, f64),
    Stopped,
}

#[derive(Clone)]
pub struct Animation<T> {
    pub state: AnimationState,
    pub delay: f64,
    pub frames: Vec<T>,
}

impl<T> Animation<T> {
    pub fn new(frames: &[T], delay: f64) -> Self
    where
        T: Clone,
    {
        Self {
            state: AnimationState::Playing(0, 0.0),
            delay,
            frames: frames.to_vec(),
        }
    }

    pub fn step(&mut self, delta: f64) {
        if let AnimationState::Playing(_, elapsed) = self.state {
            let elapsed = elapsed + delta;
            let fraction = elapsed / self.delay;
            let cursor = fraction.floor() as u32 % self.frames.len() as u32;
            self.state = AnimationState::Playing(cursor, elapsed);
        }
    }

    pub fn val(&self) -> T
    where
        T: Copy,
    {
        self.frames[self.cursor() as usize]
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn elapsed(&self) -> f64 {
        match self.state {
            AnimationState::Playing(_, elapsed) => elapsed,
            AnimationState::Paused(_, elapsed) => elapsed,
            AnimationState::Stopped => 0.0,
        }
    }

    pub fn cursor(&self) -> u32 {
        match self.state {
            AnimationState::Playing(cursor, _) => cursor,
            AnimationState::Paused(cursor, _) => cursor,
            AnimationState::Stopped => 0,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// RenderBatch
///////////////////////////////////////////////////////////////////////////////////////////////////

// TODO: We can perhaps use compile-time attributes to mark a struct with labels indicating which
// fields represent position, uvs, &c.

#[allow(dead_code)]
pub struct RenderBatch<T> {
    vertices: Vec<T>,
    size: usize,
}

trait VertexLike<'a> {
    type Data: Default + Into<&'a [u8]>;

    fn new(x: f32, y: f32, u: f32, v: f32, data: Self::Data) -> Self;
    fn layout(&self) -> VertexLayout;
}

pub fn ortho(w: u32, h: u32) -> Matrix4<f32> {
    Ortho::<f32> {
        left: 0.0,
        right: w as f32,
        bottom: h as f32,
        top: 0.0,
        near: -1.0,
        far: 1.0,
    }
    .into()
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
struct AlignedBuffer {
    data: Matrix4<f32>,
    padding: [u8; AlignedBuffer::PAD],
}

impl AlignedBuffer {
    const ALIGNMENT: u64 = 256;
    const PAD: usize = Self::ALIGNMENT as usize - std::mem::size_of::<Matrix4<f32>>();
}

struct Model {
    buf: core::UniformBuffer,
    binding: core::BindingGroup,
    size: usize,
}

impl Model {
    fn new(
        layout: &core::BindingGroupLayout,
        transforms: &[Matrix4<f32>],
        dev: &core::Device,
    ) -> Self {
        let aligned = Self::aligned(transforms);
        let buf = dev.create_uniform_buffer(aligned.as_slice());
        let binding = dev.create_binding_group(&layout, &[&buf]);
        let size = transforms.len();
        Self { buf, binding, size }
    }

    fn aligned(transforms: &[Matrix4<f32>]) -> Vec<AlignedBuffer> {
        let mut aligned = Vec::with_capacity(transforms.len());
        for t in transforms {
            aligned.push(AlignedBuffer {
                data: *t,
                padding: [0u8; AlignedBuffer::PAD],
            });
        }
        aligned
    }
}
