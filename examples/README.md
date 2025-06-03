# RRTE Engine Examples

This directory contains example applications that demonstrate various features of the RRTE Engine. Each example is a standalone application that uses the RRTE Engine as a library dependency.

## Available Examples

### 1. Basic Demo (`basic-demo`)
**File**: `examples/basic-demo/`
**Description**: Demonstrates fundamental RRTE Engine usage with simple scene creation, basic lighting, and material setup.

**Features Shown**:
- Engine initialization and configuration
- Basic scene creation with spheres
- Material system (Lambertian materials)
- Point lighting
- Camera setup and positioning
- Basic render loop

**Run with**:
```bash
cargo run --bin basic-demo --release
```

### 2. Advanced Demo (`advanced-demo`)
**File**: `examples/advanced-demo/`
**Description**: Showcases advanced engine features including animated scenes, atmospheric lighting, and real-time camera movement.

**Features Shown**:
- Animated spooky atmospheric scene
- Orbiting camera system
- Dynamic lighting with multiple colored lights
- Dark, moody material setup
- Real-time animation systems
- Advanced scene composition

**Run with**:
```bash
cargo run --bin advanced-demo --release
```

### 3. SDF Showcase (`sdf-showcase`)
**File**: `examples/sdf-showcase/`
**Description**: Demonstrates the engine's advanced geometric modeling capabilities including SDFs, CSG operations, and procedural deformations.

**Features Shown**:
- Signed Distance Field (SDF) concepts
- Constructive Solid Geometry (CSG) operations
- Procedural deformation systems
- Complex object creation patterns
- Advanced geometric modeling

**Run with**:
```bash
cargo run --bin sdf-showcase --release
```

## How to Run Examples

### Prerequisites
- Rust 1.75+ (latest stable recommended)
- GPU with Vulkan/DirectX 12/Metal support
- Updated graphics drivers

### Quick Start
From the root directory of the RRTE project:

```bash
# Run the basic demo
cargo run --bin basic-demo --release

# Run the advanced demo with animation
cargo run --bin advanced-demo --release

# Run the SDF showcase
cargo run --bin sdf-showcase --release
```

### Development Mode
For faster compilation during development (but slower runtime):
```bash
cargo run --bin basic-demo
```

## Example Architecture

Each example follows the same architectural pattern:

1. **Engine Configuration**: Set up renderer, resolution, and performance settings
2. **Engine Initialization**: Create and initialize core engine systems
3. **Window Creation**: Create platform window using `winit`
4. **Renderer Setup**: Initialize GPU or CPU renderer with window handle
5. **Scene Creation**: Build scene with objects, materials, and lighting
6. **Camera Setup**: Position and configure camera
7. **Render Loop**: Handle events and render frames continuously

## Using RRTE Engine in Your Own Projects

These examples demonstrate the proper way to use RRTE Engine as a library:

```toml
# In your Cargo.toml
[dependencies]
rrte-engine = { path = "path/to/rrte-engine" }
winit = "0.29.15"
pixels = "0.15.0"  # For CPU rendering fallback
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
anyhow = "1.0"
```

```rust
// In your main.rs
use rrte_engine::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Configure engine
    let config = EngineConfig {
        renderer_mode: RendererMode::Gpu,
        target_fps: 60.0,
        ..Default::default()
    };
    
    // Create and initialize
    let mut engine = Engine::new(config)?;
    engine.initialize_core_systems()?;
    
    // Create window and initialize renderer
    // ... window setup code ...
    engine.initialize_renderer(Some(window)).await?;
    
    // Build your scene
    let scene = engine.scene_mut();
    // ... add objects, lights, etc ...
    
    // Run your application
    // ... event loop ...
    
    Ok(())
}
```

## Learning Path

1. **Start with `basic-demo`** - Learn fundamental engine usage
2. **Explore `advanced-demo`** - Understand animation and advanced features  
3. **Study `sdf-showcase`** - See advanced geometric modeling capabilities
4. **Build your own** - Create custom applications using the engine

## Performance Notes

- Use `--release` flag for optimal performance
- GPU mode provides better performance than CPU mode
- Adjust `samples_per_pixel` in `RaytracerConfig` to balance quality vs performance
- Monitor console output for performance statistics and warnings

## Troubleshooting

### Common Issues

**Compilation Errors**: Ensure you're in the root RRTE directory and all dependencies are available.

**Graphics Issues**: Update GPU drivers and verify Vulkan/DirectX 12/Metal support.

**Performance Issues**: Try reducing resolution or samples_per_pixel in the renderer config.

**Window Issues**: Check that your platform supports the required windowing features.

For more help, see the main RRTE Engine documentation or create an issue on the project repository. 