use std::any::Any;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

/// Trait for assets that can be loaded and managed
pub trait Asset: Any + Send + Sync + 'static {
    /// Get the asset type name
    fn type_name(&self) -> &'static str;
    
    /// Serialize the asset to bytes
    fn serialize(&self) -> anyhow::Result<Vec<u8>>;
    
    /// Get asset as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Asset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub path: String,
    pub asset_type: String,
    pub size: u64,
    pub created: std::time::SystemTime,
    pub modified: std::time::SystemTime,
    pub dependencies: Vec<String>,
}

/// Image asset
#[derive(Debug, Clone)]
pub struct ImageAsset {
    pub data: Arc<image::DynamicImage>,
    pub metadata: AssetMetadata,
}

impl Asset for ImageAsset {
    fn type_name(&self) -> &'static str {
        "Image"
    }

    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.data.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;
        Ok(buffer)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Mesh asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshAsset {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub metadata: AssetMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub position: rrte_math::Vec3,
    pub normal: rrte_math::Vec3,
    pub uv: rrte_math::Vec2,
    pub color: rrte_math::Color,
}

impl Asset for MeshAsset {
    fn type_name(&self) -> &'static str {
        "Mesh"
    }

    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Material asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialAsset {
    pub name: String,
    pub albedo: rrte_math::Color,
    pub metallic: f32,
    pub roughness: f32,
    pub specular: f32,
    pub emission: rrte_math::Color,
    pub ior: f32,
    pub albedo_texture: Option<String>,
    pub normal_texture: Option<String>,
    pub metallic_texture: Option<String>,
    pub roughness_texture: Option<String>,
    pub metadata: AssetMetadata,
}

impl Asset for MaterialAsset {
    fn type_name(&self) -> &'static str {
        "Material"
    }

    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Scene asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneAsset {
    pub name: String,
    pub entities: Vec<SceneEntity>,
    pub lights: Vec<SceneLight>,
    pub camera: SceneCamera,
    pub metadata: AssetMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneEntity {
    pub name: String,
    pub transform: rrte_math::Transform,
    pub mesh: Option<String>,
    pub material: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneLight {
    pub name: String,
    pub light_type: String,
    pub position: rrte_math::Vec3,
    pub direction: rrte_math::Vec3,
    pub color: rrte_math::Color,
    pub intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneCamera {
    pub transform: rrte_math::Transform,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Asset for SceneAsset {
    fn type_name(&self) -> &'static str {
        "Scene"
    }

    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
