pub mod algebra;
pub mod rect;
pub mod size;
pub mod traits;
pub mod transform;

pub use algebra::*;
pub use rect::*;
pub use size::*;
pub use traits::*;
pub use transform::*;

/// Any mathematical object that can be transformed.
pub trait Geometry: Sized {
    /// Return a transformed object.
    fn transform(self, t: impl Into<Transform>) -> Self;

    /// Undos a transform.
    fn untransform(self, t: impl Into<Transform>) -> Self {
        self.transform(t.into().inverse())
    }
}

impl Geometry for Rect<f32> {
    fn transform(self, t: impl Into<Transform>) -> Self {
        let t = t.into();
        let min = t * self.min();
        let max = t * self.max();

        Self::points(min, max)
    }
}

impl Geometry for Point2D<f32> {
    fn transform(self, t: impl Into<Transform>) -> Self {
        t.into() * self
    }
}
