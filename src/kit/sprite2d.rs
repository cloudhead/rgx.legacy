#![deny(clippy::all, clippy::use_self)]

use cgmath::prelude::*;
use cgmath::{Matrix4, Vector2};

use crate::core;
use crate::core::{Binding, BindingType, Rect, Rgba, Set, ShaderStage};

use crate::kit;
use crate::kit::Repeat;

use crate::nonempty::NonEmpty;

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
pub struct Rgba8 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Rgba8 {
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    pub const WHITE: Self = Self {
        r: 0xff,
        g: 0xff,
        b: 0xff,
        a: 0xff,
    };
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0xff,
    };
    pub const RED: Self = Self {
        r: 0xff,
        g: 0,
        b: 0,
        a: 0xff,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: 0xff,
        b: 0,
        a: 0xff,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 0xff,
        a: 0xff,
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

    pub fn frame<'a, F>(&mut self, r: &mut core::Renderer, clear: Rgba, inner: F)
    where
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
        }

        let mut raw = r.frame();
        if !grow {
            let data = Model::aligned(slice);
            raw.update_uniform_buffer(&self.model.buf, data.as_slice());
        }

        let mut pass = raw.pass(clear);

        // Bypass the AbstractPipeline implementation.
        pass.apply_pipeline(&self.pipeline);
        pass.apply_binding(&self.bindings, &[0]);

        let mut i = 0;
        for Command(buf, bin, _) in &frame.commands {
            pass.apply_binding(&self.model.binding, &[i]);
            pass.apply_binding(bin, &[]);
            pass.set_vertex_buffer(buf);
            pass.draw_buffer(0..buf.size, 0..1);

            i += AlignedBuffer::ALIGNMENT;
        }
    }
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
        pass.apply_pipeline(&self.pipeline);
        pass.apply_binding(&self.bindings, &[0]);
        pass.apply_binding(&self.model.binding, &[0]);
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
/// TextureView
///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct TextureView {
    pub w: u32,
    pub h: u32,
    pub size: usize,

    views: Vec<(Rect<f32>, Rect<f32>, Rgba, Repeat)>,
}

impl TextureView {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            views: Vec::new(),
            size: 0,
        }
    }

    pub fn singleton(
        w: u32,
        h: u32,
        src: Rect<f32>,
        dst: Rect<f32>,
        rgba: Rgba,
        rep: Repeat,
    ) -> Self {
        let mut view = Self::new(w, h);
        view.add(src, dst, rgba, rep);
        view
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

    pub fn offset(&mut self, x: f32, y: f32) {
        for (_, dst, _, _) in self.views.iter_mut() {
            *dst = *dst + Vector2::new(x, y);
        }
    }
}
