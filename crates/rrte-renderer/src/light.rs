use rrte_math::{Vec3, Color, Transform};
use serde::{Deserialize, Serialize};

/// Trait for all light sources
pub trait Light: Send + Sync + std::fmt::Debug {
    /// Get the light's contribution at a given point
    fn illuminate(&self, point: Vec3, normal: Vec3) -> LightContribution;
    
    /// Get the light's position
    fn position(&self) -> Vec3;
    
    /// Get the light's color
    fn color(&self) -> Color;
    
    /// Get the light's intensity
    fn intensity(&self) -> f32;
    
    /// Check if the light affects a given point
    fn affects_point(&self, point: Vec3) -> bool;
    
    /// Get the transform of the light
    fn transform(&self) -> &Transform;
    
    /// Set the transform of the light
    fn set_transform(&mut self, transform: Transform);
}

/// Light contribution result
#[derive(Debug, Clone)]
pub struct LightContribution {
    pub color: Color,
    pub direction: Vec3,
    pub distance: f32,
    pub attenuation: f32,
}

impl LightContribution {
    pub fn new(color: Color, direction: Vec3, distance: f32, attenuation: f32) -> Self {
        Self {
            color,
            direction,
            distance,
            attenuation,
        }
    }

    pub fn none() -> Self {
        Self {
            color: Color::BLACK,
            direction: Vec3::ZERO,
            distance: 0.0,
            attenuation: 0.0,
        }
    }
}

/// Directional light (like sunlight)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub transform: Transform,
}

impl DirectionalLight {
    /// Create a new directional light
    pub fn new(direction: Vec3, color: Color, intensity: f32) -> Self {
        Self {
            direction: direction.normalize(),
            color,
            intensity,
            transform: Transform::identity(),
        }
    }

    /// Create a default sun-like directional light
    pub fn sun() -> Self {
        Self::new(
            Vec3::new(-0.3, -1.0, -0.3).normalize(),
            Color::new(1.0, 0.95, 0.8, 1.0),
            5.0,
        )
    }
}

impl Light for DirectionalLight {
    fn illuminate(&self, _point: Vec3, _normal: Vec3) -> LightContribution {
        LightContribution::new(
            self.color * self.intensity,
            -self.direction,
            f32::INFINITY,
            1.0,
        )
    }

    fn position(&self) -> Vec3 {
        // Directional lights don't have a position
        Vec3::ZERO
    }

    fn color(&self) -> Color {
        self.color
    }

    fn intensity(&self) -> f32 {
        self.intensity
    }

    fn affects_point(&self, _point: Vec3) -> bool {
        true // Directional lights affect all points
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }
}

/// Point light (omnidirectional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub linear_attenuation: f32,
    pub quadratic_attenuation: f32,
    pub transform: Transform,
}

impl PointLight {
    /// Create a new point light
    pub fn new(position: Vec3, color: Color, intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
            range: 100.0,
            linear_attenuation: 0.09,
            quadratic_attenuation: 0.032,
            transform: Transform::identity(),
        }
    }

    /// Create a new point light with attenuation
    pub fn with_attenuation(
        position: Vec3,
        color: Color,
        intensity: f32,
        range: f32,
        linear: f32,
        quadratic: f32,
    ) -> Self {
        Self {
            position,
            color,
            intensity,
            range,
            linear_attenuation: linear,
            quadratic_attenuation: quadratic,
            transform: Transform::identity(),
        }
    }

    /// Calculate attenuation based on distance
    fn calculate_attenuation(&self, distance: f32) -> f32 {
        if distance > self.range {
            return 0.0;
        }
        
        let attenuation = 1.0 / (1.0 + self.linear_attenuation * distance + 
                                self.quadratic_attenuation * distance * distance);
        attenuation.max(0.0)
    }
}

impl Light for PointLight {
    fn illuminate(&self, point: Vec3, _normal: Vec3) -> LightContribution {
        let light_vector = self.position - point;
        let distance = light_vector.length();
        let direction = light_vector.normalize();
        let attenuation = self.calculate_attenuation(distance);
        
        LightContribution::new(
            self.color * self.intensity,
            direction,
            distance,
            attenuation,
        )
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn color(&self) -> Color {
        self.color
    }

    fn intensity(&self) -> f32 {
        self.intensity
    }

    fn affects_point(&self, point: Vec3) -> bool {
        let distance = (self.position - point).length();
        distance <= self.range
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }
}

/// Spot light (cone-shaped light)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotLight {
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    pub inner_angle: f32, // In radians
    pub outer_angle: f32, // In radians
    pub linear_attenuation: f32,
    pub quadratic_attenuation: f32,
    pub transform: Transform,
}

impl SpotLight {
    /// Create a new spot light
    pub fn new(
        position: Vec3,
        direction: Vec3,
        color: Color,
        intensity: f32,
        inner_angle: f32,
        outer_angle: f32,
    ) -> Self {
        Self {
            position,
            direction: direction.normalize(),
            color,
            intensity,
            range: 100.0,
            inner_angle,
            outer_angle,
            linear_attenuation: 0.09,
            quadratic_attenuation: 0.032,
            transform: Transform::identity(),
        }
    }

    /// Calculate attenuation based on distance
    fn calculate_distance_attenuation(&self, distance: f32) -> f32 {
        if distance > self.range {
            return 0.0;
        }
        
        let attenuation = 1.0 / (1.0 + self.linear_attenuation * distance + 
                                self.quadratic_attenuation * distance * distance);
        attenuation.max(0.0)
    }

    /// Calculate angular attenuation based on angle from light direction
    fn calculate_angular_attenuation(&self, direction_to_point: Vec3) -> f32 {
        let angle = self.direction.dot(-direction_to_point).acos();
        
        if angle > self.outer_angle {
            0.0
        } else if angle < self.inner_angle {
            1.0
        } else {
            // Smooth falloff between inner and outer angles
            let falloff = (self.outer_angle - angle) / (self.outer_angle - self.inner_angle);
            falloff * falloff
        }
    }
}

impl Light for SpotLight {
    fn illuminate(&self, point: Vec3, _normal: Vec3) -> LightContribution {
        let light_vector = self.position - point;
        let distance = light_vector.length();
        let direction = light_vector.normalize();
        
        let distance_attenuation = self.calculate_distance_attenuation(distance);
        let angular_attenuation = self.calculate_angular_attenuation(direction);
        let total_attenuation = distance_attenuation * angular_attenuation;
        
        LightContribution::new(
            self.color * self.intensity,
            direction,
            distance,
            total_attenuation,
        )
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn color(&self) -> Color {
        self.color
    }

    fn intensity(&self) -> f32 {
        self.intensity
    }

    fn affects_point(&self, point: Vec3) -> bool {
        let light_vector = self.position - point;
        let distance = light_vector.length();
        
        if distance > self.range {
            return false;
        }
        
        let direction = light_vector.normalize();
        let angle = self.direction.dot(-direction).acos();
        angle <= self.outer_angle
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }
}

/// Ambient light (uniform lighting)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbientLight {
    pub color: Color,
    pub intensity: f32,
    pub transform: Transform,
}

impl AmbientLight {
    /// Create a new ambient light
    pub fn new(color: Color, intensity: f32) -> Self {
        Self {
            color,
            intensity,
            transform: Transform::identity(),
        }
    }

    /// Create a default ambient light
    pub fn default_ambient() -> Self {
        Self::new(Color::new(0.2, 0.2, 0.3, 1.0), 0.3)
    }
}

impl Light for AmbientLight {
    fn illuminate(&self, _point: Vec3, _normal: Vec3) -> LightContribution {
        LightContribution::new(
            self.color * self.intensity,
            Vec3::ZERO,
            0.0,
            1.0,
        )
    }

    fn position(&self) -> Vec3 {
        Vec3::ZERO
    }

    fn color(&self) -> Color {
        self.color
    }

    fn intensity(&self) -> f32 {
        self.intensity
    }

    fn affects_point(&self, _point: Vec3) -> bool {
        true
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }
}
