# RRTE Engine - Rust Raytracing Engine

A modular 3D raytracing-based game engine built in Rust, designed for extensibility and performance with both CPU and GPU raytracing capabilities.

> **🎉 CURRENT STATUS (December 2024)**:
> The RRTE engine now features a **fully functional GPU raytracing renderer** with realistic lighting! The engine has achieved the following milestones:
> - **✅ Complete GPU Raytracing Pipeline**: Full sphere intersection, surface normals, and physically-based lighting
> - **✅ Dynamic Lighting System**: Point lights with distance attenuation and orbital animation
> - **✅ GPU-Accelerated Rendering**: WGSL compute shaders for high-performance raytracing
> - **✅ Real-time Animation**: Animated orbiting light sources with adjustable brightness
> - **✅ Material System**: Lambertian materials with proper diffuse lighting calculations
> - **✅ Scene Management**: Dynamic scene updates with time-based animations
> - **✅ Stable Execution**: Zero panics, clean compilation, and smooth rendering

## Features

### 🚀 **Raytracing Capabilities**
- **GPU Raytracing**: High-performance compute shader-based raytracing using WGSL
- **CPU Raytracing**: Multi-threaded fallback with multi-sampling support
- **Sphere Intersection**: Accurate ray-sphere intersection with surface normal calculation
- **Physically-Based Lighting**: Realistic light attenuation and diffuse calculations
- **Dynamic Scenes**: Real-time scene updates and animations

### 💡 **Lighting System**
- **Point Lights**: Configurable intensity, color, and range
- **Distance Attenuation**: Physically-accurate light falloff
- **Ambient Lighting**: Subtle ambient contribution for realistic shadows
- **Orbital Animation**: Time-based light movement for dynamic scenes
- **Multiple Light Support**: Extensible lighting system for complex scenes

### 🏗️ **Architecture**
- **Modular Design**: Clean separation between core, rendering, scene, and math components
- **Async GPU Initialization**: Modern async/await patterns for GPU resource setup
- **Cross-Platform**: Windows, macOS, and Linux support via `wgpu`
- **Hot-Swappable Renderers**: Runtime switching between CPU and GPU modes
- **Memory Efficient**: Optimized buffer management and GPU data structures

## Project Structure

```text
RRTE/
├── crates/
│   ├── rrte-core/          # Engine core (lifecycle, camera, input, events)
│   ├── rrte-math/          # Mathematics library (vectors, matrices, colors)
│   ├── rrte-renderer/      # Rendering systems (CPU/GPU raytracers, materials)
│   ├── rrte-scene/         # Scene management (objects, lights, animations)
│   ├── rrte-ecs/           # Entity Component System
│   ├── rrte-assets/        # Asset management
│   ├── rrte-plugin/        # Plugin system
│   └── rrte-api/           # Public API
├── examples/               # Example projects
└── src/                    # Main executable with demo scene
```

## Getting Started

### Prerequisites

- **Rust**: Latest stable (1.75+ recommended)
- **GPU Drivers**: Updated drivers supporting Vulkan, DirectX 12, or Metal
- **Platform SDKs**: 
  - Windows: DirectX End-User Runtimes
  - macOS: Xcode Command Line Tools
  - Linux: Vulkan SDK

### Quick Start

```bash
# Clone the repository
git clone <repository-url>
cd RRTE

# Run the GPU raytracing demo
cargo run --release

# Or run with CPU raytracing (slower but useful for comparison)
# Edit src/main.rs and change RendererMode::Gpu to RendererMode::Cpu
cargo run --release
```

### Demo Scene

The current demo showcases:
- **Ground Sphere**: Large sphere acting as ground plane with gray Lambertian material
- **Center Sphere**: Red sphere demonstrating material properties and lighting
- **Orbital Light**: White point light orbiting the scene with realistic brightness
- **Dynamic Camera**: Positioned for optimal viewing of the lit scene

## Technical Implementation

### GPU Raytracing Pipeline

The GPU renderer uses a sophisticated compute shader pipeline:

1. **Ray Generation**: Per-pixel ray generation from camera parameters
2. **Scene Intersection**: Efficient sphere intersection testing
3. **Surface Calculation**: Normal vector computation at hit points
4. **Lighting Evaluation**: Multi-light diffuse lighting with attenuation
5. **Color Output**: Final pixel color with brightness clamping

### Key Components

#### WGSL Compute Shader (`raytrace.wgsl`)
```wgsl
// Physically-based lighting calculation
fn calculate_lighting(hit_point: vec3<f32>, normal: vec3<f32>, material: MaterialGpu) -> vec3<f32> {
    var total_light = vec3<f32>(0.0);
    
    // Ambient contribution
    let ambient = vec3<f32>(0.05) * material.color.rgb;
    total_light += ambient;
    
    // Per-light calculations with distance attenuation
    for (var i = 0u; i < arrayLength(&lights); i++) {
        let light = lights[i];
        let light_dir = light.position.xyz - hit_point;
        let distance = length(light_dir);
        
        // Realistic attenuation: intensity / (1 + linear*d + quadratic*d²)
        let attenuation = light.intensity / (1.0 + 0.1 * distance + 0.01 * distance * distance);
        let ndot_l = max(dot(normal, normalize(light_dir)), 0.0);
        
        total_light += material.color.rgb * light.color.rgb * ndot_l * attenuation * 0.3;
    }
    
    return clamp(total_light, vec3<f32>(0.0), vec3<f32>(1.0));
}
```

#### Dynamic Scene Updates
```rust
// Orbital light animation in Scene::update()
let orbit_radius = 8.0;
let orbit_speed = 0.5; // radians per second
let orbit_height = 8.0;

let new_x = orbit_radius * (self.time_accumulator * orbit_speed).cos();
let new_z = orbit_radius * (self.time_accumulator * orbit_speed).sin();
let new_position = Vec3::new(new_x, orbit_height, new_z);
```

### Performance Characteristics

- **Resolution**: 800x600 pixels (configurable)
- **Threading**: Compute shaders leverage GPU parallelism
- **Memory**: Efficient GPU buffer management
- **Rendering**: Real-time performance on modern GPUs
- **Scalability**: Easily extensible for additional objects and lights

## Architecture Deep Dive

### Renderer Abstraction

The engine supports multiple rendering backends through a clean abstraction:

```rust
pub enum RendererMode {
    Cpu,  // Multi-threaded CPU raytracer
    Gpu,  // WGPU compute shader raytracer
}
```

### GPU Data Structures

Carefully designed for WGSL compatibility:

```rust
#[repr(C)]
pub struct SphereGpu {
    pub center: [f32; 4],     // xyz + padding
    pub radius: f32,
    pub material_index: u32,
    pub _padding: [u32; 2],
}

#[repr(C)]
pub struct PointLightGpu {
    pub position: [f32; 4],   // xyz + padding
    pub color: [f32; 4],      // rgba
    pub intensity: f32,
    pub range: f32,
    pub _padding: [u32; 2],
}
```

### Scene Management

The `rrte-scene` crate provides:
- Dynamic object and light storage
- Time-based animation systems
- Efficient scene updates
- GPU data marshalling

## Usage Examples

### Basic Scene Creation

```rust
use rrte_renderer::primitives::Sphere;
use rrte_renderer::light::PointLight;
use rrte_renderer::material::LambertianMaterial;

// Create materials
let red_material = LambertianMaterial::new(Color::rgb(0.7, 0.3, 0.3));

// Create sphere with material
let sphere = Sphere::with_material(
    Vec3::new(0.0, 1.0, 0.0), 
    1.0, 
    red_material
);
scene.add_object(Arc::new(sphere));

// Add orbital light
let light = PointLight::new(
    Vec3::new(10.0, 10.0, 10.0),
    Color::white(),
    25.0  // intensity
);
scene.add_light(Arc::new(light));
```

### Custom Animation

```rust
impl Scene {
    pub fn update(&mut self, dt: f32) {
        self.time_accumulator += dt;
        
        // Custom animation logic here
        // Update object positions, light properties, etc.
    }
}
```

## Roadmap

### ✅ Completed Features
- [x] GPU raytracing pipeline with compute shaders
- [x] Sphere intersection and lighting
- [x] Point light system with orbital animation
- [x] Material system (Lambertian)
- [x] Real-time scene updates
- [x] Cross-platform GPU support via WGPU
- [x] Async renderer initialization
- [x] CPU/GPU renderer switching

### 🚧 In Progress
- [ ] Camera controls (orbit, zoom, pan)
- [ ] Multiple primitive types (planes, triangles)
- [ ] Texture mapping support
- [ ] Advanced material types (metallic, glass)

### 🔮 Future Enhancements
- [ ] **Advanced Lighting**: Area lights, environment maps, shadows
- [ ] **Acceleration Structures**: BVH for complex scenes
- [ ] **Post-Processing**: Tone mapping, bloom, anti-aliasing
- [ ] **Asset Pipeline**: glTF loading, texture streaming
- [ ] **Editor Tools**: Visual scene editor, material editor
- [ ] **Performance**: Multi-GPU support, temporal accumulation
- [ ] **Effects**: Volumetrics, subsurface scattering
- [ ] **VR Support**: OpenXR integration for immersive experiences

## Performance Notes

### GPU Requirements
- **Minimum**: DirectX 11/Vulkan 1.0/Metal 2.0 support
- **Recommended**: Modern discrete GPU (GTX 1060/RX 580 or better)
- **Optimal**: RTX/RDNA2+ with compute shader optimizations

### Optimization Tips
- Use `--release` for production builds (10x+ performance improvement)
- Adjust resolution in `GpuRendererConfig` for performance scaling
- Modify light intensity and count for quality/performance balance
- Enable V-Sync for smooth animation

## Contributing

We welcome contributions! Areas where help is needed:

- **Renderer Features**: Additional primitive types, advanced materials
- **Performance**: Optimization, profiling, benchmarking
- **Platform Support**: Testing on various GPU vendors and drivers
- **Documentation**: Tutorials, examples, API documentation
- **Tools**: Scene editors, asset converters, debugging utilities

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch
cargo install wgpu-info

# Run with hot reload during development
cargo watch -x run

# Check GPU capabilities
wgpu-info
```

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

---

**RRTE Engine** - Where Rust meets realistic raytracing. ⚡🦀✨
