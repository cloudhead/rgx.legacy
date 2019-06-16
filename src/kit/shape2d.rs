#![deny(clippy::all, clippy::use_self)]
#![allow(clippy::new_without_default)]

use cgmath::prelude::*;
use cgmath::{Matrix4, Vector2};

use crate::core;
use crate::core::{Binding, BindingType, Rect, Rgba, Set, ShaderStage};

use crate::kit;
use crate::kit::{Model, Rgba8};

///////////////////////////////////////////////////////////////////////////
// Uniforms
///////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
pub struct Uniforms {
    pub ortho: Matrix4<f32>,
    pub transform: Matrix4<f32>,
}

///////////////////////////////////////////////////////////////////////////
// Vertex
///////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
pub struct Vertex {
    position: Vector2<f32>,
    color: Rgba8,
}

impl Vertex {
    fn new(x: f32, y: f32, color: Rgba8) -> Self {
        Self {
            position: Vector2::new(x, y),
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

//////////////////////////////////////////////////////////////////////////

impl<'a> core::AbstractPipeline<'a> for Pipeline {
    type PrepareContext = Matrix4<f32>;
    type Uniforms = self::Uniforms;

    fn description() -> core::PipelineDescription<'a> {
        core::PipelineDescription {
            vertex_layout: &[core::VertexFormat::Float2, core::VertexFormat::UByte4],
            pipeline_layout: &[
                Set(&[Binding {
                    binding: BindingType::UniformBuffer,
                    stage: ShaderStage::Vertex,
                }]),
                Set(&[Binding {
                    binding: BindingType::UniformBuffer,
                    stage: ShaderStage::Vertex,
                }]),
            ],
            // TODO: Use `env("CARGO_MANIFEST_DIR")`
            vertex_shader: include_str!("data/shape.vert"),
            fragment_shader: include_str!("data/shape.frag"),
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
/// Shapes
///////////////////////////////////////////////////////////////////////////////////////////////////

pub enum Shape {
    Line(Line, f32, Rgba),
    Rectangle(Rect<f32>, f32, Rgba),
}

pub struct Line {
    pub p1: Vector2<f32>,
    pub p2: Vector2<f32>,
}

impl Line {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            p1: Vector2::new(x1, y1),
            p2: Vector2::new(x2, y2),
        }
    }
}

impl From<Shape> for Vec<Vertex> {
    // TODO: (perf) This function is fairly CPU-inefficient.
    fn from(shape: Shape) -> Self {
        match shape {
            Shape::Line(l, width, color) => {
                let v = (l.p2 - l.p1).normalize();

                let wx = width / 2.0 * v.y;
                let wy = width / 2.0 * v.x;
                let c = color.into();

                vec![
                    Vertex::new(l.p1.x - wx, l.p1.y + wy, c),
                    Vertex::new(l.p1.x + wx, l.p1.y - wy, c),
                    Vertex::new(l.p2.x - wx, l.p2.y + wy, c),
                    Vertex::new(l.p2.x - wx, l.p2.y + wy, c),
                    Vertex::new(l.p1.x + wx, l.p1.y - wy, c),
                    Vertex::new(l.p2.x + wx, l.p2.y - wy, c),
                ]
            }
            Shape::Rectangle(r, width, color) => {
                let w = width / 2.0;
                let lines = vec![
                    Line::new(r.x1 + w, r.y1 + width, r.x1 + w, r.y2), // Left
                    Line::new(r.x2 - w, r.y1, r.x2 - w, r.y2 - width), // Right
                    Line::new(r.x1 + width, r.y2 - w, r.x2, r.y2 - w), // Top
                    Line::new(r.x1, r.y1 + w, r.x2 - width, r.y1 + w), // Bottom
                ];
                let mut verts = Self::with_capacity(lines.len() * 6);
                for l in lines {
                    let mut vs = Shape::Line(l, width, color).into();
                    verts.append(&mut vs);
                }
                verts
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
/// ShapeView
///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ShapeView {
    views: Vec<Shape>,
}

impl ShapeView {
    pub fn new() -> Self {
        Self { views: Vec::new() }
    }

    pub fn add(&mut self, shape: Shape) {
        self.views.push(shape);
    }

    pub fn finish(self, r: &core::Renderer) -> core::VertexBuffer {
        let mut buf = Vec::<Vertex>::new();

        for shape in self.views {
            let mut verts: Vec<Vertex> = shape.into();
            buf.append(&mut verts);
        }
        r.device.create_buffer(buf.as_slice())
    }
}
