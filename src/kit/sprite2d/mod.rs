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
// Sprite
///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct Sprite {
    pub src: Rect<f32>,
    pub pos: Vector2<f32>,
    pub angle: f32,
    pub scale: Vector2<f32>,
    pub origin: Vector2<f32>,
    pub zdepth: ZDepth,
    pub color: Rgba,
    pub alpha: f32,
    pub repeat: Repeat,
}

impl Sprite {
    pub fn new(
        src: Rect<f32>
    ) -> Self {
        Self {
            src,
            pos: Vector2::new(0.0, 0.0),
            angle: Default::default(),
            scale: Vector2::new(1.0, 1.0),
            origin: Vector2::new(0.0, 0.0),
            zdepth: Default::default(),
            color: Default::default(),
            alpha: Default::default(),
            repeat: Default::default()
        }
    }

    pub fn position(mut self, pos: Vector2<f32>) -> Self {
        self.pos = pos;
        self
    }

    pub fn angle(mut self, angle: f32) -> Self {
        self.angle = angle;
        self
    }

    pub fn scale(mut self, scale: Vector2<f32>) -> Self {
        self.scale = scale;
        self
    }

    pub fn rectangle(mut self, dest: Rect<f32>) -> Self {
        let sprite_width = self.src.width();
        let sprite_height = self.src.height();

        let position = Vector2::new(dest.x1, dest.y1);
        let scale = Vector2::new(dest.width() / sprite_width, dest.height() / sprite_height);

        self.pos = position;
        self.scale = scale;
        self
    }

    pub fn origin(mut self, origin: Vector2<f32>) -> Self {
        self.origin = origin;
        self
    }

    pub fn color<T: Into<Rgba>>(mut self, color: T) -> Self {
        self.color = color.into();
        self
    }

    pub fn alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }

    pub fn zdepth<T: Into<ZDepth>>(mut self, zdepth: T) -> Self {
        self.zdepth = zdepth.into();
        self
    }

    pub fn repeat(mut self, x: f32, y: f32) -> Self {
        self.repeat = Repeat::new(x, y);
        self
    }
}

pub fn sprite(
    src: Rect<f32>,
    dst: Rect<f32>
) -> Sprite {
    Sprite::new(src).rectangle(dst)
}

pub fn sprite_pos(
    src: Rect<f32>,
    pos: Vector2<f32>,
    scale: Vector2<f32>,
) -> Sprite {
    Sprite::new(src).position(pos).scale(scale)
}

pub fn sprite_origin(
    src: Rect<f32>,
    pos: Vector2<f32>,
    scale: Vector2<f32>,
    origin: Vector2<f32>
) -> Sprite {
    Sprite::new(src).position(pos).scale(scale).origin(origin)
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Batch
///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct Batch {
    pub w: u32,
    pub h: u32,
    pub size: usize,

    items: Vec<Sprite>,
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
        pos: Vector2<f32>,
        angle: f32,
        scale: Vector2<f32>,
        origin: Vector2<f32>,
        zdepth: ZDepth,
        rgba: Rgba,
        alpha: f32,
        repeat: Repeat,
    ) -> Self {
        let mut view = Self::new(w, h);
        view.push(
            Sprite::new(src)
                .position(pos)
                .angle(angle)
                .scale(scale)
                .origin(origin)
                .zdepth(zdepth)
                .color(rgba)
                .alpha(alpha)
                .repeat(repeat.x, repeat.y),
        );
        view
    }

    pub fn push(&mut self, sprite: Sprite) {
        self.items.push(sprite);
    }

    pub fn add(
        &mut self,
        src: Rect<f32>,
        pos: Vector2<f32>,
        angle: f32,
        scale: Vector2<f32>,
        origin: Vector2<f32>,
        depth: ZDepth,
        rgba: Rgba,
        alpha: f32,
        repeat: Repeat,
    ) {
        if repeat != Repeat::default() {
            assert_eq!(
                src,
                Rect::origin(self.w as f32, self.h as f32),
                "using texture repeat is only valid when using the entire {}x{} texture",
                self.w,
                self.h
            );
        }
        self.items.push(
            Sprite::new(src)
                .position(pos)
                .angle(angle)
                .scale(scale)
                .origin(origin)
                .zdepth(depth)
                .color(rgba)
                .alpha(alpha)
                .repeat(repeat.x, repeat.y),
        );
        self.size += 1;
    }

    pub fn vertices(&self) -> Vec<Vertex> {
        let mut buf = Vec::with_capacity(6 * self.items.len());

        for Sprite {
            src,
            pos,
            angle,
            scale,
            zdepth,
            color,
            alpha,
            repeat,
            origin
        } in self.items.iter()
        {
            let ZDepth(z) = zdepth;
            let re = repeat;

            // Relative texture coordinates
            let rx1: f32 = src.x1 / self.w as f32;
            let ry1: f32 = src.y1 / self.h as f32;
            let rx2: f32 = src.x2 / self.w as f32;
            let ry2: f32 = src.y2 / self.h as f32;

            let c: Rgba8 = (*color).into();

            // Transform matrix
            let scale_mat = Matrix4::from_nonuniform_scale(
                scale.x * src.width(), scale.y * src.height(), 1.0
            );
            let origin_translation = Matrix4::from_translation(Vector3::new(
                -src.width() * origin.x * scale.x,
                -src.height() * origin.y * scale.y,
                0.0,
            ));
            let rotation = Matrix4::from_angle_z(*angle * std::f32::consts::PI / 180.0);
            let translation = Matrix4::from_translation(Vector3::new((*pos).x, (*pos).y, 0.0));
            let transformation = translation * rotation * origin_translation * scale_mat;

            let vec1 = Vector3::new(0.0, 0.0, 1.0);

            let vec1 = transformation * vec1;
            let vec2 = Vector3::new(1.0, 0.0, 1.0);
            let vec2 = transformation * vec2;
            let vec3 = Vector3::new(1.0, 1.0, 1.0);
            let vec3 = transformation * vec3;
            let vec4 = Vector3::new(0.0, 0.0, 1.0);
            let vec4 = transformation * vec4;
            let vec5 = Vector3::new(0.0, 1.0, 1.0);
            let vec5 = transformation * vec5;
            let vec6 = Vector3::new(1.0, 1.0, 1.0);
            let vec6 = transformation * vec6;

            // TODO: Use an index buffer
            buf.extend_from_slice(&[
                Vertex::new(vec1.x, vec1.y, *z, rx1 * re.x, ry2 * re.y, c, *alpha),
                Vertex::new(vec2.x, vec2.y, *z, rx2 * re.x, ry2 * re.y, c, *alpha),
                Vertex::new(vec3.x, vec3.y, *z, rx2 * re.x, ry1 * re.y, c, *alpha),
                Vertex::new(vec4.x, vec4.y, *z, rx1 * re.x, ry2 * re.y, c, *alpha),
                Vertex::new(vec5.x, vec5.y, *z, rx1 * re.x, ry1 * re.y, c, *alpha),
                Vertex::new(vec6.x, vec6.y, *z, rx2 * re.x, ry1 * re.y, c, *alpha),
            ]);
        }
        buf
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.size = 0;
    }

    pub fn offset(&mut self, x: f32, y: f32) {
        for sprite in self.items.iter_mut() {
            sprite.pos = sprite.pos + Vector2::new(x, y);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut batch = Batch::new(32, 32);
        batch.push(
            Sprite::new(Rect::new(32., 32., 64., 64.))
                .position(Vector2::new(0.0, 0.0))
                .scale(Vector2::new(1.0, 1.0))
                .origin(Vector2::new(0.5, 0.5))
                .color(Rgba::BLUE)
                .alpha(0.5)
                .zdepth(0.1)
                .repeat(8., 8.),
        );
    }
}
