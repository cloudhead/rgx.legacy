use crate::core;
use crate::core::{Context, Texture, VertexLayout};

use wgpu::winit::Window;

use cgmath::prelude::*;
use cgmath::{Matrix4, Ortho};

#[derive(Copy, Clone)]
#[rustfmt::skip]
pub struct Vertex(
    f32, f32,               // X Y
    f32, f32, f32, f32,     // R G B A
    f32, f32,               // U V
);

pub struct Rect<T> {
    pub x1: T,
    pub y1: T,
    pub x2: T,
    pub y2: T,
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

pub struct Color<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl From<Color<u8>> for u32 {
    fn from(item: Color<u8>) -> Self {
        ((item.r as u32) << 24)
            | ((item.g as u32) << 16)
            | ((item.b as u32) << 8)
            | ((item.a as u32) << 0)
    }
}

pub struct Kit<'a> {
    pub ctx: Context<'a>,
    pub ortho: Ortho<f32>,
    pub transform: Matrix4<f32>,
}

impl<'a> Kit<'a> {
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

        Self {
            ctx,
            ortho,
            transform,
        }
    }

    #[allow(dead_code)]
    fn resize(&mut self, _w: u32, _h: u32) {
        unimplemented!();
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

///////////////////////////////////////////////////////////////////////////////////////////////////
/// SpriteBatch
///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SpriteBatch<'a> {
    pub texture: &'a Texture,
    pub vertices: Vec<Vertex>,
    pub buffer: Option<core::VertexBuffer>,
    pub size: usize,
}

impl<'a> SpriteBatch<'a> {
    pub fn new(t: &'a Texture) -> Self {
        Self {
            texture: t,
            vertices: Vec::with_capacity(6),
            buffer: None,
            size: 0,
        }
    }

    pub fn add(&mut self, src: Rect<f32>, dst: Rect<f32>, xrep: f32, yrep: f32, c: Color<f32>) {
        assert!(
            self.buffer.is_none(),
            "SpriteBatch::add called after SpriteBatch::finish"
        );

        let (tw, th) = (self.texture.w, self.texture.h);

        // Relative texture coordinates
        let rx1: f32 = src.x1 / tw as f32;
        let ry1: f32 = src.y1 / th as f32;
        let rx2: f32 = src.x2 / tw as f32;
        let ry2: f32 = src.y2 / th as f32;

        // TODO: Use an index buffer
        let mut verts: Vec<Vertex> = vec![
            Vertex(dst.x1, dst.y1, c.r, c.g, c.b, c.a, rx1 * xrep, ry2 * yrep),
            Vertex(dst.x2, dst.y1, c.r, c.g, c.b, c.a, rx2 * xrep, ry2 * yrep),
            Vertex(dst.x2, dst.y2, c.r, c.g, c.b, c.a, rx2 * xrep, ry1 * yrep),
            Vertex(dst.x1, dst.y1, c.r, c.g, c.b, c.a, rx1 * xrep, ry2 * yrep),
            Vertex(dst.x1, dst.y2, c.r, c.g, c.b, c.a, rx1 * xrep, ry1 * yrep),
            Vertex(dst.x2, dst.y2, c.r, c.g, c.b, c.a, rx2 * xrep, ry1 * yrep),
        ];

        self.vertices.append(&mut verts);
        self.size += 1;
    }

    pub fn finish(&mut self, ctx: &core::Context) {
        assert!(
            self.buffer.is_none(),
            "SpriteBatch::finish called more than once"
        );
        self.buffer = Some(ctx.create_buffer(self.vertices.as_slice()))
    }

    pub fn draw(&self, pass: &mut core::Pass) {
        let buffer = self
            .buffer
            .as_ref()
            .expect("SpriteBatch::finish wasn't called");

        pass.set_vertex_buffer(buffer);
        pass.draw(0..self.vertices.len() as u32, 0..1);
    }
}
