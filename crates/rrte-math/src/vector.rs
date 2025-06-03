// Additional vector utilities beyond glam
pub use glam::{Vec2, Vec3, Vec4};

/// Vector extension traits
pub trait Vec3Ext {
    fn reflect(&self, normal: Vec3) -> Vec3;
    fn refract(&self, normal: Vec3, eta: f32) -> Option<Vec3>;
    fn random_in_unit_sphere() -> Vec3;
    fn random_unit_vector() -> Vec3;
    fn random_in_hemisphere(normal: Vec3) -> Vec3;
    fn random() -> Vec3;
    fn random_range(min: f32, max: f32) -> Vec3;
}

impl Vec3Ext for Vec3 {
    /// Reflect vector around a normal
    fn reflect(&self, normal: Vec3) -> Vec3 {
        *self - 2.0 * self.dot(normal) * normal
    }

    /// Refract vector through a surface
    fn refract(&self, normal: Vec3, eta: f32) -> Option<Vec3> {
        let cos_theta = (-*self).dot(normal).min(1.0);
        let r_out_perp = eta * (*self + cos_theta * normal);
        let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * normal;
        
        if r_out_perp.length_squared() < 1.0 {
            Some(r_out_perp + r_out_parallel)
        } else {
            None
        }
    }

    /// Generate random vector in unit sphere
    fn random_in_unit_sphere() -> Vec3 {
        loop {
            let p = Vec3::new(
                rand::random::<f32>() * 2.0 - 1.0,
                rand::random::<f32>() * 2.0 - 1.0,
                rand::random::<f32>() * 2.0 - 1.0,
            );
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }

    /// Generate random unit vector
    fn random_unit_vector() -> Vec3 {
        Self::random_in_unit_sphere().normalize()
    }    /// Generate random vector in hemisphere
    fn random_in_hemisphere(normal: Vec3) -> Vec3 {
        let in_unit_sphere = Self::random_in_unit_sphere();
        if in_unit_sphere.dot(normal) > 0.0 {
            in_unit_sphere
        } else {
            -in_unit_sphere
        }
    }

    /// Generate random vector with components in [0, 1)
    fn random() -> Vec3 {
        Vec3::new(
            rand::random::<f32>(),
            rand::random::<f32>(),
            rand::random::<f32>(),
        )
    }

    /// Generate random vector with components in [min, max)
    fn random_range(min: f32, max: f32) -> Vec3 {
        Vec3::new(
            min + (max - min) * rand::random::<f32>(),
            min + (max - min) * rand::random::<f32>(),
            min + (max - min) * rand::random::<f32>(),
        )
    }
}
