# RRTE Engine - Rust Raytracing Engine

A modular 3D raytracing-based game engine built in Rust, designed for extensibility and performance.

> **⚠️ CURRENT STATUS**: The engine is in active development with some compilation errors that need to be resolved. See [Current Development Status](#current-development-status-june-3-2025) for details on what needs to be fixed next.

## Overview

RRTE is a modern game engine that leverages raytracing for realistic lighting and rendering. The engine is built with a modular architecture that allows for easy component swapping, plugin development, and API integration.

## Features

- **Modular Architecture**: Separated into focused crates for better maintainability
- **CPU & GPU Raytracing**: Both CPU-based raytracing with multi-sampling and GPU acceleration support
- **Entity Component System (ECS)**: Flexible entity management system with type-safe components
- **Plugin System**: Dynamic plugin loading and API support with manifest-based configuration
- **Material System**: Physically-based materials (Lambertian, Metal, Dielectric, Emissive)
- **Mathematics Library**: Comprehensive math utilities for 3D graphics built on GLM
- **Asset Management**: Efficient loading and management of game assets with handle-based system
- **Event System**: Comprehensive input handling and event management
- **Scene Management**: Hierarchical scene graph with transform support

## Project Structure

```text
RRTE/
├── crates/
│   ├── rrte-core/          # Core engine functionality (Engine, Time, Input, Events, Scene, Camera)
│   ├── rrte-math/          # Mathematical utilities (Vectors, Matrices, Rays, Transforms, Colors, Bounds)
│   ├── rrte-renderer/      # Raytracing system (CPU/GPU Raytracer, Materials, Primitives, Lighting)
│   ├── rrte-ecs/           # Entity Component System (Entities, Components, Queries)
│   ├── rrte-assets/        # Asset management (Loading, Caching, Handles)
│   ├── rrte-plugin/        # Plugin system (Dynamic loading, Manifests)
│   └── rrte-api/           # Public API for game development
├── examples/               # Example projects and demos (planned)
└── src/                    # Main engine integration and demos
```

## Architecture

### Core Components

- **rrte-math**: Complete vector math library with Vec2/3/4, matrices, rays, transforms, colors, bounds, and utility functions
- **rrte-core**: Engine lifecycle management, time system, input handling, event system, camera system, and scene management
- **rrte-renderer**: Full raytracing implementation with CPU and GPU backends, material system, primitive shapes, and lighting
- **rrte-ecs**: Type-safe Entity-Component-System with efficient component storage and query system
- **rrte-assets**: Asset loading and management with handle-based system, caching, and async loading support
- **rrte-plugin**: Dynamic plugin system with manifest-based configuration and runtime loading
- **rrte-api**: High-level unified API for game developers with convenient prelude module

### Implemented Features

#### Raytracing Renderer

- **CPU Raytracer**: Multi-threaded raytracing with configurable depth and sampling
- **GPU Raytracer**: WGPU-based compute shader acceleration (basic implementation)
- **Anti-aliasing**: Multi-sampling for smooth edges
- **Material Support**: Physically-based materials with proper light scattering
- **Primitive Shapes**: Spheres, planes, triangles, and mesh support
- **Lighting System**: Point lights, directional lights, area lights

#### Material System

- **Lambertian**: Diffuse materials for realistic matte surfaces
- **Metal**: Reflective materials with configurable roughness and fuzziness
- **Dielectric**: Glass and transparent materials with proper refraction (Snell's law)
- **Emissive**: Light-emitting materials for self-illuminated objects

#### Mathematics Library

- **Vectors**: Vec2, Vec3, Vec4 with comprehensive operations
- **Matrices**: Mat3, Mat4 for transformations and projections
- **Quaternions**: Rotation representation and interpolation
- **Rays**: Ray-object intersection testing
- **Transforms**: 3D transformations with translation, rotation, scale
- **Colors**: RGBA color handling with various color spaces
- **Bounds**: AABB (Axis-Aligned Bounding Boxes) for optimization

#### ECS System

- **Entities**: Unique identifiers for game objects
- **Components**: Type-safe data containers
- **Archetype Storage**: Efficient memory layout for component data
- **Query System**: Fast iteration over entities with specific components
- **Component Registration**: Runtime type registration system

#### Asset Management

- **Handle System**: Reference-counted asset handles
- **Async Loading**: Non-blocking asset loading with futures
- **Caching**: Automatic asset caching and deduplication
- **Multiple Formats**: Support for various asset types (textures, models, etc.)

#### Core Engine Features

- **Engine Lifecycle**: Proper initialization, update loop, and shutdown
- **Time Management**: Delta time, frame rate tracking, and timing utilities
- **Input System**: Keyboard, mouse, and gamepad input handling
- **Event System**: Type-safe event dispatch and handling
- **Camera System**: Perspective and orthographic projection support
- **Scene Management**: Hierarchical scene graph with transforms

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo
- For GPU features: DirectX 12 (Windows), Vulkan, or Metal (macOS)

### Building the Engine

```powershell
# Clone the repository
git clone <repository-url>
cd RRTE

# Build all crates
cargo build

# Run tests
cargo test

# Build with optimizations
cargo build --release

# Check for compilation errors
cargo check
```

### Quick Start

Run the basic demo to see the engine in action:

```powershell
# Run the main demo
cargo run

# Run with release optimizations for better performance
cargo run --release
```

## Usage

### Basic Engine Setup

```rust
use rrte_api::prelude::*;
use rrte_core::Engine;
use rrte_renderer::RaytracerConfig;

fn main() {
    // Initialize the engine
    let mut engine = Engine::new();
    
    // Configure the raytracer
    let config = RaytracerConfig {
        max_depth: 50,
        samples_per_pixel: 100,
        width: 800,
        height: 600,
        background_color: Color::new(0.5, 0.7, 1.0, 1.0),
    };
    
    // Create a scene with objects
    let mut scene = Scene::new();
    
    // Add a sphere
    let sphere = Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5);
    let material = LambertianMaterial::new(Color::rgb(0.7, 0.3, 0.3));
    scene.add_object(sphere, material);
    
    // Create a camera
    let camera = Camera::new_perspective(
        45.0_f32.to_radians(), // FOV
        16.0 / 9.0,           // Aspect ratio
        0.1,                  // Near plane
        100.0                 // Far plane
    );
    
    // Main loop
    loop {
        engine.update();
        
        if engine.should_close() {
            break;
        }
    }
}
```

### ECS Usage

```rust
use rrte_ecs::*;

// Define components
#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Velocity(Vec3);

// Create entity with components
let mut world = World::new();
let entity = world.spawn()
    .with(Position(Vec3::new(0.0, 0.0, 0.0)))
    .with(Velocity(Vec3::new(1.0, 0.0, 0.0)))
    .build();

// Query entities
for (pos, vel) in world.query::<(&mut Position, &Velocity)>() {
    pos.0 += vel.0 * delta_time;
}
```

### Working with Assets

```rust
use rrte_assets::*;

// Load an asset
let asset_manager = AssetManager::new();
let texture_handle: Handle<Texture> = asset_manager.load("textures/brick.png");

// Use the asset when it's loaded
if let Some(texture) = asset_manager.get(&texture_handle) {
    // Use the texture
}
```

### Plugin Development

```rust
use rrte_plugin::*;

// Define a plugin
#[derive(Default)]
struct MyGamePlugin {
    initialized: bool,
}

impl Plugin for MyGamePlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            name: "MyGamePlugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Example game plugin".to_string(),
            author: "Your Name".to_string(),
            dependencies: vec![],
        }
    }
    
    fn initialize(&mut self, engine: &mut Engine) -> anyhow::Result<()> {
        log::info!("Initializing MyGamePlugin");
        self.initialized = true;
        Ok(())
    }
    
    fn update(&mut self, engine: &mut Engine, delta_time: f32) -> anyhow::Result<()> {
        // Per-frame update logic
        Ok(())
    }
    
    fn shutdown(&mut self, engine: &mut Engine) -> anyhow::Result<()> {
        log::info!("Shutting down MyGamePlugin");
        Ok(())
    }
}

// Load the plugin
let mut plugin_manager = PluginManager::new();
plugin_manager.register_plugin(Box::new(MyGamePlugin::default()))?;
```

## Configuration

The engine supports various configuration options:

```rust
let config = RaytracerConfig {
    max_depth: 50,           // Maximum ray bounce depth
    samples_per_pixel: 100,  // Anti-aliasing samples
    width: 1920,             // Render width
    height: 1080,            // Render height
    background_color: Color::new(0.5, 0.7, 1.0, 1.0), // Sky color
};
```

## Performance

- **Parallel Rendering**: Utilizes Rayon for multi-threaded ray processing
- **Optimized Math**: Built on GLM for high-performance vector operations
- **Memory Efficient**: ECS design minimizes memory overhead
- **Configurable Quality**: Adjustable sample counts and ray depth

## Development Status

### ✅ Completed

- **Core Math Library**: Full vector math, matrices, rays, transforms, colors, bounds
- **CPU Raytracing Renderer**: Multi-threaded raytracing with material support
- **Material System**: Lambertian, Metal, Dielectric, and Emissive materials
- **ECS Foundation**: Entity management, component storage, query system
- **Asset Management System**: Handle-based asset loading with caching
- **Plugin System**: Dynamic plugin loading with manifest support
- **Core Engine**: Engine lifecycle, time management, input handling
- **Event System**: Type-safe event dispatch and handling
- **Camera System**: Perspective and orthographic projections
- **Scene Management**: Hierarchical scene organization

### 🚧 In Progress

- **GPU Raytracing**: WGPU-based compute shader implementation (basic version complete)
- **Primitive Shapes**: Expanding beyond spheres to triangles, meshes, etc.
- **Lighting System**: Advanced light types and shadow support
- **Examples and Documentation**: Comprehensive examples and tutorials

### ⏳ Planned

- **Scene Serialization**: Save/load scene files in various formats
- **Animation System**: Keyframe animation and skeletal animation
- **Audio System**: 3D spatial audio with sound effects and music
- **Physics Integration**: Collision detection and rigid body dynamics
- **Editor Tools**: Visual scene editor and asset pipeline
- **Advanced Rendering**: Volumetric rendering, denoising, global illumination

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

This project is licensed under the MIT OR Apache-2.0 license.

## Dependencies

The engine is built on top of carefully selected, high-quality Rust crates:

### Core Dependencies

- **glam**: Fast 3D math library with SIMD optimizations
- **wgpu**: Modern graphics API abstraction for cross-platform GPU access
- **winit**: Cross-platform windowing and event handling
- **rayon**: Data parallelism for multi-threaded raytracing
- **serde**: Serialization framework for assets and configuration
- **anyhow**: Ergonomic error handling throughout the engine
- **thiserror**: Custom error types for better error reporting

### Async and Concurrency

- **tokio**: Async runtime for non-blocking asset loading
- **rayon**: Parallel processing for raytracing and compute-heavy tasks

### Plugin System

- **libloading**: Dynamic library loading for plugin system
- **toml**: Configuration file parsing for plugin manifests

### Math and Linear Algebra

- **nalgebra**: Additional linear algebra utilities where needed
- **bytemuck**: Safe transmutation for GPU buffer management

### Development and Debugging

- **log**: Logging facade for debugging and development
- **env_logger**: Environment-based log configuration

## Performance Characteristics

- **Multi-threaded Raytracing**: Scales with CPU core count using Rayon
- **SIMD Optimizations**: GLM provides vectorized math operations
- **Memory Efficient ECS**: Archetype-based storage for optimal cache usage
- **Asset Caching**: Prevents duplicate loading and reduces memory usage
- **Configurable Quality**: Trade quality for performance with adjustable sampling
- **GPU Acceleration**: Optional GPU compute shaders for enhanced performance

## Examples and Demos

### Current Examples

- **Basic Demo**: Simple raytraced scene with multiple materials (`cargo run`)
- **Material Showcase**: Demonstrates different material types and properties
- **ECS Example**: Shows entity-component system usage
- **Plugin Example**: Demonstrates plugin development and loading

### Planned Examples

- **Advanced Lighting**: Complex lighting setups with multiple light sources
- **Animation Demo**: Keyframe animation and interpolation
- **Physics Integration**: Collision detection and rigid body simulation
- **Performance Benchmark**: Raytracing performance testing and optimization

## Roadmap

### Version 0.1.0 (Current)

- ✅ Core architecture and modular design
- ✅ CPU raytracing with basic materials
- ✅ ECS system foundation
- ✅ Plugin system framework
- ✅ Asset management basics

### Version 0.2.0 (Next Release)

- 🚧 GPU raytracing acceleration
- 🚧 Extended primitive shapes (triangles, meshes)
- 🚧 Advanced lighting models
- 🚧 Scene serialization format
- 🚧 Comprehensive examples and documentation

### Version 0.3.0 (Future)

- ⏳ Animation system
- ⏳ Physics integration
- ⏳ Audio system
- ⏳ Performance optimizations
- ⏳ Editor tools foundation

### Version 1.0.0 (Long Term)

- ⏳ Complete feature set
- ⏳ Production-ready stability
- ⏳ Comprehensive documentation
- ⏳ Platform-specific optimizations
- ⏳ Community ecosystem

## Current Development Status (June 3, 2025)

### 🚨 **WHERE WE LEFT OFF - BUILD ERRORS TO FIX**

The engine is currently in a non-compilable state due to several API mismatches that need to be resolved:

#### **Critical Compilation Errors:**

1. **Transform API Mismatches** (Priority: HIGH)
   - **File**: `crates/rrte-renderer/src/camera.rs`
   - **Issues**:
     - `transform.matrix()` should be `transform.to_matrix()`
     - `transform.translation()` property access needs verification
     - `transform.rotation()` property access needs verification
   - **Action**: Check Transform struct API in `crates/rrte-math/src/transform.rs` and update camera.rs accordingly

2. **Camera Method Name Mismatch** (Priority: HIGH)
   - **File**: `crates/rrte-renderer/src/raytracer.rs:67`
   - **Issue**: `camera.screen_to_ray()` should be `camera.generate_ray()`
   - **Action**: Update method call or implement missing method

3. **Missing Quaternion Constructors** (Priority: MEDIUM)
   - **File**: `crates/rrte-renderer/src/camera.rs:91`
   - **Issue**: `Quat::from_rotation_arc()` may not exist
   - **Action**: Check glam quaternion API and use correct constructor

#### **Fixed Issues (Completed):**

✅ **Cyclic Dependencies**: Removed `rrte-core` dependency from `rrte-renderer`  
✅ **HitInfo Constructor**: Fixed all `HitInfo::new()` parameter order issues  
✅ **Asset Loader Traits**: Added proper `Send + Sync` bounds  
✅ **Material Debug Traits**: Added Debug implementation for all materials  
✅ **Primitive Serialization**: Removed problematic Serialize/Deserialize derives  
✅ **Lighting Integration**: Fixed light contribution calculations in raytracer  

#### **Immediate Next Steps:**

1. **Check Transform API**:

   ```powershell
   # Examine the Transform struct definition
   code crates/rrte-math/src/transform.rs
   ```

2. **Fix Transform Usage**:
   - Update `transform.matrix()` → `transform.to_matrix()` (or correct method)
   - Verify property access patterns for translation/rotation

3. **Check Camera API**:

   ```powershell
   # Check if screen_to_ray exists or needs to be implemented
   code crates/rrte-core/src/camera.rs
   ```

4. **Verify Build**:

   ```powershell
   cargo check
   ```

#### **Current Architecture Status:**

- **Math Library**: ✅ Complete (Vec3, Mat4, Transform, Ray, Color, etc.)
- **ECS System**: ✅ Functional (Entity, Component, World, Query)
- **Asset Management**: ✅ Working (Handle, Loader, Manager)
- **Plugin System**: ✅ Implemented (Dynamic loading, Manifests)
- **Renderer Core**: 🚧 **BLOCKED** (Compilation errors preventing testing)
- **Materials**: ✅ Complete (Lambertian, Metal, Dielectric, Emissive)
- **Primitives**: ✅ Complete (Sphere, Plane, Triangle, Cube)
- **Lighting**: ✅ Complete (Point, Directional, Area lights)

#### **Development Environment:**

- **Platform**: Windows with PowerShell
- **Rust Version**: Latest stable
- **Target**: CPU raytracing with multi-threading
- **Graphics**: WGPU for future GPU acceleration

#### **To Resume Development:**

1. Fix the Transform API mismatches in camera.rs
2. Resolve the Camera method naming issue
3. Run `cargo check` to verify compilation
4. Test the basic raytracing demo with `cargo run`
5. Continue with GPU raytracing implementation

The foundation is solid - just need to resolve these API compatibility issues to get back to working raytracing!
