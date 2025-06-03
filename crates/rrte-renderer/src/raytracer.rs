use rrte_math::{Ray, HitInfo, Vec3, Color};
use crate::{Material, SceneObject, Light, Camera};
use rayon::prelude::*;
use std::sync::Arc;

/// Raytracing renderer configuration
#[derive(Debug, Clone)]
pub struct RaytracerConfig {
    pub max_depth: u32,
    pub samples_per_pixel: u32,
    pub width: u32,
    pub height: u32,
    pub background_color: Color,
}

impl Default for RaytracerConfig {
    fn default() -> Self {
        Self {
            max_depth: 50,
            samples_per_pixel: 100,
            width: 800,
            height: 600,
            background_color: Color::new(0.5, 0.7, 1.0, 1.0), // Sky blue
        }
    }
}

/// CPU-based raytracer
pub struct Raytracer {
    config: RaytracerConfig,
}

impl Raytracer {
    /// Create a new raytracer with configuration
    pub fn new(config: RaytracerConfig) -> Self {
        Self { config }
    }

    /// Update the raytracer's configuration
    pub fn update_config(&mut self, new_config: RaytracerConfig) {
        self.config = new_config;
    }

    /// Render a scene to a pixel buffer
    pub fn render(
        &self,
        objects: &[Arc<dyn SceneObject>],
        lights: &[Arc<dyn Light>],
        materials: &[Arc<dyn Material>],
        camera: &Camera,
    ) -> Vec<u8> {
        let width = self.config.width as usize;
        let height = self.config.height as usize;
        let mut pixels = vec![0u8; width * height * 4];

        // Parallel rendering
        pixels
            .par_chunks_mut(4)
            .enumerate()
            .for_each(|(i, pixel)| {
                let x = i % width;
                let y = i / width;
                
                let mut color = Color::BLACK;
                  // Multi-sampling for anti-aliasing
                for _ in 0..self.config.samples_per_pixel {
                    let u = (x as f32 + rand::random::<f32>()) / width as f32;
                    let v = (y as f32 + rand::random::<f32>()) / height as f32;
                    
                    let ray = camera.generate_ray(u, v);
                    let sample_color = self.ray_color(&ray, objects, lights, materials, self.config.max_depth);
                    color = color + sample_color;
                }
                
                // Average the samples
                color = color * (1.0 / self.config.samples_per_pixel as f32);
                
                // Gamma correction
                color = color.to_gamma(2.2).clamp();
                
                // Convert to u8 RGBA
                pixel[0] = (color.r * 255.0) as u8;
                pixel[1] = (color.g * 255.0) as u8;
                pixel[2] = (color.b * 255.0) as u8;
                pixel[3] = (color.a * 255.0) as u8;
            });

        pixels
    }

    /// Calculate color for a ray
    fn ray_color(
        &self,
        ray: &Ray,
        objects: &[Arc<dyn SceneObject>],
        lights: &[Arc<dyn Light>],
        materials: &[Arc<dyn Material>],
        depth: u32,
    ) -> Color {
        if depth == 0 {
            return Color::BLACK;
        }        // Find closest intersection
        let mut closest_hit: Option<HitInfo> = None;
        let mut closest_object: Option<&Arc<dyn SceneObject>> = None;
        
        for object in objects {
            if let Some(hit) = object.intersect(ray, 0.001, f32::INFINITY) {
                if closest_hit.is_none() || hit.t < closest_hit.as_ref().unwrap().t {
                    closest_hit = Some(hit);
                    closest_object = Some(object);
                }
            }
        }

        if let (Some(hit), Some(object_arc)) = (closest_hit, closest_object) {
            // Get material directly from the object
            if let Some(material_arc) = object_arc.material() {
                let material = material_arc; // material is Arc<dyn Material>
            
                // Calculate lighting
                let mut color = Color::BLACK;
            
                // Ambient lighting
                color = color + material.ambient_color() * 0.1; // Assuming ambient_color() exists and is suitable
                // Direct lighting from light sources
                for light in lights {
                    let light_contribution = light.illuminate(hit.point, hit.normal);
                    color = color + light_contribution.color * light_contribution.attenuation;
                }
            
                // Recursive reflection/refraction
                if let Some(scattered_ray) = material.scatter(ray, &hit) {
                    let attenuation = material.albedo();
                    let scattered_color = self.ray_color(&scattered_ray, objects, lights, materials, depth - 1);
                    color = color + Color::from(attenuation.to_vec3() * scattered_color.to_vec3());
                }
            
                return color; // Return the calculated color
            } else {
                // Object hit but has no material. This should ideally be handled.
                // For now, return black to make it visually distinct if this path is taken.
                return Color::BLACK; 
            }
        } else {
            // Background color
            self.config.background_color
        }
    }
}
