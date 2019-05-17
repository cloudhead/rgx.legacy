#![allow(dead_code)]
use crate::core;
use crate::core::{Binding, BindingType, Sampler, Set, ShaderStage, Texture, VertexLayout};

pub use crate::core::Rgba;

use cgmath::prelude::*;
use cgmath::{Matrix4, Ortho, Vector2};

#[derive(Clone)]
struct NonEmpty<T>(T, Vec<T>);

impl<T> NonEmpty<T>
where
    T: Clone,
{
    fn singleton(e: T) -> Self {
        NonEmpty(e, Vec::new())
    }

    fn push(&mut self, e: T) {
        self.1.push(e)
    }

    fn pop(&mut self) -> Option<T> {
        self.1.pop()
    }

    fn len(&self) -> usize {
        self.1.len() + 1
    }

    fn last(&self) -> &T {
        match self.1.last() {
            None => &self.0,
            Some(e) => e,
        }
    }
}

impl<T> Into<Vec<T>> for NonEmpty<T> {
    /// Turns a non-empty list into a Vec.
    fn into(self) -> Vec<T> {
        std::iter::once(self.0).chain(self.1).collect()
    }
}

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

#[derive(Copy, Clone)]
pub struct Rect<T> {
    pub x1: T,
    pub y1: T,
    pub x2: T,
    pub y2: T,
}

impl<T> Rect<T> {
    pub fn new(x1: T, y1: T, x2: T, y2: T) -> Rect<T> {
        Rect { x1, y1, x2, y2 }
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

pub struct Pipeline2dDescription<'a> {
    description: core::PipelineDescription<'a>,
}

impl<'a> core::PipelineDescriptionLike<'static> for Pipeline2dDescription<'a> {
    type ConcretePipeline = Pipeline2d;
    type Uniforms = self::Uniforms;

    fn setup(&mut self, pip: core::Pipeline, dev: &core::Device, w: u32, h: u32) -> Pipeline2d {
        let ortho = ortho(w, h);
        let transform = Matrix4::identity();

        let buf =
            std::rc::Rc::new(dev.create_uniform_buffer(&[Self::Uniforms { ortho, transform }]));

        let binding = dev.create_binding(&pip.layout.sets[0], &[core::Uniform::Buffer(&buf)]);

        Pipeline2d {
            pipeline: pip,
            buf,
            binding,
            ortho,
        }
    }

    fn description(&self) -> &core::PipelineDescription {
        &self.description
    }
}

pub struct Pipeline2d {
    pipeline: core::Pipeline,
    binding: core::Uniforms,
    buf: std::rc::Rc<core::UniformBuffer>,
    ortho: Matrix4<f32>,
}

impl Pipeline2d {
    pub fn binding(
        &self,
        renderer: &core::Renderer,
        texture: &core::Texture,
        sampler: &core::Sampler,
    ) -> core::Uniforms {
        renderer.device.create_binding(
            &self.pipeline.layout.sets[1],
            &[
                core::Uniform::Texture(&texture),
                core::Uniform::Sampler(&sampler),
            ],
        )
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

    pub fn sprite_batch<'a>(
        &self,
        texture: &'a core::Texture,
        sampler: &'a core::Sampler,
    ) -> SpriteBatch<'a> {
        SpriteBatch::new(texture, sampler)
    }
}

impl<'a> core::PipelineLike<'a> for Pipeline2d {
    type PrepareContext = Matrix4<f32>;
    type Uniforms = self::Uniforms;

    fn resize(&mut self, w: u32, h: u32) {
        self.ortho = ortho(w, h);
    }

    fn apply(&self, pass: &mut core::Pass) {
        pass.apply_pipeline(&self.pipeline);
        pass.apply_uniforms(&self.binding, &[0]);
    }

    fn prepare(
        &'a self,
        transform: Matrix4<f32>,
    ) -> (std::rc::Rc<core::UniformBuffer>, Vec<self::Uniforms>) {
        (
            self.buf.clone(),
            vec![self::Uniforms {
                transform,
                ortho: self.ortho,
            }],
        )
    }
}

pub const SPRITE2D: Pipeline2dDescription = Pipeline2dDescription {
    description: core::PipelineDescription {
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
    },
};

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Framebuffer
///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Framebuffer {
    pub texture: Texture,
    pub sampler: Sampler,
    pub buffer: core::VertexBuffer,
    pub binding: core::Uniforms,
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

pub struct SpriteBatch<'a> {
    pub texture: &'a Texture,
    pub sampler: &'a Sampler,
    pub vertices: Vec<Vertex>,
    pub size: usize,
}

impl<'a> SpriteBatch<'a> {
    fn new(t: &'a Texture, s: &'a Sampler) -> Self {
        Self {
            texture: t,
            sampler: s,
            vertices: Vec::with_capacity(6),
            size: 0,
        }
    }

    pub fn add(&mut self, src: Rect<f32>, dst: Rect<f32>, rgba: Rgba, rep: Repeat) {
        let c: Rgba8 = rgba.into();
        let (tw, th) = (self.texture.w, self.texture.h);

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
