use glam::Vec3;
use serde::{Deserialize, Serialize};

/// A ray in 3D space with origin and direction
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// Create a new ray
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Get a point along the ray at parameter t
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t * self.direction
    }

    /// Transform the ray by a matrix
    pub fn transform(&self, transform: &glam::Mat4) -> Self {
        let origin = transform.transform_point3(self.origin);
        let direction = transform.transform_vector3(self.direction);
        Self::new(origin, direction)
    }
}

/// Hit information for ray intersections
#[derive(Debug, Clone, PartialEq)]
pub struct HitInfo {
    pub t: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub front_face: bool,
    pub material_id: Option<u32>,
}

impl HitInfo {
    /// Create new hit info and determine front face
    pub fn new(t: f32, point: Vec3, outward_normal: Vec3, ray: &Ray) -> Self {
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face { outward_normal } else { -outward_normal };
        
        Self {
            t,
            point,
            normal,
            front_face,
            material_id: None,
        }
    }

    /// Set the material ID
    pub fn with_material(mut self, material_id: u32) -> Self {
        self.material_id = Some(material_id);
        self
    }
}
