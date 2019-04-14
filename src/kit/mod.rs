#![allow(dead_code)]
use crate::core;
use crate::core::{
    Binding, BindingType, Context, Sampler, Set, ShaderStage, Texture, VertexLayout,
};

pub use crate::core::Rgba;

use wgpu::winit::dpi::PhysicalSize;
use wgpu::winit::Window;

use cgmath::prelude::*;
use cgmath::{Matrix4, Ortho, Vector2};

use std::rc::*;

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

#[derive(Copy, Clone)]
pub struct Uniforms {
    pub ortho: Matrix4<f32>,
    pub transform: Matrix4<f32>,
}

impl Vertex {
    fn new(x: f32, y: f32, u: f32, v: f32, c: Rgba8) -> Vertex {
        Vertex {
            position: Vector2::new(x, y),
            uv: Vector2::new(u, v),
            color: c,
        }
    }
}

pub struct Repeat {
    pub x: f32,
    pub y: f32,
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

pub struct Kit {
    pub ctx: Context,
    pub ortho: Ortho<f32>,
    pub transform: Matrix4<f32>,
    pub clear: Rgba,
    pub pipeline: core::Pipeline,

    mvp_buf: Rc<core::UniformBuffer>,
    mvp_binding: core::Uniforms,
}

impl Kit {
    pub fn new(w: &Window) -> Self {
        let win_size = w
            .get_inner_size()
            .unwrap()
            .to_physical(w.get_hidpi_factor());

        let transform = Matrix4::identity();
        let ortho = Ortho::<f32> {
            left: 0.0,
            right: win_size.width as f32,
            bottom: 0.0,
            top: win_size.height as f32,
            near: -1.0,
            far: 1.0,
        };

        let ctx = Context::new(w);

        let vertex_layout = VertexLayout::from(&[
            core::VertexFormat::Float2,
            core::VertexFormat::Float2,
            core::VertexFormat::UByte4,
        ]);
        let pipeline_layout = ctx.create_pipeline_layout(&[
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
        ]);

        let clear = Rgba::new(1.0, 1.0, 1.0, 1.0);

        // TODO: Use `env("CARGO_MANIFEST_DIR")`
        let pipeline = {
            let vs = ctx.create_shader(
                "shader.vert",
                include_str!("data/shader.vert"),
                ShaderStage::Vertex,
            );

            let fs = ctx.create_shader(
                "shader.frag",
                include_str!("data/shader.frag"),
                ShaderStage::Fragment,
            );
            ctx.create_pipeline(pipeline_layout, vertex_layout, &vs, &fs)
        };

        let mvp_buf = Rc::new(ctx.create_uniform_buffer(Uniforms {
            ortho: ortho.into(),
            transform,
        }));
        let mvp_binding =
            ctx.create_binding(&pipeline.layout.sets[0], &[core::Uniform::Buffer(&mvp_buf)]);

        Self {
            ctx,
            ortho,
            transform,
            clear,
            pipeline,
            mvp_buf,
            mvp_binding,
        }
    }

    pub fn texture(&mut self, texels: &[u8], w: u32, h: u32) -> Texture {
        self.ctx.create_texture(texels, w, h)
    }

    pub fn sampler(&self, min_filter: core::Filter, mag_filter: core::Filter) -> core::Sampler {
        self.ctx.create_sampler(min_filter, mag_filter)
    }

    pub fn frame<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Pass),
    {
        let clear = self.clear;
        let mut frame = self._frame();
        {
            let mut pass = frame.pass(clear);
            f(&mut pass);
        }
        frame.commit();
    }

    pub fn offscreen<F>(&mut self, fb: &Framebuffer, f: F)
    where
        F: FnOnce(&mut Pass),
    {
        let clear = self.clear;
        let mut frame = self._frame();
        {
            let mut pass = frame.offscreen_pass(&fb.texture, clear);
            f(&mut pass);
        }
        frame.commit();
    }

    pub fn canvas(&mut self, physical: PhysicalSize) -> Framebuffer {
        let texture = self
            .ctx
            .create_framebuffer_texture(physical.width as u32, physical.height as u32);
        let sampler = self.sampler(core::Filter::Nearest, core::Filter::Nearest);

        self.framebuffer(texture, sampler)
    }

    pub fn resize(&mut self, physical: PhysicalSize) {
        self.ctx.resize(physical);

        self.ortho = Ortho::<f32> {
            left: 0.0,
            right: physical.width as f32,
            bottom: 0.0,
            top: physical.height as f32,
            near: -1.0,
            far: 1.0,
        };
    }

    ////////////////////////////////////////////////////////////////////////////

    fn framebuffer(&self, texture: Texture, sampler: Sampler) -> Framebuffer {
        let (tw, th) = (texture.w, texture.h);

        let src = texture.rect();
        let dst = texture.rect();
        let rep = Repeat::default();

        // Relative texture coordinates
        let rx1: f32 = src.x1 / tw as f32;
        let ry1: f32 = src.y1 / th as f32;
        let rx2: f32 = src.x2 / tw as f32;
        let ry2: f32 = src.y2 / th as f32;

        let c = Rgba::TRANSPARENT.into();

        let vertices = vec![
            Vertex::new(dst.x1, dst.y1, rx1 * rep.x, ry2 * rep.y, c),
            Vertex::new(dst.x2, dst.y1, rx2 * rep.x, ry2 * rep.y, c),
            Vertex::new(dst.x2, dst.y2, rx2 * rep.x, ry1 * rep.y, c),
            Vertex::new(dst.x1, dst.y1, rx1 * rep.x, ry2 * rep.y, c),
            Vertex::new(dst.x1, dst.y2, rx1 * rep.x, ry1 * rep.y, c),
            Vertex::new(dst.x2, dst.y2, rx2 * rep.x, ry1 * rep.y, c),
        ];

        let buffer = self.ctx.create_buffer(vertices.as_slice());

        #[rustfmt::skip]
        let binding = self.ctx.create_binding(
            &self.pipeline.layout.sets[1],
            &[
                core::Uniform::Texture(&texture),
                core::Uniform::Sampler(&sampler)
            ],
        );

        Framebuffer {
            texture,
            sampler,
            buffer,
            binding,
        }
    }

    fn _frame(&mut self) -> Frame {
        self.ctx.update_uniform_buffer(
            self.mvp_buf.clone(),
            Uniforms {
                transform: self.transform,
                ortho: self.ortho.into(),
            },
        );

        Frame {
            frame: self.ctx.frame(),
            pipeline: &self.pipeline,
            mvp_binding: &self.mvp_binding,
        }
    }
}

impl Drawable for Framebuffer {
    fn draw(&self, pass: &mut core::Pass) {
        pass.apply_uniforms(&self.binding);
        pass.set_vertex_buffer(&self.buffer);
        pass.draw(0..6, 0..1);
    }
}

pub struct Pass<'a> {
    pass: core::Pass<'a>,
}

impl<'a> Pass<'a> {
    pub fn draw<T: Drawable>(&mut self, t: &T) {
        t.draw(&mut self.pass)
    }
}

pub struct Frame<'a> {
    frame: core::Frame<'a>,
    pipeline: &'a core::Pipeline,
    mvp_binding: &'a core::Uniforms,
}

impl<'a> Frame<'a> {
    pub fn pass(&mut self, clear_color: Rgba) -> Pass {
        let mut pass = self.frame.begin_pass(clear_color);
        pass.apply_pipeline(&self.pipeline);
        pass.apply_uniforms(&self.mvp_binding);
        Pass { pass }
    }

    pub fn offscreen_pass(&mut self, texture: &Texture, clear_color: Rgba) -> Pass {
        let mut pass = self.frame.begin_offscreen_pass(texture, clear_color);
        pass.apply_pipeline(&self.pipeline);
        pass.apply_uniforms(&self.mvp_binding);
        Pass { pass }
    }

    pub fn commit(self) {
        self.frame.commit();
    }
}

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
/// Drawable
///////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Drawable {
    fn draw(&self, pass: &mut core::Pass);
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
/// SpriteBatch
///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SpriteBatch<'a> {
    pub texture: &'a Texture,
    pub sampler: &'a Sampler,
    pub vertices: Vec<Vertex>,
    pub buffer: Option<core::VertexBuffer>,
    pub binding: Option<core::Uniforms>,
    pub size: usize,
}

impl<'a> SpriteBatch<'a> {
    pub fn new(t: &'a Texture, s: &'a Sampler) -> Self {
        Self {
            texture: t,
            sampler: s,
            vertices: Vec::with_capacity(6),
            buffer: None,
            binding: None,
            size: 0,
        }
    }

    pub fn add(&mut self, src: Rect<f32>, dst: Rect<f32>, rgba: Rgba, rep: Repeat) {
        assert!(
            self.buffer.is_none(),
            "SpriteBatch::add called after SpriteBatch::finish"
        );

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

    pub fn finish(&mut self, kit: &Kit) {
        assert!(
            self.buffer.is_none(),
            "SpriteBatch::finish called more than once"
        );
        let buffer = kit.ctx.create_buffer(self.vertices.as_slice());
        #[rustfmt::skip]
        let binding = kit.ctx.create_binding(
            &kit.pipeline.layout.sets[1],
            &[
                core::Uniform::Texture(&self.texture),
                core::Uniform::Sampler(&self.sampler)
            ],
        );
        self.buffer = Some(buffer);
        self.binding = Some(binding);
    }
}

impl<'a> Drawable for SpriteBatch<'a> {
    fn draw(&self, pass: &mut core::Pass) {
        let buffer = self
            .buffer
            .as_ref()
            .expect("SpriteBatch::finish wasn't called");
        let binding = self
            .binding
            .as_ref()
            .expect("SpriteBatch::finish wasn't called");

        pass.apply_uniforms(binding);
        pass.set_vertex_buffer(buffer);
        pass.draw(0..self.vertices.len() as u32, 0..1);
    }
}
