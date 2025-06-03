use rrte_math::{Transform, Mat4, Vec3, Ray, Quat};

/// Camera projection types
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectionType {
    Perspective {
        fov: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}

/// Camera component for rendering
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera transform in world space
    pub transform: Transform,
    /// Projection information
    pub projection: ProjectionType,
    /// Whether the camera is currently active
    pub is_active: bool,
}

impl Camera {
    /// Create a new perspective camera
    pub fn new_perspective(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            transform: Transform::identity(),
            projection: ProjectionType::Perspective {
                fov,
                aspect_ratio,
                near,
                far,
            },
            is_active: true,
        }
    }

    /// Create a new orthographic camera
    pub fn new_orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self {
            transform: Transform::identity(),
            projection: ProjectionType::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            },
            is_active: true,
        }
    }    /// Get the view matrix
    pub fn view_matrix(&self) -> Mat4 {
        self.transform.to_matrix().inverse()
    }

    /// Get the projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        match &self.projection {
            ProjectionType::Perspective { fov, aspect_ratio, near, far } => {
                Mat4::perspective_rh(*fov, *aspect_ratio, *near, *far)
            },
            ProjectionType::Orthographic { left, right, bottom, top, near, far } => {
                Mat4::orthographic_rh(*left, *right, *bottom, *top, *near, *far)
            }
        }
    }

    /// Get the view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Look at a target position
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        // Ensure self.transform.position is set before calling this
        let forward = (target - self.transform.position).normalize();
        let right = forward.cross(up).normalize();
        let _actual_up = right.cross(forward); // Recalculate up vector to be orthogonal
        self.transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, forward);
        // Note: A more robust look_at might involve creating a rotation matrix
        // from the basis vectors (right, actual_up, -forward) and then converting to Quat.
        // Quat::from_rotation_arc might have issues if forward is parallel to Vec3::NEG_Z.
        // For now, this matches the rrte_core::Camera implementation.
    }

    /// Generate a ray from screen coordinates (normalized 0-1)
    pub fn generate_ray(&self, u: f32, v: f32) -> Ray {
        // Convert from screen space to world space
        let ndc_x = 2.0 * u - 1.0;
        let ndc_y = 1.0 - 2.0 * v; // Flip Y for screen coordinates
        
        match &self.projection {
            ProjectionType::Perspective { fov, aspect_ratio, .. } => {
                let half_height = (fov * 0.5).tan();
                let half_width = aspect_ratio * half_height;
                
                let world_x = ndc_x * half_width;
                let world_y = ndc_y * half_height;
                
                // Direction in camera space (looking down -Z)
                let camera_dir = Vec3::new(world_x, world_y, -1.0).normalize();
                  // Transform to world space
                let world_origin = self.transform.position;
                let world_direction = self.transform.rotation * camera_dir;
                
                Ray::new(world_origin, world_direction)
            },
            ProjectionType::Orthographic { left, right, bottom, top, .. } => {
                let world_x = left + (right - left) * u;
                let world_y = bottom + (top - bottom) * v;
                
                // For orthographic, all rays are parallel
                let camera_origin = Vec3::new(world_x, world_y, 0.0);
                let camera_dir = Vec3::new(0.0, 0.0, -1.0);
                  // Transform to world space
                let world_origin = self.transform.to_matrix().transform_point3(camera_origin);
                let world_direction = self.transform.rotation * camera_dir;
                
                Ray::new(world_origin, world_direction)
            }
        }
    }
}
