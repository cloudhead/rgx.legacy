#![deny(clippy::all, clippy::use_self)]
#![allow(clippy::new_without_default)]

use std::f32;

use cgmath::prelude::*;
use cgmath::{Matrix4, Point2, Vector2};

use arrayvec::ArrayVec;

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

impl Shape {
    // TODO: (perf) This function is fairly CPU-inefficient.
    pub fn triangulate(self) -> Vec<Vertex> {
        match self {
            Shape::Line(l, Stroke { width, color }) => {
                let v = (l.p2 - l.p1).normalize();

                let wx = width / 2.0 * v.y;
                let wy = width / 2.0 * v.x;
                let rgba8 = color.into();

                vec![
                    Vertex::new(l.p1.x - wx, l.p1.y + wy, rgba8),
                    Vertex::new(l.p1.x + wx, l.p1.y - wy, rgba8),
                    Vertex::new(l.p2.x - wx, l.p2.y + wy, rgba8),
                    Vertex::new(l.p2.x - wx, l.p2.y + wy, rgba8),
                    Vertex::new(l.p1.x + wx, l.p1.y - wy, rgba8),
                    Vertex::new(l.p2.x + wx, l.p2.y - wy, rgba8),
                ]
            }
            Shape::Rectangle(r, stroke, fill) => {
                let width = stroke.width;
                let inner = Rect::new(r.x1 + width, r.y1 + width, r.x2 - width, r.y2 - width);

                let mut verts = if stroke != Stroke::NONE {
                    let rgba8 = stroke.color.into();

                    let outer = r;

                    vec![
                        // Bottom
                        Vertex::new(outer.x1, outer.y1, rgba8),
                        Vertex::new(outer.x2, outer.y1, rgba8),
                        Vertex::new(inner.x1, inner.y1, rgba8),
                        Vertex::new(inner.x1, inner.y1, rgba8),
                        Vertex::new(outer.x2, outer.y1, rgba8),
                        Vertex::new(inner.x2, inner.y1, rgba8),
                        // Left
                        Vertex::new(outer.x1, outer.y1, rgba8),
                        Vertex::new(inner.x1, inner.y1, rgba8),
                        Vertex::new(outer.x1, outer.y2, rgba8),
                        Vertex::new(outer.x1, outer.y2, rgba8),
                        Vertex::new(inner.x1, inner.y1, rgba8),
                        Vertex::new(inner.x1, inner.y2, rgba8),
                        // Right
                        Vertex::new(inner.x2, inner.y1, rgba8),
                        Vertex::new(outer.x2, outer.y1, rgba8),
                        Vertex::new(outer.x2, outer.y2, rgba8),
                        Vertex::new(inner.x2, inner.y1, rgba8),
                        Vertex::new(inner.x2, inner.y2, rgba8),
                        Vertex::new(outer.x2, outer.y2, rgba8),
                        // Top
                        Vertex::new(outer.x1, outer.y2, rgba8),
                        Vertex::new(outer.x2, outer.y2, rgba8),
                        Vertex::new(inner.x1, inner.y2, rgba8),
                        Vertex::new(inner.x1, inner.y2, rgba8),
                        Vertex::new(outer.x2, outer.y2, rgba8),
                        Vertex::new(inner.x2, inner.y2, rgba8),
                    ]
                } else {
                    Vec::with_capacity(6)
                };

                match fill {
                    Fill::Solid(color) => {
                        let rgba8 = color.into();

                        verts.extend_from_slice(&[
                            Vertex::new(inner.x1, inner.y1, rgba8),
                            Vertex::new(inner.x2, inner.y1, rgba8),
                            Vertex::new(inner.x2, inner.y2, rgba8),
                            Vertex::new(inner.x1, inner.y1, rgba8),
                            Vertex::new(inner.x1, inner.y2, rgba8),
                            Vertex::new(inner.x2, inner.y2, rgba8),
                        ]);
                    }
                    Fill::Gradient(_, _) => {
                        unimplemented!();
                    }
                    Fill::Empty() => {}
                }
                verts
            }
            Shape::Circle(position, radius, sides, stroke, fill) => {
                let inner = Self::circle(position, radius - stroke.width, sides);

                let mut verts = if stroke != Stroke::NONE {
                    // If there is a stroke, the outer circle is larger.
                    let outer = Self::circle(position, radius, sides);
                    let rgba8 = stroke.color.into();

                    let n = inner.len() - 1;
                    let mut vs = Vec::with_capacity(n * 6);
                    for i in 0..n {
                        let (i0, i1) = (inner[i], inner[i + 1]);
                        let (o0, o1) = (outer[i], outer[i + 1]);

                        vs.extend_from_slice(&[
                            Vertex::new(i0.x, i0.y, rgba8),
                            Vertex::new(o0.x, o0.y, rgba8),
                            Vertex::new(o1.x, o1.y, rgba8),
                            Vertex::new(i0.x, i0.y, rgba8),
                            Vertex::new(o1.x, o1.y, rgba8),
                            Vertex::new(i1.x, i1.y, rgba8),
                        ]);
                    }
                    vs
                } else {
                    Vec::new()
                };

                match fill {
                    Fill::Solid(color) => {
                        let rgba8 = color.into();
                        let center = Vertex::new(position.x, position.y, rgba8);
                        let inner_verts: Vec<Vertex> =
                            inner.iter().map(|p| Vertex::new(p.x, p.y, rgba8)).collect();
                        for i in 0..sides as usize {
                            verts.extend_from_slice(&[center, inner_verts[i], inner_verts[i + 1]]);
                        }
                        verts.extend_from_slice(&[
                            center,
                            *inner_verts.last().unwrap(),
                            *inner_verts.first().unwrap(),
                        ]);
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

    fn circle(position: Vector2<f32>, radius: f32, sides: u32) -> Vec<Point2<f32>> {
        let mut verts = Vec::with_capacity(sides as usize + 1);

        for i in 0..=sides as usize {
            let angle: f32 = i as f32 * ((2. * f32::consts::PI) / sides as f32);
            verts.push(Point2::new(
                position.x + radius * angle.cos(),
                position.y + radius * angle.sin(),
            ));
        }
        verts
    }
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
            let mut verts: Vec<Vertex> = shape.triangulate();
            buf.append(&mut verts);
        }
        r.device.create_buffer(buf.as_slice())
    }
}