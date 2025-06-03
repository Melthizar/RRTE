// Matrix utilities beyond glam
pub use glam::{Mat3, Mat4};

/// Matrix extension traits
pub trait Mat4Ext {
    fn look_at_rh(eye: glam::Vec3, center: glam::Vec3, up: glam::Vec3) -> Mat4;
    fn perspective_rh(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4;
    fn orthographic_rh(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4;
}

impl Mat4Ext for Mat4 {
    /// Create a right-handed look-at matrix
    fn look_at_rh(eye: glam::Vec3, center: glam::Vec3, up: glam::Vec3) -> Mat4 {
        Mat4::look_at_rh(eye, center, up)
    }

    /// Create a right-handed perspective projection matrix
    fn perspective_rh(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        Mat4::perspective_rh(fov_y, aspect, near, far)
    }

    /// Create a right-handed orthographic projection matrix
    fn orthographic_rh(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
        Mat4::orthographic_rh(left, right, bottom, top, near, far)
    }
}
