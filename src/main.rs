use rrte_core::{Engine, EngineConfig, RendererMode};
use rrte_renderer::{RaytracerConfig, GpuRendererConfig};
use rrte_math::{Vec3, Color};
use anyhow::Result;
use log::{info, error};
use std::sync::Arc;
use rrte_renderer::material::LambertianMaterial;

mod advanced_demo;
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    // Configure the raytracer (CPU) and GPU renderer configs
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
        // Use a common format like Bgra8UnormSrgb if pixels provides one, 
        // otherwise GpuRenderer has its own default.
        // For now, let GpuRenderer use its default and potentially update it.
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

    // Create and initialize the engine's core systems
    let mut engine = Engine::new(engine_config)?;
    engine.initialize_core_systems()?;

    info!("RRTE Raytracing Demo Starting with Winit Window (GPU Mode Attempt)...");
    
    // --- Winit Setup ---
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("RRTE Engine (GPU Mode)")
            .with_inner_size(LogicalSize::new(
                engine.config().renderer_config.width as f64, 
                engine.config().renderer_config.height as f64
            ))
            .build(&event_loop)
            .unwrap(),
    );

    // Initialize renderer (now async and takes window handle)
    engine.initialize_renderer(Some(window.clone())).await?;

    // Create the advanced demo scene showcasing SDF, CSG, and deformation features
    if let Err(e) = advanced_demo::create_advanced_demo_scene(&mut engine) {
        error!("Failed to create advanced demo scene: {}. Falling back to basic scene.", e);
        if let Err(e2) = create_demo_scene(&mut engine) {
            error!("Failed to create basic demo scene: {}", e2);
        }
    }

    // Initialize scene animation for orbiting camera and cycling lights
    let scene_animation = advanced_demo::SceneAnimation::new();

    // Pixels is only for CPU rendering path. 
    // We initialize it but will only use it if engine.get_frame_buffer() is Some.
    let RENDER_CPU_PATH = engine.get_frame_buffer().is_some();
    let mut pixels: Option<Pixels> = if RENDER_CPU_PATH {
        info!("CPU rendering path detected, initializing Pixels.");
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        Some(Pixels::new(engine.config().renderer_config.width, engine.config().renderer_config.height, surface_texture)?)
    } else {
        info!("GPU rendering path detected, Pixels will not be used for display.");
        None
    };
    
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
                if size.width == 0 || size.height == 0 {return;}
                
                engine.update_resolution(size.width, size.height);

                if let Some(p) = pixels.as_mut() {
                    if let Err(e) = p.resize_surface(size.width, size.height) {
                        error!("Pixels resize_surface error: {}", e); engine.stop(); elwt.exit(); return;
                    }
                    if let Err(e) = p.resize_buffer(size.width, size.height) {
                        error!("Pixels resize_buffer error: {}", e); engine.stop(); elwt.exit(); return;
                    }
                } 
                // GpuRenderer's resize is called within engine.update_resolution()
            }
            Event::AboutToWait => {
                if !engine.is_running() {
                    elwt.exit();
                    return;
                }

                let dt = engine.time().delta_time();
                engine.time_mut().update();
                engine.input_mut().update();
                engine.scene_mut().update(dt);

                // Update scene animation (orbiting camera and cycling lights)
                if let Err(e) = advanced_demo::update_scene_animation(&mut engine, &scene_animation) {
                    error!("Scene animation update error: {}", e);
                }

                if let Err(e) = engine.render_frame() {
                    error!("Engine render_frame error: {}", e);
                    engine.stop();
                    elwt.exit();
                    return;
                }

                // If CPU rendering, copy to pixels buffer and render
                if let (Some(p), Some(frame_buffer)) = (pixels.as_mut(), engine.get_frame_buffer()) {
                    p.frame_mut().copy_from_slice(frame_buffer);
                    if let Err(e) = p.render() {
                        error!("Pixels render error: {}", e);
                        engine.stop();
                        elwt.exit();
                        return;
                    }
                } else if engine.config().renderer_mode == RendererMode::Gpu {
                    // For GPU, wgpu handles presentation in GpuRenderer::render().
                    // We still request redraw to keep the event loop spinning for input etc.
                }
                window_clone.request_redraw();
            }
            _ => (),
        }
    })?;

    info!("Exiting application.");
    Ok(())
}

fn create_demo_scene(engine: &mut Engine) -> Result<()> {
    use rrte_renderer::primitives::Sphere;
    use rrte_renderer::light::PointLight;

    // --- Scene Setup ---
    {
        let scene = engine.scene_mut();

        // Create materials
        let ground_material = LambertianMaterial::new(Color::rgb(0.5, 0.5, 0.5));
        let center_material = LambertianMaterial::new(Color::rgb(0.7, 0.3, 0.3));
        // let left_material = LambertianMaterial::new(Color::rgb(0.8, 0.8, 0.0)); // Removed
        // let right_material = LambertianMaterial::new(Color::rgb(0.0, 0.0, 0.8)); // Removed
        // let small_sphere_material = LambertianMaterial::new(Color::rgb(0.2, 0.8, 0.2)); // Removed

        // Create some basic spheres with materials
        let ground_sphere = Sphere::with_material(Vec3::new(0.0, -1000.0, 0.0), 1000.0, ground_material.clone());
        scene.add_sphere(Arc::new(ground_sphere));

        let center_sphere = Sphere::with_material(Vec3::new(0.0, 1.0, 0.0), 1.0, center_material.clone());
        scene.add_sphere(Arc::new(center_sphere));

        // let left_sphere = Sphere::with_material(Vec3::new(-4.0, 1.0, 0.0), 1.0, left_material.clone()); // Removed
        // scene.add_object(Arc::new(left_sphere)); // Removed

        // let right_sphere = Sphere::with_material(Vec3::new(4.0, 1.0, 0.0), 1.0, right_material.clone()); // Removed
        // scene.add_object(Arc::new(right_sphere)); // Removed

        // Add a few more smaller spheres // Removed
        // for i in 0..5 { // Removed
        //     let angle = (i as f32) * std::f32::consts::PI * 2.0 / 5.0; // Removed
        //     let x = angle.cos() * 2.0; // Removed
        //     let z = angle.sin() * 2.0; // Removed
        //     let sphere = Sphere::with_material(Vec3::new(x, 0.3, z), 0.3, small_sphere_material.clone()); // Removed
        //     scene.add_object(Arc::new(sphere)); // Removed
        // } // Removed

        // Add point lights
        let main_light = PointLight::new(
            Vec3::new(10.0, 10.0, 10.0),
            Color::new(1.0, 1.0, 1.0, 1.0),
            25.0 // Reduced from 100.0 to 25.0 for less brightness
        );
        scene.add_point_light(Arc::new(main_light));

        // Add blue fill light
        let fill_light = PointLight::new(
            Vec3::new(-10.0, 5.0, 0.0),
            Color::new(0.7, 0.8, 1.0, 1.0), // Cool blue fill
            15.0 // Subtle fill lighting
        );
        scene.add_light(Arc::new(fill_light));

        // Add warm accent light
        let accent_light = PointLight::new(
            Vec3::new(0.0, 8.0, -8.0),
            Color::new(1.0, 0.8, 0.6, 1.0), // Warm orange
            12.0 // Subtle accent
        );
        scene.add_light(Arc::new(accent_light));

        // Add a rim light for dramatic effect
        let rim_light = PointLight::new(
            Vec3::new(-8.0, 2.0, 2.0),
            Color::new(0.8, 0.9, 1.0, 1.0), // Cool blue rim
            10.0 // Rim lighting intensity
        );
        scene.add_light(Arc::new(rim_light));

        // Add a subtle floor bounce light
        let bounce_light = PointLight::new(
            Vec3::new(3.0, -0.5, 3.0), // Low to the ground
            Color::new(0.9, 0.7, 0.6, 1.0), // Warm bounce
            8.0 // Subtle bounce
        );
        scene.add_light(Arc::new(bounce_light));
    }

    // --- Camera Setup ---
    let look_from = Vec3::new(6.0, 3.0, 6.0); // Closer to the scene and higher up
    let look_at = Vec3::new(0.0, 0.0, 0.0);
    let up = Vec3::new(0.0, 1.0, 0.0);
    let fov = 45.0_f32.to_radians(); // Increased from 20.0 to 45.0 degrees for wider view
    // Aspect ratio is now set by engine.update_resolution and initialized in Engine::new
    // let aspect_ratio = engine.config().renderer_config.width as f32 / engine.config().renderer_config.height as f32;

    {
        let camera = engine.camera_mut();
        // Camera FOV and near/far can be set here, aspect ratio is handled by resize.
        if let rrte_renderer::camera::ProjectionType::Perspective { fov: camera_fov, ..} = &mut camera.projection {
            *camera_fov = fov;
        }
        // For a new camera, you might do:
        // *camera = Camera::new_perspective(fov, initial_aspect_ratio, 0.1, 100.0);
        camera.transform.position = look_from;
        camera.look_at(look_at, up);
    }

    info!("Demo scene created with {} objects and {} lights", 
          engine.scene().object_count(), engine.scene().light_count());

    Ok(())
}
