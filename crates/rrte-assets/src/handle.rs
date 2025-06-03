use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use crate::Asset;

/// Handle to an asset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetHandle<T: Asset> {
    id: u64,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Asset> AssetHandle<T> {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

/// Handle generator for creating unique asset handles
pub struct HandleGenerator {
    next_id: AtomicU64,
}

impl HandleGenerator {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
        }
    }

    pub fn generate<T: Asset>(&self) -> AssetHandle<T> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        AssetHandle::new(id)
    }
}

impl Default for HandleGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Type-erased asset handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UntypedHandle {
    id: u64,
}

impl UntypedHandle {
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn typed<T: Asset>(self) -> AssetHandle<T> {
        AssetHandle::new(self.id)
    }
}

impl<T: Asset> From<AssetHandle<T>> for UntypedHandle {
    fn from(handle: AssetHandle<T>) -> Self {
        UntypedHandle::new(handle.id())
    }
}
