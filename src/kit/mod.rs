#![allow(dead_code)]
use crate::core;
use crate::core::{Binding, BindingType, Rect, Set, ShaderStage, Texture, VertexLayout};

pub use crate::core::Rgba;

use cgmath::prelude::*;
use cgmath::{Matrix4, Ortho, Vector2};

#[derive(Copy, Clone)]
pub struct Vertex {
    position: Vector2<f32>,
    uv: Vector2<f32>,
    color: Rgba8,
}

#[derive(Copy, Clone)]
struct Rgba8 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Rgba8 {
    const TRANSPARENT: Rgba8 = Rgba8 {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
}

#[derive(Copy, Clone)]
pub struct Uniforms {
    pub ortho: Matrix4<f32>,
    pub transform: Matrix4<f32>,
}

impl Vertex {
    fn new(x: f32, y: f32, u: f32, v: f32, color: Rgba8) -> Vertex {
        Vertex {
            position: Vector2::new(x, y),
            uv: Vector2::new(u, v),
            color,
        }
    }
}

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

impl Texture {
    pub fn rect(&self) -> Rect<f32> {
        Rect {
            x1: 0.0,
            y1: 0.0,
            x2: self.w as f32,
            y2: self.h as f32,
        }
    }
}

impl From<Rgba> for Rgba8 {
    fn from(rgba: Rgba) -> Self {
        Self {
            r: (rgba.r * 255.0).round() as u8,
            g: (rgba.g * 255.0).round() as u8,
            b: (rgba.b * 255.0).round() as u8,
            a: (rgba.a * 255.0).round() as u8,
        }
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

pub struct Pipeline2d {
    pipeline: core::Pipeline,
    bindings: core::BindingGroup,
    buf: core::UniformBuffer,
    ortho: Matrix4<f32>,
}

impl Pipeline2d {
    pub fn binding(
        &self,
        renderer: &core::Renderer,
        texture: &core::Texture,
        sampler: &core::Sampler,
    ) -> core::BindingGroup {
        renderer
            .device
            .create_binding_group(&self.pipeline.layout.sets[1], &[texture, sampler])
    }

    pub fn sprite<'a>(
        &self,
        renderer: &'a core::Renderer,
        texture: &'a core::Texture,
        src: Rect<f32>,
        dst: Rect<f32>,
        color: Rgba,
        rep: Repeat,
    ) -> core::VertexBuffer {
        Sprite::new(texture).build(&renderer, src, dst, color, rep)
    }

    pub fn sprite_batch(&self, w: u32, h: u32) -> SpriteBatch {
        SpriteBatch::new(w, h)
    }
}

impl<'a> core::PipelineLike<'a> for Pipeline2d {
    type PrepareContext = Matrix4<f32>;
    type Uniforms = self::Uniforms;

    fn setup(pipeline: core::Pipeline, dev: &core::Device, w: u32, h: u32) -> Self {
        let ortho = ortho(w, h);
        let transform = Matrix4::identity();
        let buf = dev.create_uniform_buffer(&[self::Uniforms { ortho, transform }]);
        let bindings = dev.create_binding_group(&pipeline.layout.sets[0], &[&buf]);

        Pipeline2d {
            pipeline,
            buf,
            bindings,
            ortho,
        }
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.ortho = ortho(w, h);
    }

    fn apply(&self, pass: &mut core::Pass) {
        pass.apply_pipeline(&self.pipeline);
        pass.apply_binding(&self.bindings, &[0]);
    }

    fn prepare(
        &'a self,
        transform: Matrix4<f32>,
    ) -> Option<(&'a core::UniformBuffer, Vec<self::Uniforms>)> {
        Some((
            &self.buf,
            vec![self::Uniforms {
                transform,
                ortho: self.ortho,
            }],
        ))
    }
}

pub const SPRITE2D: core::PipelineDescription = core::PipelineDescription {
    vertex_layout: &[
        core::VertexFormat::Float2,
        core::VertexFormat::Float2,
        core::VertexFormat::UByte4,
    ],
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
    vertex_shader: include_str!("data/sprite.vert"),
    fragment_shader: include_str!("data/sprite.frag"),
};

pub const FRAMEBUFFER: core::PipelineDescription = core::PipelineDescription {
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
};

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

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Sprite & SpriteBatch
///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Sprite<'a> {
    pub texture: &'a Texture,
}

impl<'a> Sprite<'a> {
    pub fn new(t: &'a Texture) -> Self {
        Self { texture: t }
    }

    pub fn build(
        self,
        renderer: &core::Renderer,
        src: Rect<f32>,
        dst: Rect<f32>,
        color: Rgba,
        rep: Repeat,
    ) -> core::VertexBuffer {
        let (tw, th) = (self.texture.w, self.texture.h);

        // Relative texture coordinates
        let rx1: f32 = src.x1 / tw as f32;
        let ry1: f32 = src.y1 / th as f32;
        let rx2: f32 = src.x2 / tw as f32;
        let ry2: f32 = src.y2 / th as f32;

        let c = color.into();

        // TODO: Use an index buffer
        let verts: Vec<Vertex> = vec![
            Vertex::new(dst.x1, dst.y1, rx1 * rep.x, ry2 * rep.y, c),
            Vertex::new(dst.x2, dst.y1, rx2 * rep.x, ry2 * rep.y, c),
            Vertex::new(dst.x2, dst.y2, rx2 * rep.x, ry1 * rep.y, c),
            Vertex::new(dst.x1, dst.y1, rx1 * rep.x, ry2 * rep.y, c),
            Vertex::new(dst.x1, dst.y2, rx1 * rep.x, ry1 * rep.y, c),
            Vertex::new(dst.x2, dst.y2, rx2 * rep.x, ry1 * rep.y, c),
        ];

        renderer.device.create_buffer(verts.as_slice())
    }
}

pub struct SpriteBatch {
    pub w: u32,
    pub h: u32,
    pub vertices: Vec<Vertex>,
    pub size: usize,
}

impl SpriteBatch {
    fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            vertices: Vec::with_capacity(6),
            size: 0,
        }
    }

    pub fn add(&mut self, src: Rect<f32>, dst: Rect<f32>, rgba: Rgba, rep: Repeat) {
        let c: Rgba8 = rgba.into();
        let (tw, th) = (self.w, self.h);

        // Relative texture coordinates
        let rx1: f32 = src.x1 / tw as f32;
        let ry1: f32 = src.y1 / th as f32;
        let rx2: f32 = src.x2 / tw as f32;
        let ry2: f32 = src.y2 / th as f32;

        // TODO: Use an index buffer
        let mut verts: Vec<Vertex> = vec![
            Vertex::new(dst.x1, dst.y1, rx1 * rep.x, ry2 * rep.y, c),
            Vertex::new(dst.x2, dst.y1, rx2 * rep.x, ry2 * rep.y, c),
            Vertex::new(dst.x2, dst.y2, rx2 * rep.x, ry1 * rep.y, c),
            Vertex::new(dst.x1, dst.y1, rx1 * rep.x, ry2 * rep.y, c),
            Vertex::new(dst.x1, dst.y2, rx1 * rep.x, ry1 * rep.y, c),
            Vertex::new(dst.x2, dst.y2, rx2 * rep.x, ry1 * rep.y, c),
        ];

        self.vertices.append(&mut verts);
        self.size += 1;
    }

    pub fn finish(self, r: &core::Renderer) -> core::VertexBuffer {
        r.device.create_buffer(self.vertices.as_slice())
    }
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
