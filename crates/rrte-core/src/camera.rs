use rrte_math::{Transform, Mat4, Vec3, Ray, Quat};
use serde::{Deserialize, Serialize};

/// Camera projection types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub transform: Transform,
    pub projection: ProjectionType,
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
    }

    /// Get the view matrix
    pub fn view_matrix(&self) -> Mat4 {
        self.transform.inverse_matrix()
    }

    /// Get the projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        match &self.projection {
            ProjectionType::Perspective { fov, aspect_ratio, near, far } => {
                Mat4::perspective_rh(*fov, *aspect_ratio, *near, *far)
            }
            ProjectionType::Orthographic { left, right, bottom, top, near, far } => {
                Mat4::orthographic_rh(*left, *right, *bottom, *top, *near, *far)
            }
        }
    }

    /// Get the view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Generate a ray from screen coordinates (normalized device coordinates)
    pub fn screen_to_ray(&self, ndc_x: f32, ndc_y: f32) -> Ray {
        match &self.projection {
            ProjectionType::Perspective { .. } => {
                // For perspective projection
                let direction = Vec3::new(ndc_x, ndc_y, -1.0).normalize();
                let world_direction = self.transform.transform_vector(direction);
                Ray::new(self.transform.position, world_direction)
            }
            ProjectionType::Orthographic { left, right, bottom, top, .. } => {
                // For orthographic projection
                let world_x = left + (ndc_x + 1.0) * 0.5 * (right - left);
                let world_y = bottom + (ndc_y + 1.0) * 0.5 * (top - bottom);
                let origin = self.transform.position + Vec3::new(world_x, world_y, 0.0);
                Ray::new(origin, self.transform.forward())
            }
        }
    }

    /// Look at a target position
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = (target - self.transform.position).normalize();
        let right = forward.cross(up).normalize();
        let _up = right.cross(forward);        // Create rotation from forward direction  
        self.transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, forward);
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new_perspective(45.0_f32.to_radians(), 16.0/9.0, 0.1, 100.0)
    }
}
