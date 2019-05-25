#![allow(dead_code)]
use crate::core;
use crate::core::{Binding, BindingType, Set, ShaderStage, VertexLayout};

pub use crate::core::Rgba;

pub mod sprite2d;

use cgmath::{Matrix4, Ortho};

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

pub enum AnimationState {
    Playing(u32, f64),
    Paused(u32, f64),
    Stopped,
}

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

pub struct Framebuffer {
    pub target: core::Framebuffer,

    vb: core::VertexBuffer,
}

impl core::Draw for Framebuffer {
    fn draw(&self, binding: &core::BindingGroup, pass: &mut core::Pass) {
        pass.draw(&self.vb, binding);
    }
}

pub struct PipelinePost {
    pipeline: core::Pipeline,
    bindings: core::BindingGroup,
    buf: core::UniformBuffer,
    color: core::Rgba,
    width: u32,
    height: u32,
}

impl PipelinePost {
    pub fn binding(
        &self,
        renderer: &core::Renderer,
        framebuffer: &Framebuffer,
        sampler: &core::Sampler,
    ) -> core::BindingGroup {
        renderer.device.create_binding_group(
            &self.pipeline.layout.sets[1],
            &[&framebuffer.target, sampler],
        )
    }

    pub fn framebuffer(&self, r: &core::Renderer) -> Framebuffer {
        #[derive(Copy, Clone)]
        struct Vertex(f32, f32, f32, f32);

        #[rustfmt::skip]
        let vertices: &[Vertex] = &[
            Vertex(-1.0, -1.0, 0.0, 1.0),
            Vertex( 1.0, -1.0, 1.0, 1.0),
            Vertex( 1.0,  1.0, 1.0, 0.0),
            Vertex(-1.0, -1.0, 0.0, 1.0),
            Vertex(-1.0,  1.0, 0.0, 0.0),
            Vertex( 1.0,  1.0, 1.0, 0.0),
        ];

        Framebuffer {
            target: r.framebuffer(self.width, self.height),
            vb: r.vertexbuffer(vertices),
        }
    }
}

impl<'a> core::PipelineLike<'a> for PipelinePost {
    type PrepareContext = core::Rgba;
    type Uniforms = core::Rgba;

    fn description() -> core::PipelineDescription<'a> {
        core::PipelineDescription {
            vertex_layout: &[core::VertexFormat::Float2, core::VertexFormat::Float2],
            pipeline_layout: &[
                Set(&[Binding {
                    binding: BindingType::UniformBuffer,
                    stage: ShaderStage::Vertex,
                }]),
                Set(&[
                    Binding {
                        binding: BindingType::SampledTexture,
                        stage: ShaderStage::Fragment,
                    },
                    Binding {
                        binding: BindingType::Sampler,
                        stage: ShaderStage::Fragment,
                    },
                ]),
            ],
            // TODO: Use `env("CARGO_MANIFEST_DIR")`
            vertex_shader: include_str!("data/post.vert"),
            fragment_shader: include_str!("data/post.frag"),
        }
    }

    fn setup(pipeline: core::Pipeline, dev: &core::Device, width: u32, height: u32) -> Self {
        let color = core::Rgba::new(0.0, 0.0, 0.0, 0.0);
        let buf = dev.create_uniform_buffer(&[color]);
        let bindings = dev.create_binding_group(&pipeline.layout.sets[0], &[&buf]);

        PipelinePost {
            pipeline,
            buf,
            bindings,
            color,
            width,
            height,
        }
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.width = w;
        self.height = h;
    }

    fn apply(&self, pass: &mut core::Pass) {
        pass.apply_pipeline(&self.pipeline);
        pass.apply_binding(&self.bindings, &[0]);
    }

    fn prepare(&'a self, ctx: core::Rgba) -> Option<(&'a core::UniformBuffer, Vec<core::Rgba>)> {
        Some((&self.buf, vec![ctx]))
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
        bottom: 0.0,
        top: h as f32,
        near: -1.0,
        far: 1.0,
    }
    .into()
}
