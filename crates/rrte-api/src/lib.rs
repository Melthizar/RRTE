//! RRTE Engine Public API
//! 
//! This crate provides the public-facing API for the RRTE (Rust Raytracing Engine).
//! It re-exports the most commonly used types and functions from all engine modules.

// Re-export core functionality
pub use rrte_core::{Engine, Time, Events, Input, EngineConfig as CoreEngineConfig};
pub use rrte_scene::Scene;
pub use rrte_math::*;
pub use rrte_ecs::*;
pub use rrte_plugin::{Plugin, PluginManifest, PluginContext, PluginEvent};

// Re-export renderer types
pub use rrte_renderer::{
    Camera,
    Raytracer, RaytracerConfig, Material, LambertianMaterial, MetalMaterial, 
    DielectricMaterial, EmissiveMaterial, MaterialProperties
};

// Re-export asset management
pub use rrte_assets::*;

pub mod prelude {
    //! Common imports for RRTE applications
    
    pub use crate::{
        // Math
        Vec3, Vec4, Mat4, Quat, Transform, Color, Ray, AABB,
        
        // ECS
        Entity, Component, World,
        
        // Core
        Time, Engine, CoreEngineConfig,
        
        // Rendering
        Camera,
        Raytracer, RaytracerConfig,
        LambertianMaterial, MetalMaterial, DielectricMaterial, EmissiveMaterial,
        
        // Plugins
        Plugin, PluginManifest, PluginContext, PluginEvent,
        
        // Assets
        AssetManager, AssetHandle,
    };
}

/// Engine builder for easy setup
pub struct EngineBuilder {
    config: EngineConfig,
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub window_title: String,
    pub window_width: u32,
    pub window_height: u32,
    pub raytracer_config: RaytracerConfig,
    pub enable_plugins: bool,
    pub plugin_directories: Vec<String>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            window_title: "RRTE Game".to_string(),
            window_width: 800,
            window_height: 600,
            raytracer_config: RaytracerConfig::default(),
            enable_plugins: true,
            plugin_directories: vec!["plugins".to_string()],
        }
    }
}

impl EngineBuilder {
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
        }
    }

    pub fn window_title(mut self, title: impl Into<String>) -> Self {
        self.config.window_title = title.into();
        self
    }

    pub fn window_size(mut self, width: u32, height: u32) -> Self {
        self.config.window_width = width;
        self.config.window_height = height;
        self
    }

    pub fn raytracer_config(mut self, config: RaytracerConfig) -> Self {
        self.config.raytracer_config = config;
        self
    }

    pub fn enable_plugins(mut self, enable: bool) -> Self {
        self.config.enable_plugins = enable;
        self
    }

    pub fn plugin_directory(mut self, directory: impl Into<String>) -> Self {
        self.config.plugin_directories.push(directory.into());
        self
    }

    pub fn build(self) -> anyhow::Result<Engine> {
        let mut core_config = CoreEngineConfig::default();
        core_config.renderer_config = self.config.raytracer_config.clone();
        Engine::new(core_config)
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
