//! Basic Demo Application
//! 
//! This demonstrates the fundamental usage of the RRTE Engine.
//! Shows how external applications should integrate with the engine.

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
    
    info!("Starting RRTE Engine Basic Demo");
    info!("Engine: {} v{}", rrte_engine::ENGINE_NAME, rrte_engine::VERSION);
    
    // Configure the engine
    let raytracer_config = RaytracerConfig {
        max_depth: 50,
        samples_per_pixel: 10,
        width: 800,
        height: 600,
        background_color: Color::new(0.5, 0.7, 1.0, 1.0),
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

    // Create and initialize the engine
    let mut engine = Engine::new(engine_config)?;
    engine.initialize_core_systems()?;

    info!("Creating window and initializing graphics...");
    
    // Create window
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("RRTE Engine - Basic Demo")
            .with_inner_size(LogicalSize::new(
                engine.config().renderer_config.width as f64, 
                engine.config().renderer_config.height as f64
            ))
            .build(&event_loop)
            .unwrap(),
    );

    // Initialize the engine's renderer with our window
    engine.initialize_renderer(Some(window.clone())).await?;

    // Create a basic scene
    create_basic_scene(&mut engine)?;

    // Set up camera
    setup_camera(&mut engine);

    // Initialize pixels for CPU fallback rendering
    let render_cpu_path = engine.get_frame_buffer().is_some();
    let mut pixels: Option<Pixels> = if render_cpu_path {
        info!("CPU rendering path detected, initializing Pixels.");
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        Some(Pixels::new(
            engine.config().renderer_config.width, 
            engine.config().renderer_config.height, 
            surface_texture
        )?)
    } else {
        info!("GPU rendering path detected.");
        None
    };

    info!("Starting render loop...");
    
    // Main application loop
    let window_clone = window.clone();
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
                ..
            } if window_id == window_clone.id() => {
                info!("Close requested, stopping engine.");
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

                // Handle CPU rendering path
                if let (Some(p), Some(frame_buffer)) = (pixels.as_mut(), engine.get_frame_buffer()) {
                    p.frame_mut().copy_from_slice(frame_buffer);
                    if let Err(e) = p.render() {
                        error!("Pixels render error: {}", e);
                        engine.stop();
                        elwt.exit();
                        return;
                    }
                }
                
                window_clone.request_redraw();
            }
            _ => (),
        }
    })?;

    info!("Basic demo completed successfully.");
    Ok(())
}

/// Create a simple scene with a few spheres and basic lighting
fn create_basic_scene(engine: &mut Engine) -> Result<()> {
    info!("Creating basic scene...");
    
    let scene = engine.scene_mut();
    
    // Create materials
    let ground_material = LambertianMaterial::new(Color::rgb(0.5, 0.5, 0.5));
    let red_material = LambertianMaterial::new(Color::rgb(0.7, 0.3, 0.3));
    let blue_material = LambertianMaterial::new(Color::rgb(0.3, 0.3, 0.7));
    let green_material = LambertianMaterial::new(Color::rgb(0.3, 0.7, 0.3));
    
    // Add ground sphere (large sphere below scene)
    let ground_sphere = Sphere::with_material(
        Vec3::new(0.0, -1000.0, 0.0), 
        1000.0, 
        ground_material
    );
    scene.add_sphere(Arc::new(ground_sphere));
    
    // Add three colorful spheres
    let center_sphere = Sphere::with_material(
        Vec3::new(0.0, 1.0, 0.0), 
        1.0, 
        red_material
    );
    scene.add_sphere(Arc::new(center_sphere));
    
    let left_sphere = Sphere::with_material(
        Vec3::new(-2.5, 1.0, 0.0), 
        1.0, 
        blue_material
    );
    scene.add_sphere(Arc::new(left_sphere));
    
    let right_sphere = Sphere::with_material(
        Vec3::new(2.5, 1.0, 0.0), 
        1.0, 
        green_material
    );
    scene.add_sphere(Arc::new(right_sphere));
    
    // Add lighting
    let main_light = PointLight::new(
        Vec3::new(0.0, 5.0, 5.0),
        Color::rgb(1.0, 1.0, 1.0),
        50.0
    );
    scene.add_point_light(Arc::new(main_light));
    
    // Add accent light
    let accent_light = PointLight::new(
        Vec3::new(-5.0, 3.0, -2.0),
        Color::rgb(0.8, 0.9, 1.0),
        30.0
    );
    scene.add_point_light(Arc::new(accent_light));
    
    info!("Basic scene created with {} objects and {} lights", 
          scene.object_count(), 
          scene.light_count());
    
    Ok(())
}

/// Set up the camera for a nice view of the scene
fn setup_camera(engine: &mut Engine) {
    let look_from = Vec3::new(6.0, 4.0, 6.0);
    let look_at = Vec3::new(0.0, 1.0, 0.0);
    let up = Vec3::new(0.0, 1.0, 0.0);
    let fov = 45.0_f32.to_radians();

    let camera = engine.camera_mut();
    
    // Update camera FOV
    if let ProjectionType::Perspective { fov: camera_fov, .. } = &mut camera.projection {
        *camera_fov = fov;
    }
    
    // Set camera position and orientation
    camera.transform.position = look_from;
    camera.look_at(look_at, up);
    
    info!("Camera positioned at {:?} looking at {:?}", look_from, look_at);
} 