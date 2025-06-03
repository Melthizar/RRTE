//! Simple Demo Application
//! 
//! A minimal demo with just:
//! - A ground plane
//! - A single cube
//! - One light
//! - A camera

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
    
    info!("Starting Simple RRTE Demo");
    
    // Configure the engine
    let raytracer_config = RaytracerConfig {
        max_depth: 10,
        samples_per_pixel: 2,
        width: 800,
        height: 600,
        background_color: Color::new(0.2, 0.3, 0.4, 1.0), // Nice blue-gray background
    };
    
    let gpu_renderer_config = GpuRendererConfig {
        width: raytracer_config.width,
        height: raytracer_config.height,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        ..Default::default()
    };
    
    let engine_config = EngineConfig {
        renderer_mode: RendererMode::Gpu,
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
            .with_title("Simple RRTE Demo - Plane, Cube, Light")
            .with_inner_size(LogicalSize::new(800, 600))
            .with_resizable(true)
            .build(&event_loop)
            .unwrap(),
    );
    
    // Initialize renderer with window
    engine.initialize_renderer(Some(window.clone())).await?;
    
    // Create the simple scene
    create_simple_scene(&mut engine)?;
    
    // Set up camera
    setup_camera(&mut engine);
    
    // Initialize pixels for CPU fallback rendering
    let render_cpu_path = engine.get_frame_buffer().is_some();
    let mut pixels: Option<Pixels> = if render_cpu_path {
        info!("CPU rendering path detected, initializing Pixels.");
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        Some(Pixels::new(800, 600, surface_texture)?)
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

/// Create a simple scene with just a plane, cube, and light
fn create_simple_scene(engine: &mut Engine) -> Result<()> {
    info!("Creating simple scene...");
    
    let scene = engine.scene_mut();
    
    // Create materials
    let ground_material = LambertianMaterial::new(Color::rgb(0.5, 0.5, 0.5)); // Gray ground
    let cube_material = LambertianMaterial::new(Color::rgb(0.8, 0.3, 0.3));   // Red cube
    
    // Add ground plane (using a large sphere below)
    let ground = Sphere::with_material(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material
    );
    scene.add_sphere(Arc::new(ground));
    
    // Add a single cube
    let cube = Cube::with_material(
        Vec3::new(0.0, 1.0, 0.0),      // Position: 1 unit above ground
        Vec3::new(2.0, 2.0, 2.0),      // Size: 2x2x2 cube
        cube_material
    );
    scene.add_object(Arc::new(cube));
    
    // Add a single light
    let light = PointLight::new(
        Vec3::new(5.0, 10.0, 5.0),     // Position: up and to the side
        Color::rgb(1.0, 1.0, 1.0),     // White light
        50.0                           // Moderate intensity
    );
    scene.add_point_light(Arc::new(light));
    
    info!("Simple scene created successfully!");
    info!("Scene contains:");
    info!("  • 1 Ground plane (sphere)");
    info!("  • 1 Red cube");
    info!("  • 1 White light");
    info!("  • Total objects: {}", scene.object_count());
    info!("  • Total lights: {}", scene.light_count());
    
    Ok(())
}

/// Set up camera for a nice view of the scene
fn setup_camera(engine: &mut Engine) {
    let camera = engine.camera_mut();
    
    // Position camera for a good view of the cube
    camera.transform.position = Vec3::new(5.0, 3.0, 5.0);
    camera.look_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y); // Look at the cube
    
    info!("Camera positioned for simple scene view");
} 