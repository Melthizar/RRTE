use rrte_core::{Engine, EngineConfig};
use rrte_renderer::RaytracerConfig;
use rrte_math::{Vec3, Color, Transform};
use anyhow::Result;
use log::info;
use std::sync::Arc;

fn main() -> Result<()> {
    // Configure the raytracer for a nice demo
    let raytracer_config = RaytracerConfig {
        max_depth: 50,
        samples_per_pixel: 10, // Lower for faster demo
        width: 800,
        height: 600,
        background_color: Color::new(0.5, 0.7, 1.0, 1.0), // Sky blue
    };

    let engine_config = EngineConfig {
        renderer_config: raytracer_config,
        target_fps: 30.0, // Lower FPS for raytracing demo
        enable_vsync: true,
        log_level: log::LevelFilter::Info,
    };

    // Create and initialize the engine
    let mut engine = Engine::new(engine_config)?;
    engine.initialize()?;

    info!("RRTE Raytracing Demo Starting...");
    info!("Creating demo scene with multiple materials and objects...");

    // Create the demo scene
    create_demo_scene(&mut engine)?;

    // Run the engine
    engine.run()?;

    // Shutdown
    engine.shutdown()?;
    
    Ok(())
}

fn create_demo_scene(engine: &mut Engine) -> Result<()> {
    use rrte_renderer::primitives::Sphere;
    use rrte_renderer::light::PointLight;
    use rrte_core::Camera;

    let scene = engine.scene_mut();

    // Create some basic spheres for now (without materials initially)
    let ground_sphere = Sphere::new(Vec3::new(0.0, -1000.0, 0.0), 1000.0);
    scene.add_object(Arc::new(ground_sphere));

    let center_sphere = Sphere::new(Vec3::new(0.0, 1.0, 0.0), 1.0);
    scene.add_object(Arc::new(center_sphere));

    let left_sphere = Sphere::new(Vec3::new(-4.0, 1.0, 0.0), 1.0);
    scene.add_object(Arc::new(left_sphere));

    let right_sphere = Sphere::new(Vec3::new(4.0, 1.0, 0.0), 1.0);
    scene.add_object(Arc::new(right_sphere));

    // Add a few more smaller spheres
    for i in 0..5 {
        let angle = (i as f32) * std::f32::consts::PI * 2.0 / 5.0;
        let x = angle.cos() * 2.0;
        let z = angle.sin() * 2.0;
        let sphere = Sphere::new(Vec3::new(x, 0.3, z), 0.3);
        scene.add_object(Arc::new(sphere));
    }

    // Add point lights
    let main_light = PointLight::new(
        Vec3::new(10.0, 10.0, 10.0),
        Color::new(1.0, 1.0, 1.0, 1.0),
        100.0
    );
    scene.add_light(Arc::new(main_light));

    let fill_light = PointLight::new(
        Vec3::new(-10.0, 5.0, 0.0),
        Color::new(0.7, 0.8, 1.0, 1.0),
        50.0
    );
    scene.add_light(Arc::new(fill_light));

    // Set up camera
    let camera = engine.camera_mut();
    let look_from = Vec3::new(13.0, 2.0, 3.0);
    let look_at = Vec3::new(0.0, 0.0, 0.0);
    let up = Vec3::new(0.0, 1.0, 0.0);
    let fov = 20.0_f32.to_radians();
    let aspect_ratio = 800.0 / 600.0;

    *camera = Camera::new_perspective(fov, aspect_ratio, 0.1, 100.0);
    
    // Set camera position and look at target
    camera.transform.position = look_from;
    camera.look_at(look_at, up);

    info!("Demo scene created with {} objects and {} lights", 
          scene.object_count(), scene.light_count());

    Ok(())
}
