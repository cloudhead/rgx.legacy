#[cfg(feature = "renderer")]
mod backend;
#[cfg(feature = "renderer")]
pub use backend::*;

use crate::color::Rgba;
use crate::kit::ZDepth;
use crate::kit::{Repeat, Rgba8};
use crate::math::*;
use crate::rect::Rect;

///////////////////////////////////////////////////////////////////////////
// Vertex
///////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub uv: Vector2<f32>,
    pub color: Rgba8,
    pub opacity: f32,
}

impl Vertex {
    fn new(x: f32, y: f32, z: f32, u: f32, v: f32, color: Rgba8, opacity: f32) -> Self {
        Self {
            position: Vector3::new(x, y, z),
            uv: Vector2::new(u, v),
            color,
            opacity,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Batch
///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct Batch {
    pub w: u32,
    pub h: u32,
    pub size: usize,

    items: Vec<(Rect<f32>, Rect<f32>, ZDepth, Rgba, f32, Repeat)>,
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
        depth: ZDepth,
        rgba: Rgba,
        opa: f32,
        rep: Repeat,
    ) -> Self {
        let mut view = Self::new(w, h);
        view.add(src, dst, depth, rgba, opa, rep);
        view
    }

    pub fn add(
        &mut self,
        src: Rect<f32>,
        dst: Rect<f32>,
        depth: ZDepth,
        rgba: Rgba,
        opacity: f32,
        rep: Repeat,
    ) {
        if rep != Repeat::default() {
            assert!(
                src == Rect::origin(self.w as f32, self.h as f32),
                "using texture repeat is only valid when using the entire {}x{} texture",
                self.w,
                self.h
            );
        }
        self.items.push((src, dst, depth, rgba, opacity, rep));
        self.size += 1;
    }

    pub fn vertices(&self) -> Vec<Vertex> {
        let mut buf = Vec::with_capacity(6 * self.items.len());

        for (src, dst, ZDepth(z), rgba, o, rep) in self.items.iter() {
            // Relative texture coordinates
            let rx1: f32 = src.x1 / self.w as f32;
            let ry1: f32 = src.y1 / self.h as f32;
            let rx2: f32 = src.x2 / self.w as f32;
            let ry2: f32 = src.y2 / self.h as f32;

            let c: Rgba8 = (*rgba).into();

            // TODO: Use an index buffer
            buf.extend_from_slice(&[
                Vertex::new(dst.x1, dst.y1, *z, rx1 * rep.x, ry2 * rep.y, c, *o),
                Vertex::new(dst.x2, dst.y1, *z, rx2 * rep.x, ry2 * rep.y, c, *o),
                Vertex::new(dst.x2, dst.y2, *z, rx2 * rep.x, ry1 * rep.y, c, *o),
                Vertex::new(dst.x1, dst.y1, *z, rx1 * rep.x, ry2 * rep.y, c, *o),
                Vertex::new(dst.x1, dst.y2, *z, rx1 * rep.x, ry1 * rep.y, c, *o),
                Vertex::new(dst.x2, dst.y2, *z, rx2 * rep.x, ry1 * rep.y, c, *o),
            ]);
        }
        buf
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.size = 0;
    }

    pub fn offset(&mut self, x: f32, y: f32) {
        for (_, dst, _, _, _, _) in self.items.iter_mut() {
            *dst = *dst + Vector2::new(x, y);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
