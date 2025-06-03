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
        // Transform ray to local space
        let inv_transform = self.transform.inverse_matrix();
        let local_ray = Ray::new(
            inv_transform.transform_point3(ray.origin),
            inv_transform.transform_vector3(ray.direction).normalize()
        );
        
        let min_bounds = self.center - self.size * 0.5;
        let max_bounds = self.center + self.size * 0.5;
        
        let mut t_near = t_min;
        let mut t_far = t_max;
        let mut normal = Vec3::ZERO;
        
        for i in 0..3 {
            let axis = match i {
                0 => Vec3::X,
                1 => Vec3::Y,
                _ => Vec3::Z,
            };
            
            let origin_component = local_ray.origin.dot(axis);
            let direction_component = local_ray.direction.dot(axis);
            let min_component = min_bounds.dot(axis);
            let max_component = max_bounds.dot(axis);
            
            if direction_component.abs() < 1e-6 {
                // Ray is parallel to the slab
                if origin_component < min_component || origin_component > max_component {
                    return None;
                }
            } else {
                let t1 = (min_component - origin_component) / direction_component;
                let t2 = (max_component - origin_component) / direction_component;
                
                let (t_min_slab, t_max_slab) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
                
                if t_min_slab > t_near {
                    t_near = t_min_slab;
                    normal = if t1 < t2 { -axis } else { axis };
                }
                
                if t_max_slab < t_far {
                    t_far = t_max_slab;
                }
                
                if t_near > t_far {
                    return None;
                }
            }
        }
        
        let t = if t_near >= t_min { t_near } else { t_far };
        if t < t_min || t > t_max {
            return None;
        }
        
        let local_point = local_ray.at(t);
        let world_point = self.transform.to_matrix().transform_point3(local_point);
        let world_normal = self.transform.to_matrix().transform_vector3(normal).normalize();
        
        Some(HitInfo::new(t, world_point, world_normal, ray))
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

/// Cylinder primitive
#[derive(Debug, Clone)]
pub struct Cylinder {
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
    pub material: Option<Arc<dyn Material>>,
    pub transform: Transform,
}

impl Cylinder {
    /// Create a new cylinder
    pub fn new(center: Vec3, radius: f32, height: f32) -> Self {
        Self {
            center,
            radius,
            height,
            material: None,
            transform: Transform::identity(),
        }
    }

    /// Create a new cylinder with material
    pub fn with_material(center: Vec3, radius: f32, height: f32, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            height,
            material: Some(material),
            transform: Transform::identity(),
        }
    }

    /// Set the material
    pub fn set_material(&mut self, material: Arc<dyn Material>) {
        self.material = Some(material);
    }
}

impl SceneObject for Cylinder {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo> {
        // Transform ray to local space
        let inv_transform = self.transform.inverse_matrix();
        let local_ray = Ray::new(
            inv_transform.transform_point3(ray.origin),
            inv_transform.transform_vector3(ray.direction).normalize()
        );
        
        let oc = local_ray.origin - self.center;
        let half_height = self.height * 0.5;
        
        // Check intersection with infinite cylinder (ignoring Y)
        let a = local_ray.direction.x * local_ray.direction.x + local_ray.direction.z * local_ray.direction.z;
        let b = 2.0 * (oc.x * local_ray.direction.x + oc.z * local_ray.direction.z);
        let c = oc.x * oc.x + oc.z * oc.z - self.radius * self.radius;
        
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }
        
        let sqrt_d = discriminant.sqrt();
        let t1 = (-b - sqrt_d) / (2.0 * a);
        let t2 = (-b + sqrt_d) / (2.0 * a);
        
        // Check both intersection points
        for &t in &[t1, t2] {
            if t >= t_min && t <= t_max {
                let point = local_ray.at(t);
                let y_dist = (point.y - self.center.y).abs();
                
                if y_dist <= half_height {
                    let world_point = self.transform.to_matrix().transform_point3(point);
                    let local_normal = Vec3::new(
                        (point.x - self.center.x) / self.radius,
                        0.0,
                        (point.z - self.center.z) / self.radius
                    );
                    let world_normal = self.transform.to_matrix().transform_vector3(local_normal).normalize();
                    
                    return Some(HitInfo::new(t, world_point, world_normal, ray));
                }
            }
        }
        
        None
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

/// Cone primitive
#[derive(Debug, Clone)]
pub struct Cone {
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
    pub material: Option<Arc<dyn Material>>,
    pub transform: Transform,
}

impl Cone {
    /// Create a new cone
    pub fn new(center: Vec3, radius: f32, height: f32) -> Self {
        Self {
            center,
            radius,
            height,
            material: None,
            transform: Transform::identity(),
        }
    }

    /// Create a new cone with material
    pub fn with_material(center: Vec3, radius: f32, height: f32, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            height,
            material: Some(material),
            transform: Transform::identity(),
        }
    }

    /// Set the material
    pub fn set_material(&mut self, material: Arc<dyn Material>) {
        self.material = Some(material);
    }
}

impl SceneObject for Cone {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo> {
        // Transform ray to local space
        let inv_transform = self.transform.inverse_matrix();
        let local_ray = Ray::new(
            inv_transform.transform_point3(ray.origin),
            inv_transform.transform_vector3(ray.direction).normalize()
        );
        
        let oc = local_ray.origin - self.center;
        let half_height = self.height * 0.5;
        let k = self.radius / self.height;
        let k2 = k * k;
        
        // Cone equation: x² + z² = (k * (h/2 - y))²
        let a = local_ray.direction.x * local_ray.direction.x + local_ray.direction.z * local_ray.direction.z - k2 * local_ray.direction.y * local_ray.direction.y;
        let b = 2.0 * (oc.x * local_ray.direction.x + oc.z * local_ray.direction.z - k2 * (oc.y - half_height) * local_ray.direction.y);
        let c = oc.x * oc.x + oc.z * oc.z - k2 * (oc.y - half_height) * (oc.y - half_height);
        
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }
        
        let sqrt_d = discriminant.sqrt();
        let t1 = (-b - sqrt_d) / (2.0 * a);
        let t2 = (-b + sqrt_d) / (2.0 * a);
        
        // Check both intersection points
        for &t in &[t1, t2] {
            if t >= t_min && t <= t_max {
                let point = local_ray.at(t);
                let y_local = point.y - self.center.y;
                
                if y_local >= -half_height && y_local <= half_height {
                    let world_point = self.transform.to_matrix().transform_point3(point);
                    
                    // Calculate normal
                    let r = (point.x * point.x + point.z * point.z).sqrt();
                    let local_normal = Vec3::new(
                        point.x / r,
                        k,
                        point.z / r
                    ).normalize();
                    let world_normal = self.transform.to_matrix().transform_vector3(local_normal).normalize();
                    
                    return Some(HitInfo::new(t, world_point, world_normal, ray));
                }
            }
        }
        
        None
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

/// Capsule primitive (rounded cylinder)
#[derive(Debug, Clone)]
pub struct Capsule {
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
    pub material: Option<Arc<dyn Material>>,
    pub transform: Transform,
}

impl Capsule {
    /// Create a new capsule
    pub fn new(center: Vec3, radius: f32, height: f32) -> Self {
        Self {
            center,
            radius,
            height,
            material: None,
            transform: Transform::identity(),
        }
    }

    /// Create a new capsule with material
    pub fn with_material(center: Vec3, radius: f32, height: f32, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            height,
            material: Some(material),
            transform: Transform::identity(),
        }
    }

    /// Set the material
    pub fn set_material(&mut self, material: Arc<dyn Material>) {
        self.material = Some(material);
    }
}

impl SceneObject for Capsule {
    fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitInfo> {
        // Transform ray to local space
        let inv_transform = self.transform.inverse_matrix();
        let local_ray = Ray::new(
            inv_transform.transform_point3(ray.origin),
            inv_transform.transform_vector3(ray.direction).normalize()
        );
        
        let half_height = self.height * 0.5;
        let top_center = self.center + Vec3::new(0.0, half_height, 0.0);
        let bottom_center = self.center - Vec3::new(0.0, half_height, 0.0);
        
        let mut closest_t = f32::INFINITY;
        let mut closest_hit: Option<HitInfo> = None;
        
        // Check intersection with top hemisphere
        let oc_top = local_ray.origin - top_center;
        let a = local_ray.direction.length_squared();
        let half_b = oc_top.dot(local_ray.direction);
        let c = oc_top.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        
        if discriminant >= 0.0 {
            let sqrt_d = discriminant.sqrt();
            let t1 = (-half_b - sqrt_d) / a;
            let t2 = (-half_b + sqrt_d) / a;
            
            for &t in &[t1, t2] {
                if t >= t_min && t <= t_max && t < closest_t {
                    let point = local_ray.at(t);
                    if point.y >= self.center.y {
                        let world_point = self.transform.to_matrix().transform_point3(point);
                        let local_normal = (point - top_center).normalize();
                        let world_normal = self.transform.to_matrix().transform_vector3(local_normal).normalize();
                        closest_t = t;
                        closest_hit = Some(HitInfo::new(t, world_point, world_normal, ray));
                    }
                }
            }
        }
        
        // Check intersection with bottom hemisphere
        let oc_bottom = local_ray.origin - bottom_center;
        let half_b = oc_bottom.dot(local_ray.direction);
        let c = oc_bottom.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        
        if discriminant >= 0.0 {
            let sqrt_d = discriminant.sqrt();
            let t1 = (-half_b - sqrt_d) / a;
            let t2 = (-half_b + sqrt_d) / a;
            
            for &t in &[t1, t2] {
                if t >= t_min && t <= t_max && t < closest_t {
                    let point = local_ray.at(t);
                    if point.y <= self.center.y {
                        let world_point = self.transform.to_matrix().transform_point3(point);
                        let local_normal = (point - bottom_center).normalize();
                        let world_normal = self.transform.to_matrix().transform_vector3(local_normal).normalize();
                        closest_t = t;
                        closest_hit = Some(HitInfo::new(t, world_point, world_normal, ray));
                    }
                }
            }
        }
        
        // Check intersection with cylindrical body
        let oc = local_ray.origin - self.center;
        let a_cyl = local_ray.direction.x * local_ray.direction.x + local_ray.direction.z * local_ray.direction.z;
        let b_cyl = 2.0 * (oc.x * local_ray.direction.x + oc.z * local_ray.direction.z);
        let c_cyl = oc.x * oc.x + oc.z * oc.z - self.radius * self.radius;
        
        let discriminant = b_cyl * b_cyl - 4.0 * a_cyl * c_cyl;
        if discriminant >= 0.0 {
            let sqrt_d = discriminant.sqrt();
            let t1 = (-b_cyl - sqrt_d) / (2.0 * a_cyl);
            let t2 = (-b_cyl + sqrt_d) / (2.0 * a_cyl);
            
            for &t in &[t1, t2] {
                if t >= t_min && t <= t_max && t < closest_t {
                    let point = local_ray.at(t);
                    let y_dist = (point.y - self.center.y).abs();
                    
                    if y_dist <= half_height {
                        let world_point = self.transform.to_matrix().transform_point3(point);
                        let local_normal = Vec3::new(
                            (point.x - self.center.x) / self.radius,
                            0.0,
                            (point.z - self.center.z) / self.radius
                        );
                        let world_normal = self.transform.to_matrix().transform_vector3(local_normal).normalize();
                        closest_t = t;
                        closest_hit = Some(HitInfo::new(t, world_point, world_normal, ray));
                    }
                }
            }
        }
        
        closest_hit
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
