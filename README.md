# RRTE Engine - Rust Raytracing Engine

A modular 3D raytracing-based game engine built in Rust, designed for extensibility and performance.

> **✅ CURRENT STATUS (June 5, 2025)**:
> The `rrte-scene` crate has been created and integrated. The basic GPU raytracing path via compute shaders in `GpuRenderer` is now **functional and displays output**:
> - **GPU Data Structures**: `CameraGpu`, `SphereGpu`, `MaterialGpu` in Rust and WGSL are used. WGPU buffers are in place.
> - **WGSL Compute Shader**: `raytrace.wgsl` generates rays based on camera parameters and outputs ray direction as color to an `Rgba8Unorm` texture.
> - **WGPU Pipeline**: `GpuRenderer` correctly dispatches the compute shader.
> - **Blit Pass for Display**: A render pipeline (`blit.wgsl` and associated Rust code) in `GpuRenderer` copies the `Rgba8Unorm` output of the compute shader to the swap chain texture. This pass handles potential RGBA vs. BGRA format differences (e.g., if the swap chain is `Bgra8UnormSrgb`) by swizzling channels in the fragment shader. The vertex shader uses a `switch` statement to avoid dynamic indexing, resolving previous validation errors.
> - **Build System**: Dependencies between `rrte-core`, `rrte-renderer`, and `rrte-scene` are established. The `wgpu` "macros" feature issue was resolved. Cyclic dependencies were broken. Texture format issues for storage and copying have been iteratively addressed.
> - **Application Runs**: The application now compiles and runs without panics, displaying the output of the `raytrace.wgsl` compute shader via the blit pass.
>
> **Next immediate goal**: Verify the visual output of the compute shader (ray directions) is correct. Then, simplify the demo scene in `src/main.rs` and implement basic sphere intersection in the `raytrace.wgsl` compute shader.

## Overview

RRTE is a modern game engine that leverages raytracing for realistic lighting and rendering. The engine is built with a modular architecture that allows for easy component swapping, plugin development, and API integration. It supports both CPU-based raytracing and a foundational GPU-based path using `wgpu`.

## Features

- **Modular Architecture**: Separated into focused crates.
- **Switchable Renderers**:
    - **CPU Raytracer**: Multi-threaded raytracing with multi-sampling.
    - **GPU Raytracer**: Foundational `wgpu` integration, ready for compute shader-based raytracing.
- **Windowing and Display**: Integrated with `winit` for window management. `pixels` crate is used for CPU framebuffer display.
- **Async Initialization**: Uses `tokio` for asynchronous GPU renderer setup.
- **Entity Component System (ECS)**: Flexible entity management system (foundational).
- **Plugin System**: Dynamic plugin loading (foundational).
- **Material System**: Physically-based materials (Lambertian, etc.). Objects directly manage materials.
- **Mathematics Library**: Comprehensive math utilities.
- **Asset Management**: Foundational asset loading and management.
- **Event System**: Input handling via `winit`.
- **Scene Management**: Basic scene graph support.

## Project Structure

```text
RRTE/
├── crates/
│   ├── rrte-core/          # Core engine (Engine, Time, Input, Events, Scene, Camera, RendererMode selection)
│   ├── rrte-math/          # Math utilities
│   ├── rrte-renderer/      # Raytracing (CPU Raytracer, GPU Renderer shell, Materials, Primitives)
│   ├── rrte-ecs/           # ECS
│   ├── rrte-assets/        # Asset management
│   ├── rrte-plugin/        # Plugin system
│   └── rrte-api/           # Public API
├── examples/               # Planned
└── src/                    # Main engine executable (main.rs) with winit/tokio integration
```

## Architecture

### Core Components

- **rrte-core**: Engine lifecycle, `RendererMode` (CPU/GPU) selection, time system, input handling, event system, camera, scene management. Manages `ActiveRenderer` (either CPU or GPU).
- **rrte-renderer**: Contains both `Raytracer` (CPU) and `GpuRenderer` (WGPU based, for compute shaders). Includes materials, primitives, lighting.

### Implemented Features (Highlights)

#### Raytracing & Rendering
- **CPU Raytracer**: Functional, multi-threaded, multi-sampling, renders to window via `pixels`.
- **GPU Renderer Shell**: `wgpu` context initialized, clears screen. Ready for compute shader implementation.
- **Switchable Backend**: `EngineConfig` allows selection between `RendererMode::Cpu` and `RendererMode::Gpu`.
- **Async GPU Init**: GPU renderer initialization is `async` and managed by `tokio` in `main.rs`.

#### Windowing & Core Engine
- **Window Management**: `winit` for window creation and event loop.
- **CPU Framebuffer Display**: `pixels` crate for CPU path.
- **GPU Presentation**: `GpuRenderer` will handle its own presentation via `wgpu` surface.

## Getting Started

### Prerequisites

- Rust (latest stable recommended, e.g., 1.78+)
- Cargo
- For GPU features (future): Vulkan SDK, DirectX End-User Runtimes, or Metal development tools.

### Building and Running

```powershell
# Clone the repository
git clone <repository-url>
cd RRTE

# Build all crates and run the main demo
cargo run

# Run with release optimizations for better performance
cargo run --release

# Build without running
cargo build

# Check for compilation errors and warnings
cargo check

# Run tests
cargo test
```
The main demo is in `src/main.rs`.

### Running the Demo

By default, `src/main.rs` is now configured to attempt to run in `RendererMode::Gpu`. Initially, this will show a clear color from the `GpuRenderer`.
To run the CPU raytracer:
1. Open `src/main.rs`.
2. Find the `EngineConfig` initialization.
3. Change `renderer_mode: RendererMode::Gpu` to `renderer_mode: RendererMode::Cpu`.
4. Run `cargo run`.

## Usage

### Basic Engine Structure (Illustrative - see `src/main.rs` for current example)

The `Engine` now supports an async initialization path for the renderer, especially for GPU mode.

```rust
// src/main.rs (Conceptual - Refer to actual file for details)
use rrte_core::{Engine, EngineConfig, RendererMode};
use rrte_renderer::{RaytracerConfig, GpuRendererConfig /* ... */};
// ... winit, tokio, etc.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine_config = EngineConfig {
        renderer_mode: RendererMode::Gpu, // or RendererMode::Cpu
        renderer_config: RaytracerConfig::default(),      // For CPU path or initial sizing
        gpu_renderer_config: GpuRendererConfig::default(),// For GPU path
        // ... other configs
    };

    let mut engine = Engine::new(engine_config)?;
    engine.initialize_core_systems()?; // Initializes non-renderer systems
    
    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new().build(&event_loop)?);

    // Renderer initialization is now separate and async
    engine.initialize_renderer(Some(window.clone())).await?;
    
    // create_demo_scene(&mut engine)?;

    let mut pixels: Option<Pixels> = None;
    if engine.config().renderer_mode == RendererMode::Cpu {
        // Setup pixels for CPU rendering path
    }

    event_loop.run(move |event, elwt| {
        // ... event handling ...
        match event {
            Event::AboutToWait => {
                engine.render_frame().unwrap(); // Calls CPU or GPU render internally

                if engine.config().renderer_mode == RendererMode::Cpu {
                    if let (Some(p), Some(fb)) = (pixels.as_mut(), engine.get_frame_buffer()) {
                        p.frame_mut().copy_from_slice(fb);
                        p.render().unwrap();
                    }
                } // GPU renderer presents internally
                window.request_redraw();
            }
            _ => {}
        }
    })?;
    Ok(())
}
```

## Roadmap / To-Do List

### Immediate Next Steps
-   [x] **Integrate GpuRenderer Shell**: `Engine` can now initialize and use `GpuRenderer` to clear the screen. (`pixels` crate bypassed for GPU path). `tokio` integrated for async init.
-   [x] **Resolve `rrte-scene` Path Issue**: Fix the compilation error related to the missing `rrte-scene/Cargo.toml`. (Created and integrated)
-   [x] **GPU Raytracing - Phase 1 (Basic Compute & Display)**:
    -   [x] **Define GPU Data Structures**: Rust and WGSL structs for Camera, Sphere, Material defined. WGPU Buffers created.
    -   [x] **Write WGSL Shaders (Minimal - Ray Generation & Blit)**:
        -   `raytrace.wgsl`: Generates a ray per pixel/invocation. Colors pixel by ray direction.
        -   `blit.wgsl`: Fullscreen triangle pass to copy/swizzle `raytrace.wgsl` output to swapchain. Shader validation errors fixed.
        -   [ ] `intersection.wgsl` / Update `raytrace.wgsl`: Simple sphere intersection logic.
        -   [ ] `shading.wgsl` / Update `raytrace.wgsl`: Basic shading (e.g., object color or normal visualization post-intersection).
    -   [x] **Setup Compute Pipeline**: In `GpuRenderer`, shaders loaded, WGPU buffers, bind groups, and compute pipeline created.
    -   [x] **Setup Blit Pipeline**: In `GpuRenderer`, blit shader, pipeline, and resources created.
    -   [x] **Dispatch & Output**: Compute shader dispatched. Output texture rendered to swapchain via blit pass.
-   [ ] **Verify Visual Output**: Confirm the ray-direction visualization from the compute shader is working as expected.
-   [ ] **Simplify Demo Scene**: Modify `src/main.rs create_demo_scene` for a ground plane + optional simple object. This scene data will then be used for the GPU buffers.
-   [ ] **Camera Controls**: Implement basic camera controls (e.g., orbit, zoom) via `winit` input. Update camera data for GPU buffers.
-   [ ] **Code Cleanup**: Address compiler warnings (`cargo fix`), improve `unwrap()` handling.
-   [ ] **Resolve Exit Code**: Investigate `0xcfffffff` exit code on Windows (if still present after current changes).

### Core Engine & Renderer Enhancements
-   [ ] **GPU Raytracing - Phase 2 (Core Features)**:
    -   [ ] Scene data marshalling from `Engine::Scene` to GPU buffers.
    -   [ ] Basic material properties (albedo from `LambertianMaterial`) in shaders.
    -   [ ] Point light support in shaders.
    -   [ ] Multi-sampling / accumulation on GPU.
    -   [ ] BVH generation (CPU-side for now) and traversal on GPU for spheres.
-   [ ] **GPU Raytracing - Phase 3 (Advanced Features & Optimizations)**:
    -   [ ] Advanced materials (textures, PBR properties) in shaders.
    -   [ ] More light types, shadows.
    -   [ ] Explore more advanced WGPU features (e.g., bindless textures, push constants).
-   [ ] **CPU Raytracer Optimizations**: BVH, SIMD (if not already present from earlier plans).

### Scene & Asset Management
-   [ ] **Scene Graph**: Implement a full hierarchical scene graph in `rrte-core` or `rrte-scene`.
    -   [ ] Parent-child relationships.
    -   [ ] Local and world transforms.
-   [ ] **Asset Loading**:
    -   [ ] Robust glTF 2.0 loader (meshes, materials, textures, scene hierarchy).
    -   [ ] OBJ loader (meshes, basic materials).
    -   [ ] Image loading for textures (PNG, JPG, etc. - expand on current `image` crate usage).
    -   [ ] Material definition files.
-   [ ] **Asset Pipeline**: Tools or processes for converting and optimizing assets.

### ECS & Plugin System
-   [ ] **ECS Integration**:
    -   [ ] Deeper integration of ECS with rendering and game logic.
    -   [ ] More examples and use-cases for `rrte-ecs`.
    -   [ ] Systems for physics, animation, game logic.
-   [ ] **Plugin System Development**:
    -   [ ] Hot reloading for plugins.
    -   [ ] More extensive API for plugins to interact with the engine.
    -   [ ] Example plugins (e.g., custom renderer, physics).

### User Interface & Tools
-   [ ] **In-Engine GUI**:
    -   [ ] Integrate `egui` or `Dear ImGui` for debugging and editor-like functionality.
    -   [ ] Display performance metrics, scene hierarchy, material properties.
-   [ ] **Editor**:
    -   [ ] Long-term goal: A simple scene editor built using the engine and a GUI library.

### Miscellaneous
-   [ ] **Physics Integration**: Integrate a 2D/3D physics library (e.g., `Rapier`, `nphysics`).
-   [ ] **Cross-Platform Testing**: Rigorous testing on Windows, macOS, and Linux.
-   [ ] **Documentation**:
    -   [ ] Detailed API documentation for all crates (`cargo doc --open`).
    -   [ ] Tutorials and guides for using the engine.
-   [ ] **Examples**: Create more example projects in the `examples/` directory showcasing different features.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details (assuming a LICENSE.md exists or will be added).
