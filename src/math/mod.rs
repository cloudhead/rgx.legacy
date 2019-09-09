#[cfg(not(feature = "cgmath"))]
pub mod algebra;
#[cfg(not(feature = "cgmath"))]
pub use algebra::*;
#[cfg(not(feature = "cgmath"))]
pub use num_traits::{Float, One, Zero};

#[cfg(feature = "cgmath")]
pub use cgmath::prelude::*;
#[cfg(feature = "cgmath")]
pub use cgmath::*;
