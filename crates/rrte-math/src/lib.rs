pub mod vector;
pub mod matrix;
pub mod ray;
pub mod bounds;
pub mod transform;
pub mod color;

pub use glam::{Vec2, Vec3, Vec4, Mat3, Mat4, Quat};
pub use vector::*;
pub use matrix::*;
pub use ray::*;
pub use bounds::*;
pub use transform::*;
pub use color::*;

/// Common mathematical constants
pub mod constants {
    pub const PI: f32 = std::f32::consts::PI;
    pub const TAU: f32 = std::f32::consts::TAU;
    pub const EPSILON: f32 = 1e-6;
    pub const INFINITY: f32 = f32::INFINITY;
}

/// Utility functions for common mathematical operations
pub mod utils {

    /// Clamp a value between min and max
    pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
        value.max(min).min(max)
    }

    /// Linear interpolation between two values
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    /// Smooth step interpolation
    pub fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// Convert degrees to radians
    pub fn deg_to_rad(degrees: f32) -> f32 {
        degrees * super::constants::PI / 180.0
    }

    /// Convert radians to degrees
    pub fn rad_to_deg(radians: f32) -> f32 {
        radians * 180.0 / super::constants::PI
    }
}
