//! Advanced Demo Application
//! 
//! This demonstrates advanced features of the RRTE Engine including:
//! - Animated scenes with orbiting camera
//! - Dynamic lighting effects
//! - Atmospheric spooky scenes
//! - Real-time animation systems

use rrte_engine::prelude::*;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use std::sync::Arc;
use anyhow::Result;
use log::{info, error};
use std::time::{SystemTime, UNIX_EPOCH};

/// Animation controller for the spooky scene
pub struct SceneAnimation {
    pub start_time: f64,
    pub camera_orbit_speed: f32,
    pub camera_radius: f32,
    pub camera_height: f32,
    pub light_cycle_speed: f32,
}

impl SceneAnimation {
    pub fn new() -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
            
        Self {
            start_time,
            camera_orbit_speed: 0.3, // Slow, smooth orbit
            camera_radius: 12.0,      // Distance from center
            camera_height: 6.0,       // Height above ground
            light_cycle_speed: 0.8,   // Speed of color cycling
        }
    }
    
    pub fn get_time(&self) -> f32 {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        (current_time - self.start_time) as f32
    }
    
    pub fn update_camera(&self, engine: &mut Engine) {
        let time = self.get_time();
        let angle = time * self.camera_orbit_speed;
        
        // Calculate camera position in orbit
        let camera_x = angle.cos() * self.camera_radius;
        let camera_z = angle.sin() * self.camera_radius;
        let look_from = Vec3::new(camera_x, self.camera_height, camera_z);
        let look_at = Vec3::new(0.0, 1.0, 0.0); // Always look at center
        let up = Vec3::new(0.0, 1.0, 0.0);
        
        let camera = engine.camera_mut();
        camera.transform.position = look_from;
        camera.look_at(look_at, up);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    info!("Starting RRTE Engine Advanced Demo");
    info!("Engine: {} v{}", rrte_engine::ENGINE_NAME, rrte_engine::VERSION);
    info!("Features: GPU={}, SDF={}, Deformation={}", 
          rrte_engine::features::GPU_RAYTRACING,
          rrte_engine::features::SDF_SUPPORT,
          rrte_engine::features::DEFORMATION_SUPPORT);
    
    // Configure the engine for the advanced demo
    let raytracer_config = RaytracerConfig {
        max_depth: 50,
        samples_per_pixel: 10,
        width: 1200,
        height: 800,
        background_color: Color::new(0.05, 0.05, 0.1, 1.0), // Dark background
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
            .with_title("RRTE Engine - Advanced Demo (Spooky Scene)")
            .with_inner_size(LogicalSize::new(
                engine.config().renderer_config.width as f64, 
                engine.config().renderer_config.height as f64
            ))
            .build(&event_loop)
            .unwrap(),
    );

    // Initialize the engine's renderer with our window
    engine.initialize_renderer(Some(window.clone())).await?;

    // Create the advanced spooky scene
    create_spooky_scene(&mut engine)?;

    // Initialize scene animation
    let scene_animation = SceneAnimation::new();

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

    info!("Starting animated render loop...");
    
    // Main application loop with animation
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

                // Update scene animation (orbiting camera and cycling lights)
                scene_animation.update_camera(&mut engine);
                if let Err(e) = update_scene_lights(&mut engine, &scene_animation) {
                    error!("Scene lighting update error: {}", e);
                }

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

    info!("Advanced demo completed successfully.");
    Ok(())
}

/// Create the spooky animated scene with dark atmosphere
fn create_spooky_scene(engine: &mut Engine) -> Result<()> {
    info!("Creating dark spooky scene...");
    
    // Create dark, spooky materials with muted colors
    let dark_red = LambertianMaterial::new(Color::rgb(0.3, 0.05, 0.05));
    let dark_blue = LambertianMaterial::new(Color::rgb(0.05, 0.1, 0.3));
    let dark_green = LambertianMaterial::new(Color::rgb(0.05, 0.2, 0.05));
    let dark_purple = LambertianMaterial::new(Color::rgb(0.2, 0.05, 0.2));
    let dark_orange = LambertianMaterial::new(Color::rgb(0.3, 0.15, 0.02));
    let very_dark_gray = LambertianMaterial::new(Color::rgb(0.1, 0.1, 0.1));
    
    // Create spheres with darker materials for spooky atmosphere
    let spheres = [
        (Vec3::new(-6.0, 1.0, 0.0), 1.2, dark_red),
        (Vec3::new(-3.0, 1.0, 0.0), 0.8, dark_blue),
        (Vec3::new(0.0, 1.0, 0.0), 0.8, dark_green),
        (Vec3::new(3.0, 1.0, 0.0), 0.9, dark_purple),
        (Vec3::new(6.0, 1.0, 0.0), 0.7, dark_orange),
    ];
    
    {
        let scene = engine.scene_mut();
        for (position, radius, material) in spheres {
            let sphere = Sphere::with_material(position, radius, material);
            scene.add_sphere(Arc::new(sphere));
        }
        
        // Dark ground plane
        let ground_sphere = Sphere::with_material(
            Vec3::new(0.0, -1000.0, 0.0),
            1000.0,
            very_dark_gray
        );
        scene.add_sphere(Arc::new(ground_sphere));
        
        // Spooky lighting setup
        info!("Setting up atmospheric lighting...");
        
        let lights = [
            // Main atmospheric light
            (Vec3::new(0.0, 8.0, 0.0), Color::rgb(0.2, 0.1, 0.4), 15.0),
            // Wandering lights with different colors
            (Vec3::new(-8.0, 3.0, 4.0), Color::rgb(0.4, 0.1, 0.05), 12.0),
            (Vec3::new(8.0, 4.0, -4.0), Color::rgb(0.05, 0.2, 0.3), 10.0),
            (Vec3::new(-2.0, 1.5, -8.0), Color::rgb(0.1, 0.3, 0.05), 8.0),
            (Vec3::new(4.0, 6.0, 6.0), Color::rgb(0.3, 0.05, 0.3), 6.0),
        ];
        
        for (position, color, intensity) in lights {
            let light = PointLight::new(position, color, intensity);
            scene.add_point_light(Arc::new(light));
        }
    }
    
    // Set initial camera position
    setup_camera(engine);
    
    let scene = engine.scene();
    info!("Spooky scene created with {} objects and {} lights!", 
          scene.object_count(), 
          scene.light_count());
    
    Ok(())
}

/// Set up the initial camera position for the scene
fn setup_camera(engine: &mut Engine) {
    let look_from = Vec3::new(12.0, 6.0, 0.0);
    let look_at = Vec3::new(0.0, 1.0, 0.0);
    let up = Vec3::new(0.0, 1.0, 0.0);
    let fov = 60.0_f32.to_radians();

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

/// Update lighting effects for atmospheric animation
fn update_scene_lights(_engine: &mut Engine, animation: &SceneAnimation) -> Result<()> {
    let _time = animation.get_time();
    let _cycle_speed = animation.light_cycle_speed;
    
    // For now, we'll just leave the lights static
    // This is where you could add dynamic light color cycling
    // when the engine supports mutable light access
    
    // Future enhancement: cycle light colors based on time
    // let phase_offset = [0.0, 1.0, 2.0, 3.0, 4.0];
    // for (i, light) in scene.lights_mut().enumerate() {
    //     let phase = time * cycle_speed + phase_offset[i];
    //     light.color = calculate_cyclic_color(phase);
    // }
    
    Ok(())
} 