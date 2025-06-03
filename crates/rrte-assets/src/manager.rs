use crate::{Asset, UntypedHandle, LoaderRegistry};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use anyhow::Result;

/// Manages loaded assets and their handles
pub struct AssetManager {
    assets: Arc<RwLock<HashMap<UntypedHandle, Arc<dyn Asset>>>>,
    loader_registry: LoaderRegistry,
    next_handle: UntypedHandle,
}

impl AssetManager {    pub fn new() -> Self {
        Self {
            assets: Arc::new(RwLock::new(HashMap::new())),
            loader_registry: LoaderRegistry::new(),
            next_handle: UntypedHandle::new(0),
        }
    }    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<UntypedHandle> {
        let asset = self.loader_registry.load_asset(path.as_ref())?;
        let handle = self.next_handle;
        self.next_handle = UntypedHandle::new(self.next_handle.id() + 1);

        let mut assets = self.assets.write().unwrap();
        assets.insert(handle, Arc::from(asset));

        Ok(handle)
    }    pub fn get(&self, handle: UntypedHandle) -> Option<Arc<dyn Asset>> {
        let assets = self.assets.read().unwrap();
        assets.get(&handle).cloned()
    }

    pub fn unload(&mut self, handle: UntypedHandle) {
        let mut assets = self.assets.write().unwrap();
        assets.remove(&handle);
    }

    pub fn register_loader<T: Asset + 'static>(&mut self, loader: Box<dyn crate::AssetLoader<T>>) {
        self.loader_registry.register_loader(loader);
    }    pub fn is_loaded(&self, handle: UntypedHandle) -> bool {
        let assets = self.assets.read().unwrap();
        assets.contains_key(&handle)
    }

    pub fn loaded_assets(&self) -> Vec<UntypedHandle> {
        let assets = self.assets.read().unwrap();
        assets.keys().copied().collect()
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}
