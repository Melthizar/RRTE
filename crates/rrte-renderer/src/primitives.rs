use rrte_math::{Ray, Vec3, Transform, HitInfo};
use crate::Material;
use std::sync::Arc;

/// Trait for all renderable objects in the scene
pub trait SceneObject: Send + Sync + std::fmt::Debug {
    /// Test if a ray intersects with this object
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo>;
    
    /// Get the material of this object
    fn material(&self) -> Option<Arc<dyn Material>>;
    
    /// Get the transform of this object
    fn transform(&self) -> &Transform;
    
    /// Set the transform of this object
    fn set_transform(&mut self, transform: Transform);
}

/// Sphere primitive
#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Option<Arc<dyn Material>>,
    pub transform: Transform,
}

impl Sphere {
    /// Create a new sphere
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self {
            center,
            radius,
            material: None,
            transform: Transform::identity(),
        }
    }

    /// Create a new sphere with material
    pub fn with_material(center: Vec3, radius: f32, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material: Some(material),
            transform: Transform::identity(),
        }
    }

    /// Set the material
    pub fn set_material(&mut self, material: Arc<dyn Material>) {
        self.material = Some(material);
    }
}

impl SceneObject for Sphere {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo> {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();
        let mut root = (-half_b - sqrt_d) / a;
        
        if root < t_min || t_max < root {
            root = (-half_b + sqrt_d) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }

        let point = ray.at(root);
        let outward_normal = (point - self.center) / self.radius;        
        Some(HitInfo::new(root, point, outward_normal, &ray))
    }

    fn material(&self) -> Option<Arc<dyn Material>> {
        self.material.clone()
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }
}

/// Plane primitive
#[derive(Debug, Clone)]
pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Option<Arc<dyn Material>>,
    pub transform: Transform,
}

impl Plane {
    /// Create a new plane
    pub fn new(point: Vec3, normal: Vec3) -> Self {
        Self {
            point,
            normal: normal.normalize(),
            material: None,
            transform: Transform::identity(),
        }
    }

    /// Create a new plane with material
    pub fn with_material(point: Vec3, normal: Vec3, material: Arc<dyn Material>) -> Self {
        Self {
            point,
            normal: normal.normalize(),
            material: Some(material),
            transform: Transform::identity(),
        }
    }

    /// Set the material
    pub fn set_material(&mut self, material: Arc<dyn Material>) {
        self.material = Some(material);
    }
}

impl SceneObject for Plane {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo> {
        let denom = self.normal.dot(ray.direction);
        
        // Check if ray is parallel to plane
        if denom.abs() < 1e-6 {
            return None;
        }

        let t = (self.point - ray.origin).dot(self.normal) / denom;
        
        if t < t_min || t > t_max {
            return None;
        }        let point = ray.at(t);
        let normal = if denom < 0.0 { self.normal } else { -self.normal };
        
        Some(HitInfo::new(t, point, normal, &ray))
    }
    fn material(&self) -> Option<Arc<dyn Material>> {
        self.material.clone()
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }
}

/// Triangle primitive
#[derive(Debug, Clone)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
    pub normals: [Vec3; 3],
    pub uvs: [Vec3; 3], // Using Vec3 for future barycentric coordinates
    pub material: Option<Arc<dyn Material>>,
    pub transform: Transform,
}

impl Triangle {
    /// Create a new triangle
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(edge2).normalize();
        
        Self {
            vertices: [v0, v1, v2],
            normals: [normal, normal, normal],
            uvs: [Vec3::ZERO, Vec3::X, Vec3::Y],
            material: None,
            transform: Transform::identity(),
        }
    }

    /// Create a new triangle with material
    pub fn with_material(v0: Vec3, v1: Vec3, v2: Vec3, material: Arc<dyn Material>) -> Self {
        let mut triangle = Self::new(v0, v1, v2);
        triangle.material = Some(material);
        triangle
    }

    /// Set vertex normals
    pub fn set_normals(&mut self, n0: Vec3, n1: Vec3, n2: Vec3) {
        self.normals = [n0.normalize(), n1.normalize(), n2.normalize()];
    }

    /// Set material
    pub fn set_material(&mut self, material: Arc<dyn Material>) {
        self.material = Some(material);
    }
}

impl SceneObject for Triangle {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo> {
        // Möller-Trumbore intersection algorithm
        let edge1 = self.vertices[1] - self.vertices[0];
        let edge2 = self.vertices[2] - self.vertices[0];
        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);

        if a > -1e-6 && a < 1e-6 {
            return None; // Ray is parallel to triangle
        }

        let f = 1.0 / a;
        let s = ray.origin - self.vertices[0];
        let u = f * s.dot(h);

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = s.cross(edge1);
        let v = f * ray.direction.dot(q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * edge2.dot(q);

        if t < t_min || t > t_max {
            return None;
        }        let point = ray.at(t);
          // Interpolate normal using barycentric coordinates
        let w = 1.0 - u - v;
        let normal = (w * self.normals[0] + u * self.normals[1] + v * self.normals[2]).normalize();
        
        Some(HitInfo::new(t, point, normal, &ray))
    }

    fn material(&self) -> Option<Arc<dyn Material>> {
        self.material.clone()
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }
}

/// Cube primitive
#[derive(Debug, Clone)]
pub struct Cube {
    pub center: Vec3,
    pub size: Vec3,
    pub material: Option<Arc<dyn Material>>,
    pub transform: Transform,
}

impl Cube {
    /// Create a new cube
    pub fn new(center: Vec3, size: Vec3) -> Self {
        Self {
            center,
            size,
            material: None,
            transform: Transform::identity(),
        }
    }

    /// Create a unit cube
    pub fn unit() -> Self {
        Self::new(Vec3::ZERO, Vec3::ONE)
    }

    /// Create a new cube with material
    pub fn with_material(center: Vec3, size: Vec3, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            size,
            material: Some(material),
            transform: Transform::identity(),
        }
    }

    /// Set material
    pub fn set_material(&mut self, material: Arc<dyn Material>) {
        self.material = Some(material);
    }
}

impl SceneObject for Cube {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo> {
        let half_size = self.size * 0.5;
        let min = self.center - half_size;
        let max = self.center + half_size;

        let mut t_near = t_min;
        let mut t_far = t_max;
        let mut normal = Vec3::ZERO;

        for i in 0..3 {
            let origin_component = ray.origin[i];
            let direction_component = ray.direction[i];
            let min_component = min[i];
            let max_component = max[i];

            if direction_component.abs() < 1e-6 {
                // Ray is parallel to the slab
                if origin_component < min_component || origin_component > max_component {
                    return None;
                }
            } else {
                let inv_dir = 1.0 / direction_component;
                let mut t1 = (min_component - origin_component) * inv_dir;
                let mut t2 = (max_component - origin_component) * inv_dir;

                let mut face_normal = Vec3::ZERO;
                face_normal[i] = if direction_component > 0.0 { -1.0 } else { 1.0 };

                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                    face_normal = -face_normal;
                }

                if t1 > t_near {
                    t_near = t1;
                    normal = face_normal;
                }

                t_far = t_far.min(t2);

                if t_near > t_far {
                    return None;
                }
            }
        }

        if t_near < t_min || t_near > t_max {
            return None;
        }        let point = ray.at(t_near);
        Some(HitInfo::new(t_near, point, normal, &ray))
    }

    fn material(&self) -> Option<Arc<dyn Material>> {
        self.material.clone()
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }
}
