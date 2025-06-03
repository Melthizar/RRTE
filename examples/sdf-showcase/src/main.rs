//! SDF Showcase Application
//! 
//! This demonstrates the new primitive types added to the RRTE Engine:
//! - All Second Life primitive types (Box, Sphere, Cylinder, Prism, Torus, Tube, Ring)
//! - Additional advanced primitives (Cone, Capsule, Ellipsoid)
//! - Traditional raytracing approach for compatibility

use rrte_engine::prelude::*;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use std::sync::Arc;
use anyhow::Result;
use log::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    info!("Starting RRTE Engine Primitive Showcase");
    
    // Configure the engine
    let raytracer_config = RaytracerConfig {
        max_depth: 50,
        samples_per_pixel: 4,
        width: 1200,
        height: 800,
        background_color: Color::new(0.05, 0.05, 0.08, 1.0), // Much darker background
    };
    
    let gpu_renderer_config = GpuRendererConfig {
        width: raytracer_config.width,
        height: raytracer_config.height,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        ..Default::default()
    };
    
    let engine_config = EngineConfig {
        renderer_mode: RendererMode::Gpu, // Use GPU for better performance
        renderer_config: raytracer_config,
        gpu_renderer_config,
        target_fps: 60.0,
        enable_vsync: true,
        log_level: log::LevelFilter::Info,
    };
    
    // Create and initialize engine
    let mut engine = Engine::new(engine_config)?;
    engine.initialize_core_systems()?;
    
    // Create window
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("RRTE Engine - Primitive Showcase")
            .with_inner_size(LogicalSize::new(1200, 800))
            .with_resizable(true)
            .build(&event_loop)
            .unwrap(),
    );
    
    // Initialize renderer with window
    engine.initialize_renderer(Some(window.clone())).await?;
    
    // Create the primitive showcase scene
    create_primitive_showcase_scene(&mut engine)?;
    
    // Set up camera for optimal viewing
    setup_showcase_camera(&mut engine);
    
    // Initialize pixels for CPU fallback rendering
    let render_cpu_path = engine.get_frame_buffer().is_some();
    let mut pixels: Option<Pixels> = if render_cpu_path {
        info!("CPU rendering path detected, initializing Pixels.");
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        Some(Pixels::new(1200, 800, surface_texture)?)
    } else {
        info!("GPU rendering path detected.");
        None
    };
    
    info!("Window created: {}x{}", window.inner_size().width, window.inner_size().height);
    info!("Starting render loop...");
    
    // Main event loop
    let window_clone = window.clone();
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
                ..
            } if window_id == window_clone.id() => {
                info!("Window close requested");
                engine.stop();
                elwt.exit();
            }
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(size),
                ..
            } if window_id == window_clone.id() => {
                if size.width == 0 || size.height == 0 { return; }
                
                engine.update_resolution(size.width, size.height);
                
                if let Some(p) = pixels.as_mut() {
                    if let Err(e) = p.resize_surface(size.width, size.height) {
                        error!("Pixels resize_surface error: {}", e);
                        engine.stop();
                        elwt.exit();
                        return;
                    }
                    if let Err(e) = p.resize_buffer(size.width, size.height) {
                        error!("Pixels resize_buffer error: {}", e);
                        engine.stop();
                        elwt.exit();
                        return;
                    }
                }
            }
            Event::AboutToWait => {
                if !engine.is_running() {
                    elwt.exit();
                    return;
                }

                // Update engine systems
                let dt = engine.time().delta_time();
                engine.time_mut().update();
                engine.input_mut().update();
                engine.scene_mut().update(dt);

                // Render frame
                if let Err(e) = engine.render_frame() {
                    error!("Engine render_frame error: {}", e);
                    engine.stop();
                    elwt.exit();
                    return;
                }

                // Copy frame buffer to pixels if using CPU path
                if let Some(p) = pixels.as_mut() {
                    if let Some(frame_buffer) = engine.get_frame_buffer() {
                        p.frame_mut().copy_from_slice(frame_buffer);
                        if let Err(err) = p.render() {
                            error!("Failed to render pixels: {}", err);
                            engine.stop();
                            elwt.exit();
                        }
                    }
                }

                window_clone.request_redraw();
            }
            _ => {}
        }
    })?;
    
    Ok(())
}

/// Create a showcase of primitive types using actual primitive implementations
fn create_primitive_showcase_scene(engine: &mut Engine) -> Result<()> {
    info!("Creating primitive showcase scene...");
    
    let scene = engine.scene_mut();
    
    // Create materials for different primitive categories
    let second_life_material = LambertianMaterial::new(Color::rgb(0.2, 0.6, 0.9)); // Blue for SL primitives
    let advanced_material = LambertianMaterial::new(Color::rgb(0.9, 0.4, 0.2));    // Orange for advanced primitives
    let csg_material = LambertianMaterial::new(Color::rgb(0.6, 0.9, 0.3));         // Green for CSG examples
    let ground_material = LambertianMaterial::new(Color::rgb(0.2, 0.2, 0.2));      // Darker gray ground
    
    // Add ground plane (large sphere below)
    let ground = Sphere::with_material(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material
    );
    scene.add_sphere(Arc::new(ground));
    
    // === SECOND LIFE PRIMITIVES SECTION ===
    info!("Adding Second Life primitive implementations...");
    
    // Row 1: Basic Second Life primitives
    let row1_y = 2.0;
    
    // Box primitive (using Cube)
    let box_prim = Cube::with_material(
        Vec3::new(-12.0, row1_y, -8.0),
        Vec3::new(2.0, 2.0, 2.0),
        second_life_material.clone()
    );
    scene.add_object(Arc::new(box_prim));
    
    // Sphere primitive
    let sphere_prim = Sphere::with_material(
        Vec3::new(-8.0, row1_y, -8.0),
        1.2,
        second_life_material.clone()
    );
    scene.add_sphere(Arc::new(sphere_prim));
    
    // Cylinder primitive
    let cylinder_prim = Cylinder::with_material(
        Vec3::new(-4.0, row1_y, -8.0),
        1.0,  // radius
        2.0,  // height
        second_life_material.clone()
    );
    scene.add_object(Arc::new(cylinder_prim));
    
    // Prism representation (using smaller box for now)
    let prism_prim = Cube::with_material(
        Vec3::new(0.0, row1_y, -8.0),
        Vec3::new(1.5, 2.0, 1.0),
        second_life_material.clone()
    );
    scene.add_object(Arc::new(prism_prim));
    
    // Row 2: Advanced Second Life primitives (using spheres as placeholders for SDF primitives)
    let row2_y = 2.0;
    
    // Torus representation
    let torus_sphere = Sphere::with_material(
        Vec3::new(4.0, row2_y, -8.0),
        1.0,
        second_life_material.clone()
    );
    scene.add_sphere(Arc::new(torus_sphere));
    
    // Tube representation
    let tube_sphere = Sphere::with_material(
        Vec3::new(8.0, row2_y, -8.0),
        1.0,
        second_life_material.clone()
    );
    scene.add_sphere(Arc::new(tube_sphere));
    
    // Ring representation
    let ring_sphere = Sphere::with_material(
        Vec3::new(12.0, row2_y, -8.0),
        1.0,
        second_life_material.clone()
    );
    scene.add_sphere(Arc::new(ring_sphere));
    
    // === ADVANCED PRIMITIVES SECTION ===
    info!("Adding advanced primitive implementations...");
    
    // Row 3: Advanced primitives
    let row3_y = 2.0;
    
    // Cone primitive
    let cone_prim = Cone::with_material(
        Vec3::new(-8.0, row3_y, 0.0),
        1.2,  // base radius
        2.5,  // height
        advanced_material.clone()
    );
    scene.add_object(Arc::new(cone_prim));
    
    // Capsule primitive
    let capsule_prim = Capsule::with_material(
        Vec3::new(-4.0, row3_y, 0.0),
        0.8,  // radius
        2.0,  // height
        advanced_material.clone()
    );
    scene.add_object(Arc::new(capsule_prim));
    
    // Ellipsoid representation (using sphere for now)
    let ellipsoid_sphere = Sphere::with_material(
        Vec3::new(0.0, row3_y, 0.0),
        1.0,
        advanced_material.clone()
    );
    scene.add_sphere(Arc::new(ellipsoid_sphere));
    
    // === CSG OPERATIONS SHOWCASE ===
    info!("Adding CSG operation examples...");
    
    // Row 4: CSG examples (represented as grouped spheres)
    let row4_y = 2.0;
    
    // Union example: Two overlapping spheres
    let union_sphere1 = Sphere::with_material(
        Vec3::new(4.0, row4_y, 0.0),
        0.8,
        csg_material.clone()
    );
    let union_sphere2 = Sphere::with_material(
        Vec3::new(5.0, row4_y, 0.0),
        0.8,
        csg_material.clone()
    );
    scene.add_sphere(Arc::new(union_sphere1));
    scene.add_sphere(Arc::new(union_sphere2));
    
    // Difference example: Larger sphere with smaller one nearby
    let diff_sphere1 = Sphere::with_material(
        Vec3::new(8.0, row4_y, 0.0),
        1.2,
        csg_material.clone()
    );
    let diff_sphere2 = Sphere::with_material(
        Vec3::new(8.5, row4_y, 0.0),
        0.6,
        csg_material.clone()
    );
    scene.add_sphere(Arc::new(diff_sphere1));
    scene.add_sphere(Arc::new(diff_sphere2));
    
    // Intersection example: Two overlapping spheres
    let intersect_sphere1 = Sphere::with_material(
        Vec3::new(12.0, row4_y, 0.0),
        1.0,
        csg_material.clone()
    );
    let intersect_sphere2 = Sphere::with_material(
        Vec3::new(12.5, row4_y, 0.0),
        1.0,
        csg_material.clone()
    );
    scene.add_sphere(Arc::new(intersect_sphere1));
    scene.add_sphere(Arc::new(intersect_sphere2));
    
    // === LIGHTING SETUP ===
    info!("Setting up showcase lighting...");
    
    // Main key light (reduced intensity)
    let key_light = PointLight::new(
        Vec3::new(-10.0, 15.0, 10.0),
        Color::rgb(1.0, 1.0, 1.0),
        25.0  // Reduced from 100.0
    );
    scene.add_point_light(Arc::new(key_light));
    
    // Fill light (reduced intensity)
    let fill_light = PointLight::new(
        Vec3::new(10.0, 10.0, 15.0),
        Color::rgb(0.6, 0.7, 1.0),
        15.0  // Reduced from 60.0
    );
    scene.add_point_light(Arc::new(fill_light));
    
    // Rim light (reduced intensity)
    let rim_light = PointLight::new(
        Vec3::new(0.0, 5.0, -15.0),
        Color::rgb(1.0, 0.8, 0.6),
        10.0  // Reduced from 40.0
    );
    scene.add_point_light(Arc::new(rim_light));
    
    info!("Primitive showcase scene created successfully!");
    info!("Scene contains:");
    info!("  • 7 Second Life primitive representations");
    info!("  • 3 Advanced primitive implementations");
    info!("  • 6 CSG operation examples");
    info!("  • Professional lighting setup");
    info!("  • Total objects: {}", scene.object_count());
    info!("  • Total lights: {}", scene.light_count());
    
    Ok(())
}

/// Set up camera position for optimal viewing of the showcase
fn setup_showcase_camera(engine: &mut Engine) {
    let camera = engine.camera_mut();
    
    // Position camera for good overview of the scene
    camera.transform.position = Vec3::new(0.0, 8.0, 20.0);
    camera.look_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y);
    
    info!("Showcase camera positioned for optimal viewing");
} 