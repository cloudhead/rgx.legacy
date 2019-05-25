#![deny(clippy::all, clippy::use_self)]

use cgmath::prelude::*;
use cgmath::{Matrix4, Vector2};

use crate::core;
use crate::core::{Binding, BindingType, Rect, Rgba, Set, ShaderStage, Texture};

use crate::kit;
use crate::kit::Repeat;

///////////////////////////////////////////////////////////////////////////
// Uniforms
///////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
pub struct Uniforms {
    pub ortho: Matrix4<f32>,
    pub transform: Matrix4<f32>,
}

///////////////////////////////////////////////////////////////////////////
// Rgba8
///////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
struct Rgba8 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Rgba8 {
    const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
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

///////////////////////////////////////////////////////////////////////////
// Vertex
///////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
pub struct Vertex {
    position: Vector2<f32>,
    uv: Vector2<f32>,
    color: Rgba8,
}

impl Vertex {
    fn new(x: f32, y: f32, u: f32, v: f32, color: Rgba8) -> Self {
        Self {
            position: Vector2::new(x, y),
            uv: Vector2::new(u, v),
            color,
        }
    }
}

///////////////////////////////////////////////////////////////////////////
// Pipeline
///////////////////////////////////////////////////////////////////////////

pub struct Pipeline {
    pipeline: core::Pipeline,
    bindings: core::BindingGroup,
    buf: core::UniformBuffer,
    ortho: Matrix4<f32>,
}

impl Pipeline {
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

impl<'a> core::PipelineLike<'a> for Pipeline {
    type PrepareContext = Matrix4<f32>;
    type Uniforms = self::Uniforms;

    fn description() -> core::PipelineDescription<'a> {
        core::PipelineDescription {
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
        }
    }

    fn setup(pipeline: core::Pipeline, dev: &core::Device, w: u32, h: u32) -> Self {
        let ortho = kit::ortho(w, h);
        let transform = Matrix4::identity();
        let buf = dev.create_uniform_buffer(&[self::Uniforms { ortho, transform }]);
        let bindings = dev.create_binding_group(&pipeline.layout.sets[0], &[&buf]);

        Self {
            pipeline,
            buf,
            bindings,
            ortho,
        }
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.ortho = kit::ortho(w, h);
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

///////////////////////////////////////////////////////////////////////////////////////////////////
/// Sprite
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

///////////////////////////////////////////////////////////////////////////////////////////////////
/// SpriteBatch
///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SpriteBatch {
    pub w: u32,
    pub h: u32,
    pub size: usize,

    views: Vec<(Rect<f32>, Rect<f32>, Rgba, Repeat)>,
}

impl SpriteBatch {
    fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            views: Vec::new(),
            size: 0,
        }
    }

    pub fn add(&mut self, src: Rect<f32>, dst: Rect<f32>, rgba: Rgba, rep: Repeat) {
        self.views.push((src, dst, rgba, rep));
        self.size += 1;
    }

    pub fn finish(self, r: &core::Renderer) -> core::VertexBuffer {
        let mut buf = Vec::<Vertex>::new();

        for (src, dst, rgba, rep) in self.views.iter() {
            // Relative texture coordinates
            let rx1: f32 = src.x1 / self.w as f32;
            let ry1: f32 = src.y1 / self.h as f32;
            let rx2: f32 = src.x2 / self.w as f32;
            let ry2: f32 = src.y2 / self.h as f32;

            let c: Rgba8 = (*rgba).into();

            // TODO: Use an index buffer
            let mut verts = vec![
                Vertex::new(dst.x1, dst.y1, rx1 * rep.x, ry2 * rep.y, c),
                Vertex::new(dst.x2, dst.y1, rx2 * rep.x, ry2 * rep.y, c),
                Vertex::new(dst.x2, dst.y2, rx2 * rep.x, ry1 * rep.y, c),
                Vertex::new(dst.x1, dst.y1, rx1 * rep.x, ry2 * rep.y, c),
                Vertex::new(dst.x1, dst.y2, rx1 * rep.x, ry1 * rep.y, c),
                Vertex::new(dst.x2, dst.y2, rx2 * rep.x, ry1 * rep.y, c),
            ];
            buf.append(&mut verts);
        }
        r.device.create_buffer(buf.as_slice())
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        for (_, dst, _, _) in self.views.iter_mut() {
            *dst = dst.translate(x, y);
        }
    }
}
