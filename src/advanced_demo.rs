use rrte_core::engine::Engine;
use rrte_renderer::material::LambertianMaterial;
use rrte_renderer::sdf_object::builders::{SDFBuilder, swiss_cheese_sphere, twisted_cylinder, organic_blob, complex_sculpture};
use rrte_renderer::sdf::primitives::*;
use rrte_renderer::sdf::CSGComposite;
use rrte_renderer::deformation::*;
use rrte_renderer::sdf_object::SDFObject;
use rrte_renderer::light::PointLight;
use rrte_math::{Vec3, Color, Transform};
use std::sync::Arc;
use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

/// Global state for camera orbiting and light cycling
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

/// Create an advanced demo scene showcasing SDF, CSG, and deformation features
pub fn create_advanced_demo_scene(engine: &mut Engine) -> Result<()> {
    let scene = engine.scene_mut();
    
    // Create dark, spooky scene with basic spheres
    println!("Creating dark spooky scene with cycling lights...");
    
    // Create dark, spooky materials with muted colors
    let dark_red = LambertianMaterial::new(Color::rgb(0.3, 0.05, 0.05));
    let dark_blue = LambertianMaterial::new(Color::rgb(0.05, 0.1, 0.3));
    let dark_green = LambertianMaterial::new(Color::rgb(0.05, 0.2, 0.05));
    let dark_purple = LambertianMaterial::new(Color::rgb(0.2, 0.05, 0.2));
    let dark_orange = LambertianMaterial::new(Color::rgb(0.3, 0.15, 0.02));
    let very_dark_gray = LambertianMaterial::new(Color::rgb(0.1, 0.1, 0.1));
    
    // Create spheres with darker materials for spooky atmosphere
    let sphere1 = rrte_renderer::primitives::Sphere::with_material(
        Vec3::new(-6.0, 1.0, 0.0),
        1.2,
        dark_red.clone()
    );
    scene.add_sphere(Arc::new(sphere1));
    
    let sphere2 = rrte_renderer::primitives::Sphere::with_material(
        Vec3::new(-3.0, 1.0, 0.0),
        0.8,
        dark_blue.clone()
    );
    scene.add_sphere(Arc::new(sphere2));
    
    let sphere3 = rrte_renderer::primitives::Sphere::with_material(
        Vec3::new(0.0, 1.0, 0.0),
        0.8,
        dark_green.clone()
    );
    scene.add_sphere(Arc::new(sphere3));
    
    let sphere4 = rrte_renderer::primitives::Sphere::with_material(
        Vec3::new(3.0, 1.0, 0.0),
        0.9,
        dark_purple.clone()
    );
    scene.add_sphere(Arc::new(sphere4));
    
    let sphere5 = rrte_renderer::primitives::Sphere::with_material(
        Vec3::new(6.0, 1.0, 0.0),
        0.7,
        dark_orange.clone()
    );
    scene.add_sphere(Arc::new(sphere5));
    
    // Dark ground plane
    let ground_sphere = rrte_renderer::primitives::Sphere::with_material(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        very_dark_gray.clone()
    );
    scene.add_sphere(Arc::new(ground_sphere));
    
    // Spooky lighting setup with cycling colors
    println!("Setting up spooky cycling lights...");
    
    // Create multiple lights that will cycle through different colors
    // Each light will have a different phase offset for varied cycling
    
    // Main atmospheric light (dim purple/blue cycling)
    let main_light = PointLight::new(
        Vec3::new(0.0, 8.0, 0.0),
        Color::rgb(0.2, 0.1, 0.4), // Start with dark purple
        15.0 // Much dimmer than before
    );
    scene.add_point_light(Arc::new(main_light));
    
    // Wandering light 1 (cycles through warm colors)
    let light1 = PointLight::new(
        Vec3::new(-8.0, 3.0, 4.0),
        Color::rgb(0.4, 0.1, 0.05), // Dark red-orange
        12.0
    );
    scene.add_point_light(Arc::new(light1));
    
    // Wandering light 2 (cycles through cool colors)
    let light2 = PointLight::new(
        Vec3::new(8.0, 4.0, -4.0),
        Color::rgb(0.05, 0.2, 0.3), // Dark cyan
        10.0
    );
    scene.add_point_light(Arc::new(light2));
    
    // Mysterious low light (cycles through green/purple)
    let light3 = PointLight::new(
        Vec3::new(-2.0, 1.5, -8.0),
        Color::rgb(0.1, 0.3, 0.05), // Dark green
        8.0
    );
    scene.add_point_light(Arc::new(light3));
    
    // Accent light (cycles through all colors)
    let light4 = PointLight::new(
        Vec3::new(4.0, 6.0, 6.0),
        Color::rgb(0.3, 0.05, 0.3), // Dark magenta
        6.0
    );
    scene.add_point_light(Arc::new(light4));
    
    println!("Spooky scene created with {} objects and {} lights!", 
             scene.object_count(), 
             scene.light_count());
    
    // --- Initial Camera Setup ---
    // Camera will be updated by the animation system
    let look_from = Vec3::new(12.0, 6.0, 0.0); // Starting position
    let look_at = Vec3::new(0.0, 1.0, 0.0);   // Look at the center of the scene
    let up = Vec3::new(0.0, 1.0, 0.0);
    let fov = 60.0_f32.to_radians(); // Wider field of view for dramatic effect

    {
        let camera = engine.camera_mut();
        // Update camera FOV
        if let rrte_renderer::camera::ProjectionType::Perspective { fov: camera_fov, ..} = &mut camera.projection {
            *camera_fov = fov;
        }
        // Set camera position and orientation
        camera.transform.position = look_from;
        camera.look_at(look_at, up);
    }

    println!("Camera positioned at {:?} looking at {:?}", look_from, look_at);
    println!("Scene animation initialized - camera will orbit and lights will cycle colors!");
    
    Ok(())
}

/// Update the scene animation (call this in the main loop)
pub fn update_scene_animation(engine: &mut Engine, animation: &SceneAnimation) -> Result<()> {
    let time = animation.get_time();
    
    // Update orbiting camera
    animation.update_camera(engine);
    
    // Update cycling light colors
    update_cycling_lights(engine, time, animation.light_cycle_speed)?;
    
    Ok(())
}

/// Update light colors with smooth cycling
fn update_cycling_lights(engine: &mut Engine, time: f32, cycle_speed: f32) -> Result<()> {
    // This is a simplified approach - in a full implementation you'd need 
    // access to the actual light objects to modify their colors.
    // For now, we'll just print what the colors should be.
    
    let t = time * cycle_speed;
    
    // Calculate cycling colors for each light with different phase offsets
    let light1_color = Color::rgb(
        0.3 + 0.2 * (t).sin().abs(),           // Red channel
        0.1 + 0.1 * (t + 1.0).sin().abs(),     // Green channel  
        0.05 + 0.1 * (t + 2.0).sin().abs(),    // Blue channel
    );
    
    let light2_color = Color::rgb(
        0.05 + 0.1 * (t + 3.0).sin().abs(),
        0.2 + 0.15 * (t + 4.0).sin().abs(),
        0.3 + 0.2 * (t + 5.0).sin().abs(),
    );
    
    let light3_color = Color::rgb(
        0.1 + 0.15 * (t + 6.0).sin().abs(),
        0.3 + 0.2 * (t + 7.0).sin().abs(),
        0.05 + 0.1 * (t + 8.0).sin().abs(),
    );
    
    let light4_color = Color::rgb(
        0.3 + 0.2 * (t + 9.0).sin().abs(),
        0.05 + 0.1 * (t + 10.0).sin().abs(),
        0.3 + 0.2 * (t + 11.0).sin().abs(),
    );
    
    // Note: In a full implementation, you would update the actual light colors here
    // This requires modifying the GPU renderer to support dynamic light updates
    
    Ok(())
}

/// Create a simpler demo focusing on one advanced feature at a time
pub fn create_focused_demo_scene(engine: &mut Engine, demo_type: DemoType) -> Result<()> {
    let scene = engine.scene_mut();
    
    // Clear existing scene
    scene.clear();
    
    let material = LambertianMaterial::new(Color::rgb(0.7, 0.3, 0.5));
    let ground_material = LambertianMaterial::new(Color::rgb(0.5, 0.5, 0.5));
    
    match demo_type {
        DemoType::CSGOperations => {
            println!("Creating CSG operations demo...");
            
            // Sphere - Box difference
            let sphere = Arc::new(SDFSphere::new(Vec3::new(-3.0, 1.0, 0.0), 1.0));
            let box_cutter = Arc::new(SDFBox::new(Vec3::new(-3.0, 1.0, 0.0), Vec3::new(1.2, 1.2, 1.2)));
            let difference = Arc::new(CSGComposite::difference(sphere, box_cutter));
            let diff_obj = SDFObject::with_material(difference, material.clone());
            scene.add_object(Arc::new(diff_obj));
            
            // Union of cylinders
            let cyl1 = Arc::new(SDFCylinder::new(Vec3::new(0.0, 1.0, 0.0), 0.5, 2.0));
            let cyl2 = Arc::new(SDFCylinder::new(Vec3::new(0.0, 1.0, 0.0), 0.5, 2.0));
            // Rotate second cylinder
            let mut rotated_cyl2 = SDFCylinder::new(Vec3::new(0.0, 1.0, 0.0), 0.5, 2.0);
            rotated_cyl2.transform.rotation = rrte_math::Quat::from_rotation_z(std::f32::consts::PI / 2.0);
            let cyl2_rotated = Arc::new(rotated_cyl2);
            
            let union = Arc::new(CSGComposite::union(cyl1, cyl2_rotated));
            let union_obj = SDFObject::with_material(union, material.clone());
            scene.add_object(Arc::new(union_obj));
            
            // Smooth intersection
            let sphere1 = Arc::new(SDFSphere::new(Vec3::new(3.0, 1.0, 0.0), 0.8));
            let sphere2 = Arc::new(SDFSphere::new(Vec3::new(3.5, 1.0, 0.0), 0.8));
            let smooth_int = Arc::new(CSGComposite::smooth_intersection(sphere1, sphere2, 0.3));
            let smooth_obj = SDFObject::with_material(smooth_int, material.clone());
            scene.add_object(Arc::new(smooth_obj));
        }
        
        DemoType::Deformations => {
            println!("Creating deformations demo...");
            
            // Twisted box
            let box_sdf = Arc::new(SDFBox::new(Vec3::new(-2.0, 1.0, 0.0), Vec3::new(0.8, 1.6, 0.8)));
            let twist_deformer = TwistDeformer::new(Vec3::Y, 2.0);
            let twisted = Arc::new(DeformedSDF::new(box_sdf, Box::new(twist_deformer)));
            let twisted_obj = SDFObject::with_material(twisted, material.clone());
            scene.add_object(Arc::new(twisted_obj));
            
            // Bent cylinder
            let cyl_sdf = Arc::new(SDFCylinder::new(Vec3::new(0.0, 1.0, 0.0), 0.3, 2.0));
            let bend_deformer = BendDeformer::new(Vec3::X, Vec3::Y, 1.0);
            let bent = Arc::new(DeformedSDF::new(cyl_sdf, Box::new(bend_deformer)));
            let bent_obj = SDFObject::with_material(bent, material.clone());
            scene.add_object(Arc::new(bent_obj));
            
            // Tapered sphere
            let sphere_sdf = Arc::new(SDFSphere::new(Vec3::new(2.0, 1.0, 0.0), 0.8));
            let taper_deformer = TaperDeformer::new(Vec3::Y, 1.0, 0.3, 1.6);
            let tapered = Arc::new(DeformedSDF::new(sphere_sdf, Box::new(taper_deformer)));
            let tapered_obj = SDFObject::with_material(tapered, material.clone());
            scene.add_object(Arc::new(tapered_obj));
        }
        
        DemoType::NoiseAndWaves => {
            println!("Creating noise and waves demo...");
            
            // Noisy sphere
            let sphere_sdf = Arc::new(SDFSphere::new(Vec3::new(-1.0, 1.0, 0.0), 0.8));
            let noise_deformer = NoiseDeformer::new(3.0, 0.2).with_octaves(3);
            let noisy = Arc::new(DeformedSDF::new(sphere_sdf, Box::new(noise_deformer)));
            let noisy_obj = SDFObject::with_material(noisy, material.clone());
            scene.add_object(Arc::new(noisy_obj));
            
            // Wave-deformed torus
            let torus_sdf = Arc::new(SDFTorus::new(Vec3::new(1.0, 1.0, 0.0), 0.8, 0.3));
            let wave_deformer = WaveDeformer::new(Vec3::Y, 0.3, 5.0);
            let wavy = Arc::new(DeformedSDF::new(torus_sdf, Box::new(wave_deformer)));
            let wavy_obj = SDFObject::with_material(wavy, material.clone());
            scene.add_object(Arc::new(wavy_obj));
        }
    }
    
    // Ground
    let ground_sphere = rrte_renderer::primitives::Sphere::with_material(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material
    );
    scene.add_object(Arc::new(ground_sphere));
    
    // Simple lighting
    let main_light = PointLight::new(
        Vec3::new(0.0, 8.0, 5.0),
        Color::rgb(1.0, 1.0, 1.0),
        50.0
    );
    scene.add_light(Arc::new(main_light));
    
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum DemoType {
    CSGOperations,
    Deformations,
    NoiseAndWaves,
} 