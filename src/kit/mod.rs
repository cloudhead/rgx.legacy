#![allow(dead_code)]
pub use crate::core;
pub use crate::core::{Bgra8, Rgba, Rgba8};

pub mod shape2d;
pub mod sprite2d;

use crate::math::{Matrix4, Ortho};

use std::time;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Origin {
    Center,
    BottomLeft,
    TopLeft,
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

///////////////////////////////////////////////////////////////////////////
// Animation
///////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum AnimationState {
    Playing(u64, time::Duration),
    Paused(u64, time::Duration),
    Stopped,
}

#[derive(Clone, Debug)]
pub struct Animation<T> {
    pub state: AnimationState,
    pub delay: time::Duration,
    pub frames: Vec<T>,
}

impl<T> Animation<T> {
    pub fn new(frames: &[T], delay: time::Duration) -> Self
    where
        T: Clone,
    {
        Self {
            state: AnimationState::Playing(0, time::Duration::from_secs(0)),
            delay,
            frames: frames.to_vec(),
        }
    }

    pub fn step(&mut self, delta: time::Duration) {
        if let AnimationState::Playing(_, elapsed) = self.state {
            let elapsed = elapsed + delta;
            let fraction = elapsed.as_micros() / self.delay.as_micros();
            self.state = AnimationState::Playing(fraction as u64, elapsed);
        }
    }

    pub fn pause(&mut self) {
        if let AnimationState::Playing(_, elapsed) = self.state {
            self.state = AnimationState::Paused(0, elapsed);
        }
    }

    pub fn play(&mut self) {
        match self.state {
            AnimationState::Paused(_, elapsed) => self.state = AnimationState::Playing(0, elapsed),
            AnimationState::Stopped => {
                self.state = AnimationState::Playing(0, time::Duration::new(0, 0))
            }
            _ => {}
        }
    }

    pub fn stop(&mut self) {
        self.state = AnimationState::Stopped;
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

    pub fn is_playing(&self) -> bool {
        match self.state {
            AnimationState::Playing(_, _) => true,
            _ => false,
        }
    }

    pub fn elapsed(&self) -> time::Duration {
        match self.state {
            AnimationState::Playing(_, elapsed) => elapsed,
            AnimationState::Paused(_, elapsed) => elapsed,
            AnimationState::Stopped => time::Duration::new(0, 0),
        }
    }

    pub fn cursor(&self) -> u64 {
        let cursor = match self.state {
            AnimationState::Playing(cursor, _) => cursor,
            AnimationState::Paused(cursor, _) => cursor,
            AnimationState::Stopped => 0,
        };
        cursor % self.len() as u64
    }

    pub fn push_frame(&mut self, frame: T) {
        self.frames.push(frame);
    }

    pub fn pop_frame(&mut self) -> Option<T> {
        self.frames.pop()
    }
}

///////////////////////////////////////////////////////////////////////////////

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
pub struct AlignedBuffer {
    // TODO: Make this generic when rust-lang#43408 is fixed.
    data: Matrix4<f32>,
    padding: [u8; AlignedBuffer::PAD],
}

impl AlignedBuffer {
    pub const ALIGNMENT: u64 = 256;
    pub const PAD: usize = Self::ALIGNMENT as usize - std::mem::size_of::<Matrix4<f32>>();

    pub fn new(data: Matrix4<f32>) -> Self {
        Self {
            data,
            padding: [0u8; AlignedBuffer::PAD],
        }
    }
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
            aligned.push(AlignedBuffer::new(*t));
        }
        aligned
    }
}
