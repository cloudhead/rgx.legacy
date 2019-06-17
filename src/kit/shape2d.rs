#![deny(clippy::all, clippy::use_self)]
#![allow(clippy::new_without_default)]

use std::f32;

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

#[derive(PartialEq)]
pub struct Stroke {
    width: f32,
    color: Rgba,
}

impl Stroke {
    const NONE: Self = Self {
        width: 0.,
        color: Rgba::TRANSPARENT,
    };

    pub fn new(width: f32, color: Rgba) -> Self {
        Self { width, color }
    }
}

pub enum Fill {
    Empty(),
    Solid(Rgba),
    Gradient(Rgba, Rgba),
}

pub enum Shape {
    Line(Line, Stroke),
    Rectangle(Rect<f32>, Stroke, Fill),
    Circle(Vector2<f32>, f32, u32, Stroke, Fill),
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
            Shape::Line(l, Stroke { width, color }) => {
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
            Shape::Rectangle(r, Stroke { width, color }, fill) => {
                let w = width / 2.0;
                let stroke = vec![
                    Line::new(r.x1 + w, r.y1 + width, r.x1 + w, r.y2), // Left
                    Line::new(r.x2 - w, r.y1, r.x2 - w, r.y2 - width), // Right
                    Line::new(r.x1 + width, r.y2 - w, r.x2, r.y2 - w), // Top
                    Line::new(r.x1, r.y1 + w, r.x2 - width, r.y1 + w), // Bottom
                ];
                let mut verts = Self::with_capacity(stroke.len() * 6);
                for l in stroke {
                    let mut vs = Shape::Line(l, Stroke::new(width, color)).into();
                    verts.append(&mut vs);
                }

                match fill {
                    Fill::Solid(color) => {
                        let c = color.into();
                        let inner =
                            Rect::new(r.x1 + width, r.y1 + width, r.x2 - width, r.y2 - width);
                        let mut vs = vec![
                            Vertex::new(inner.x1, inner.y1, c),
                            Vertex::new(inner.x2, inner.y1, c),
                            Vertex::new(inner.x2, inner.y2, c),
                            Vertex::new(inner.x1, inner.y1, c),
                            Vertex::new(inner.x1, inner.y2, c),
                            Vertex::new(inner.x2, inner.y2, c),
                        ];
                        verts.append(&mut vs);
                    }
                    Fill::Gradient(_, _) => {
                        unimplemented!();
                    }
                    Fill::Empty() => {}
                }
                verts
            }
            Shape::Circle(position, radius, sides, stroke, fill) => {
                let mut verts = Self::new();
                let outer = self::circle_vertices(position, radius, sides, stroke.color.into());
                let inner = if stroke != Stroke::NONE {
                    // If there is a stroke, the inner circle is smaller.
                    let inner = self::circle_vertices(
                        position,
                        radius - stroke.width,
                        sides,
                        stroke.color.into(),
                    );

                    for i in 0..inner.len() - 1 {
                        verts.push(inner[i]);
                        verts.push(outer[i]);
                        verts.push(outer[i + 1]);
                        verts.push(inner[i]);
                        verts.push(outer[i + 1]);
                        verts.push(inner[i + 1]);
                    }
                    inner
                } else {
                    // If there is no stroke, the inner and outer circles are equal.
                    outer
                };

                match fill {
                    Fill::Solid(color) => {
                        let c = color.into();
                        let center = Vertex::new(position.x, position.y, c);
                        for i in 0..sides as usize {
                            verts.push(center);
                            verts.push(Vertex::new(inner[i].position.x, inner[i].position.y, c));
                            verts.push(Vertex::new(
                                inner[i + 1].position.x,
                                inner[i + 1].position.y,
                                c,
                            ));
                        }
                        verts.push(center);
                        verts.push(Vertex::new(
                            inner[inner.len() - 1].position.x,
                            inner[inner.len() - 1].position.y,
                            c,
                        ));
                        verts.push(Vertex::new(inner[0].position.x, inner[0].position.y, c));
                    }
                    Fill::Gradient(_, _) => {
                        unimplemented!();
                    }
                    Fill::Empty() => {}
                }
                verts
            }
        }
    }
}

fn circle_vertices(position: Vector2<f32>, radius: f32, sides: u32, c: Rgba8) -> Vec<Vertex> {
    let mut verts = Vec::new();

    for i in 0..=sides as usize {
        let angle: f32 = i as f32 * ((2. * f32::consts::PI) / sides as f32);
        verts.push(Vertex::new(
            position.x + radius * angle.cos(),
            position.y + radius * angle.sin(),
            c,
        ));
    }
    verts
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
