//! # RRTE Engine
//! 
//! A modular 3D raytracing-based game engine built in Rust, featuring advanced primitive 
//! modification, CSG operations, procedural deformations, GPU raytracing, and dynamic scene animation.
//! 
//! ## Quick Start
//! 
//! ```rust,no_run
//! use rrte_engine::prelude::*;
//! use anyhow::Result;
//! 
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create engine configuration
//!     let config = EngineConfig {
//!         renderer_mode: RendererMode::Gpu,
//!         target_fps: 60.0,
//!         ..Default::default()
//!     };
//!     
//!     // Create and initialize engine
//!     let mut engine = Engine::new(config)?;
//!     engine.initialize_core_systems()?;
//!     
//!     // Set up your scene
//!     let scene = engine.scene_mut();
//!     // ... add objects, lights, etc.
//!     
//!     // Initialize renderer (for applications with windows)
//!     engine.initialize_renderer(Some(window)).await?;
//!     
//!     // Run your game loop
//!     while engine.is_running() {
//!         engine.render_frame()?;
//!     }
//!     
//!     Ok(())
//! }
//! ```

// Re-export all core engine functionality
pub use rrte_core::*;
pub use rrte_math as math;
pub use rrte_renderer as renderer;
pub use rrte_scene as scene;
pub use rrte_ecs as ecs;
pub use rrte_assets as assets;
pub use rrte_plugin as plugin;
pub use rrte_api as api;

/// Convenience prelude that brings common types into scope
pub mod prelude {
    // Core engine types
    pub use rrte_core::{Engine, EngineConfig, RendererMode};
    
    // Math types
    pub use rrte_math::{Vec3, Color, Transform, Mat4};
    
    // Renderer types
    pub use rrte_renderer::{
        material::{Material, LambertianMaterial},
        light::PointLight,
        camera::{Camera, ProjectionType},
        primitives::{Sphere, Cube, Cylinder, Cone, Capsule, Plane, Triangle},
        raytracer::RaytracerConfig,
        gpu_renderer::GpuRendererConfig,
    };
    
    // Scene management
    pub use rrte_scene::Scene;
    pub use rrte_renderer::SceneObject;
    
    // Common std types for convenience
    pub use std::sync::Arc;
    pub use anyhow::Result;
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ENGINE_NAME: &str = "RRTE Engine";

/// Engine capabilities and feature flags
pub mod features {
    /// Whether GPU raytracing is enabled
    pub const GPU_RAYTRACING: bool = cfg!(feature = "gpu");
    
    /// Whether CPU raytracing is enabled  
    pub const CPU_RAYTRACING: bool = true;
    
    /// Whether SDF support is enabled
    pub const SDF_SUPPORT: bool = true;
    
    /// Whether deformation support is enabled
    pub const DEFORMATION_SUPPORT: bool = true;
    
    /// Whether plugin system is enabled
    pub const PLUGIN_SYSTEM: bool = cfg!(feature = "plugins");
} 