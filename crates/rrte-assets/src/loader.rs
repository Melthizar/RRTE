use crate::{Asset, AssetHandle};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use anyhow::Result;

/// Trait for loading assets from files
pub trait AssetLoader<T: Asset>: Send + Sync {
    fn load(&self, path: &Path) -> Result<T>;
    fn extensions(&self) -> &[&str];
}

/// Registry for asset loaders
pub struct LoaderRegistry {
    loaders: HashMap<String, Arc<dyn AssetLoaderDyn>>,
}

trait AssetLoaderDyn: Send + Sync {
    fn load_asset(&self, path: &Path) -> Result<Box<dyn Asset>>;
    fn extensions(&self) -> &[&str];
}

impl<T: Asset + 'static> AssetLoaderDyn for Box<dyn AssetLoader<T>> {
    fn load_asset(&self, path: &Path) -> Result<Box<dyn Asset>> {
        let asset = self.load(path)?;
        Ok(Box::new(asset))
    }

    fn extensions(&self) -> &[&str] {
        AssetLoader::extensions(self.as_ref())
    }
}

impl LoaderRegistry {
    pub fn new() -> Self {
        Self {
            loaders: HashMap::new(),
        }
    }    pub fn register_loader<T: Asset + 'static>(&mut self, loader: Box<dyn AssetLoader<T>>) {
        let extensions: Vec<String> = loader.extensions().iter().map(|s| s.to_string()).collect();
        let arc_loader = Arc::new(loader) as Arc<dyn AssetLoaderDyn>;
        for ext in extensions {
            self.loaders.insert(ext, arc_loader.clone());
        }
    }

    pub fn load_asset(&self, path: &Path) -> Result<Box<dyn Asset>> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension found"))?;

        let loader = self.loaders.get(extension)
            .ok_or_else(|| anyhow::anyhow!("No loader found for extension: {}", extension))?;

        loader.load_asset(path)
    }
}

impl Default for LoaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
