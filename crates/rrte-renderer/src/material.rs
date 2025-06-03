use rrte_math::{Ray, HitInfo, Color, Vec3};
use std::sync::Arc;

/// Trait for materials that determine how light interacts with surfaces
pub trait Material: Send + Sync + std::fmt::Debug {
    /// Get the material's albedo (base color)
    fn albedo(&self) -> Color;
    
    /// Get the material's ambient color
    fn ambient_color(&self) -> Color {
        self.albedo() * 0.1
    }
    
    /// Calculate scattered ray for reflections/refractions
    fn scatter(&self, ray_in: &Ray, hit: &HitInfo) -> Option<Ray>;
    
    /// Get material properties for lighting calculations
    fn get_properties(&self) -> MaterialProperties;
}

/// Material properties for physically-based rendering
#[derive(Debug, Clone)]
pub struct MaterialProperties {
    pub metallic: f32,
    pub roughness: f32,
    pub specular: f32,
    pub emission: Color,
    pub ior: f32, // Index of refraction
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            metallic: 0.0,
            roughness: 0.5,
            specular: 0.5,
            emission: Color::BLACK,
            ior: 1.0,
        }
    }
}

/// Lambertian (diffuse) material
#[derive(Debug)]
pub struct LambertianMaterial {
    pub albedo: Color,
}

impl LambertianMaterial {
    pub fn new(albedo: Color) -> Arc<dyn Material> {
        Arc::new(Self { albedo })
    }
}

impl Material for LambertianMaterial {
    fn albedo(&self) -> Color {
        self.albedo
    }

    fn scatter(&self, _ray_in: &Ray, hit: &HitInfo) -> Option<Ray> {
        use rrte_math::vector::Vec3Ext;
        let scatter_direction = hit.normal + Vec3::random_unit_vector();
        
        // Catch degenerate scatter direction
        let direction = if scatter_direction.length_squared() < 1e-8 {
            hit.normal
        } else {
            scatter_direction
        };
        
        Some(Ray::new(hit.point, direction))
    }

    fn get_properties(&self) -> MaterialProperties {
        MaterialProperties {
            metallic: 0.0,
            roughness: 1.0,
            ..Default::default()
        }
    }
}

/// Metal material with configurable roughness
#[derive(Debug)]
pub struct MetalMaterial {
    pub albedo: Color,
    pub roughness: f32,
}

impl MetalMaterial {
    pub fn new(albedo: Color, roughness: f32) -> Arc<dyn Material> {
        Arc::new(Self { albedo, roughness: roughness.clamp(0.0, 1.0) })
    }
}

impl Material for MetalMaterial {
    fn albedo(&self) -> Color {
        self.albedo
    }

    fn scatter(&self, ray_in: &Ray, hit: &HitInfo) -> Option<Ray> {
        use rrte_math::vector::Vec3Ext;
        let reflected = ray_in.direction.normalize().reflect(hit.normal);
        let scattered = reflected + self.roughness * Vec3::random_in_unit_sphere();
        
        if scattered.dot(hit.normal) > 0.0 {
            Some(Ray::new(hit.point, scattered))
        } else {
            None
        }
    }

    fn get_properties(&self) -> MaterialProperties {
        MaterialProperties {
            metallic: 1.0,
            roughness: self.roughness,
            ..Default::default()
        }
    }
}

/// Dielectric (glass) material
#[derive(Debug)]
pub struct DielectricMaterial {
    pub ior: f32, // Index of refraction
    pub color: Color,
}

impl DielectricMaterial {
    pub fn new(ior: f32) -> Arc<dyn Material> {
        Arc::new(Self { ior, color: Color::WHITE })
    }
    
    pub fn with_color(ior: f32, color: Color) -> Arc<dyn Material> {
        Arc::new(Self { ior, color })
    }
    
    fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
        // Schlick's approximation
        let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        let r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for DielectricMaterial {
    fn albedo(&self) -> Color {
        self.color
    }

    fn scatter(&self, ray_in: &Ray, hit: &HitInfo) -> Option<Ray> {
        use rrte_math::vector::Vec3Ext;
        
        let refraction_ratio = if hit.front_face {
            1.0 / self.ior
        } else {
            self.ior
        };

        let unit_direction = ray_in.direction.normalize();
        let cos_theta = (-unit_direction).dot(hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        
        let direction = if cannot_refract || Self::reflectance(cos_theta, refraction_ratio) > rand::random() {
            unit_direction.reflect(hit.normal)
        } else {
            unit_direction.refract(hit.normal, refraction_ratio).unwrap_or(unit_direction.reflect(hit.normal))
        };

        Some(Ray::new(hit.point, direction))
    }

    fn get_properties(&self) -> MaterialProperties {
        MaterialProperties {
            metallic: 0.0,
            roughness: 0.0,
            ior: self.ior,
            ..Default::default()
        }
    }
}

/// Emissive material that acts as a light source
#[derive(Debug)]
pub struct EmissiveMaterial {
    pub color: Color,
    pub intensity: f32,
}

impl EmissiveMaterial {
    pub fn new(color: Color, intensity: f32) -> Arc<dyn Material> {
        Arc::new(Self { color, intensity })
    }
}

impl Material for EmissiveMaterial {
    fn albedo(&self) -> Color {
        self.color
    }

    fn scatter(&self, _ray_in: &Ray, _hit: &HitInfo) -> Option<Ray> {
        None // Emissive materials don't scatter light
    }

    fn get_properties(&self) -> MaterialProperties {
        MaterialProperties {
            emission: self.color * self.intensity,
            ..Default::default()
        }
    }
}
