#![deny(clippy::all, clippy::use_self)]
#![allow(clippy::new_without_default)]

use cgmath::prelude::*;
use cgmath::{Matrix4, Vector2, Vector3};

use crate::core;
use crate::core::{Binding, BindingType, PassOp, Rect, Rgba, Set, ShaderStage};

use crate::kit;
use crate::kit::{AlignedBuffer, Model, Repeat, Rgba8};

use crate::nonempty::NonEmpty;

///////////////////////////////////////////////////////////////////////////
// Uniforms
///////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Uniforms {
    pub ortho: Matrix4<f32>,
    pub transform: Matrix4<f32>,
}

///////////////////////////////////////////////////////////////////////////
// Vertex
///////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: Vector2<f32>,
    uv: Vector2<f32>,
    color: Rgba8,
    opacity: f32,
}

impl Vertex {
    fn new(x: f32, y: f32, u: f32, v: f32, color: Rgba8, opacity: f32) -> Self {
        Self {
            position: Vector2::new(x, y),
            uv: Vector2::new(u, v),
            color,
            opacity,
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
    model: Model,
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
            .create_binding_group(&self.pipeline.layout.sets[2], &[texture, sampler])
    }

    pub fn frame<'a, T, F>(&mut self, r: &mut core::Renderer, op: PassOp, view: T, inner: F)
    where
        T: core::TextureView,
        F: FnOnce(&mut Frame<'a>),
    {
        let mut frame = Frame {
            commands: Vec::new(),
            transforms: NonEmpty::singleton(Matrix4::identity()),
        };

        inner(&mut frame);

        let mut transforms = Vec::new();
        for Command(_, _, t) in &frame.commands {
            transforms.push(*t);
        }

        let slice = transforms.as_slice();
        let grow = self.model.size < slice.len();

        if grow {
            self.model = Model::new(&self.pipeline.layout.sets[1], slice, &r.device);
        } else {
            let data = Model::aligned(slice);
            let mut e = r.device.create_command_encoder();
            r.device
                .update_uniform_buffer(data.as_slice(), &self.model.buf, &mut e);
            r.device.submit(&[e.finish()]);
        }

        let mut raw = r.frame();
        {
            let mut pass = raw.pass(op, view);

            // Bypass the AbstractPipeline implementation.
            pass.set_pipeline(&self.pipeline);
            pass.set_binding(&self.bindings, &[0]);

            let mut i = 0;
            for Command(buf, bin, _) in &frame.commands {
                pass.set_binding(&self.model.binding, &[i]);
                pass.set_binding(bin, &[]);
                pass.set_vertex_buffer(buf);
                pass.draw_buffer(0..buf.size, 0..1);

                i += AlignedBuffer::ALIGNMENT;
            }
        }
        r.submit(raw);
    }
}

//////////////////////////////////////////////////////////////////////////

pub struct Command<'a>(&'a core::VertexBuffer, &'a core::BindingGroup, Matrix4<f32>);

pub struct Frame<'a> {
    commands: Vec<Command<'a>>,
    transforms: NonEmpty<Matrix4<f32>>,
}

impl<'a> Frame<'a> {
    pub fn draw(&mut self, buffer: &'a core::VertexBuffer, binding: &'a core::BindingGroup) {
        self.commands
            .push(Command(buffer, binding, *self.transforms.last()));
    }

    pub fn transform<F>(&mut self, t: Matrix4<f32>, inner: F)
    where
        F: FnOnce(&mut Self),
    {
        self.transforms.push(self.transforms.last() * t);
        inner(self);
        self.transforms.pop();
    }

    pub fn translate<F>(&mut self, x: f32, y: f32, inner: F)
    where
        F: FnOnce(&mut Self),
    {
        self.transform(Matrix4::from_translation(Vector3::new(x, y, 0.)), inner);
    }

    pub fn scale<F>(&mut self, s: f32, inner: F)
    where
        F: FnOnce(&mut Self),
    {
        self.transform(Matrix4::from_scale(s), inner);
    }
}

//////////////////////////////////////////////////////////////////////////

impl<'a> core::AbstractPipeline<'a> for Pipeline {
    type PrepareContext = Matrix4<f32>;
    type Uniforms = self::Uniforms;

    fn description() -> core::PipelineDescription<'a> {
        core::PipelineDescription {
            vertex_layout: &[
                core::VertexFormat::Float2,
                core::VertexFormat::Float2,
                core::VertexFormat::UByte4,
                core::VertexFormat::Float,
            ],
            pipeline_layout: &[
                Set(&[Binding {
                    binding: BindingType::UniformBuffer,
                    stage: ShaderStage::Vertex,
                }]),
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
        let model = Model::new(&pipeline.layout.sets[1], &[Matrix4::identity()], dev);
        let buf = dev.create_uniform_buffer(&[self::Uniforms { ortho, transform }]);
        let bindings = dev.create_binding_group(&pipeline.layout.sets[0], &[&buf]);

        Self {
            pipeline,
            buf,
            bindings,
            model,
            ortho,
        }
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.ortho = kit::ortho(w, h);
    }

    fn apply(&self, pass: &mut core::Pass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_binding(&self.bindings, &[0]);
        pass.set_binding(&self.model.binding, &[0]);
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
/// Batch
///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct Batch {
    pub w: u32,
    pub h: u32,
    pub size: usize,

    items: Vec<(Rect<f32>, Rect<f32>, Rgba, f32, Repeat)>,
}

impl Batch {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            items: Vec::new(),
            size: 0,
        }
    }

    pub fn singleton(
        w: u32,
        h: u32,
        src: Rect<f32>,
        dst: Rect<f32>,
        rgba: Rgba,
        opa: f32,
        rep: Repeat,
    ) -> Self {
        let mut view = Self::new(w, h);
        view.add(src, dst, rgba, opa, rep);
        view
    }

    pub fn add(&mut self, src: Rect<f32>, dst: Rect<f32>, rgba: Rgba, opacity: f32, rep: Repeat) {
        self.items.push((src, dst, rgba, opacity, rep));
        self.size += 1;
    }

    pub fn vertices(&self) -> Vec<Vertex> {
        let mut buf = Vec::with_capacity(6 * self.items.len());

        for (src, dst, rgba, o, rep) in self.items.iter() {
            // Relative texture coordinates
            let rx1: f32 = src.x1 / self.w as f32;
            let ry1: f32 = src.y1 / self.h as f32;
            let rx2: f32 = src.x2 / self.w as f32;
            let ry2: f32 = src.y2 / self.h as f32;

            let c: Rgba8 = (*rgba).into();

            // TODO: Use an index buffer
            buf.extend_from_slice(&[
                Vertex::new(dst.x1, dst.y1, rx1 * rep.x, ry2 * rep.y, c, *o),
                Vertex::new(dst.x2, dst.y1, rx2 * rep.x, ry2 * rep.y, c, *o),
                Vertex::new(dst.x2, dst.y2, rx2 * rep.x, ry1 * rep.y, c, *o),
                Vertex::new(dst.x1, dst.y1, rx1 * rep.x, ry2 * rep.y, c, *o),
                Vertex::new(dst.x1, dst.y2, rx1 * rep.x, ry1 * rep.y, c, *o),
                Vertex::new(dst.x2, dst.y2, rx2 * rep.x, ry1 * rep.y, c, *o),
            ]);
        }
        buf
    }

    pub fn finish(self, r: &core::Renderer) -> core::VertexBuffer {
        let buf = self.vertices();
        r.device.create_buffer(buf.as_slice())
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.size = 0;
    }

    pub fn offset(&mut self, x: f32, y: f32) {
        for (_, dst, _, _, _) in self.items.iter_mut() {
            *dst = *dst + Vector2::new(x, y);
        }
    }
}
