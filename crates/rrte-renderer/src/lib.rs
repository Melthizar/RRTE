//! Rendering utilities for the RRTE engine.
//!
//! This crate contains CPU and GPU based renderers along with common
//! primitives, materials and camera types.

/// Raytracing implementation.
pub mod raytracer;
/// Material definitions and utilities.
pub mod material;
/// Primitive geometry types.
pub mod primitives;
/// Scene management structures.
pub mod scene;
/// Lighting types.
pub mod light;
/// GPU-based renderer implementation.
pub mod gpu_renderer;
/// Camera types.
pub mod camera;

pub use raytracer::*;
pub use material::*;
pub use primitives::*;
pub use light::*;
pub use gpu_renderer::{GpuRenderer, GpuRendererConfig};
pub use camera::*;
