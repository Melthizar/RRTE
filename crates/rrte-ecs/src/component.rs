use crate::Entity;
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Trait for components that can be attached to entities
pub trait Component: Any + Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Blanket implementation for all types that meet the requirements
impl<T: Any + Send + Sync + 'static> Component for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Storage for a specific component type
pub struct ComponentStorage {
    components: HashMap<u64, Box<dyn Component>>,
    type_id: TypeId,
}

impl ComponentStorage {
    pub fn new<T: Component>() -> Self {
        Self {
            components: HashMap::new(),
            type_id: TypeId::of::<T>(),
        }
    }

    /// Add a component for an entity
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) -> Option<Box<dyn Component>> {
        if TypeId::of::<T>() != self.type_id {
            return None;
        }
        self.components.insert(entity.id(), Box::new(component))
    }

    /// Get a component for an entity
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        if TypeId::of::<T>() != self.type_id {
            return None;
        }
        self.components
            .get(&entity.id())
            .and_then(|comp| comp.as_any().downcast_ref::<T>())
    }

    /// Get a mutable component for an entity
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if TypeId::of::<T>() != self.type_id {
            return None;
        }
        self.components
            .get_mut(&entity.id())
            .and_then(|comp| comp.as_any_mut().downcast_mut::<T>())
    }

    /// Remove a component for an entity
    pub fn remove(&mut self, entity: Entity) -> Option<Box<dyn Component>> {
        self.components.remove(&entity.id())
    }

    /// Check if an entity has this component
    pub fn has_component(&self, entity: Entity) -> bool {
        self.components.contains_key(&entity.id())
    }    /// Get all entities with this component
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.components.keys().map(|&id| Entity::new(id))
    }
}

/// Manager for all component storages
pub struct ComponentManager {
    storages: HashMap<TypeId, ComponentStorage>,
}

impl ComponentManager {
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
        }
    }

    /// Ensure storage exists for a component type
    fn ensure_storage<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        if !self.storages.contains_key(&type_id) {
            self.storages.insert(type_id, ComponentStorage::new::<T>());
        }
    }

    /// Add a component to an entity
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        self.ensure_storage::<T>();
        if let Some(storage) = self.storages.get_mut(&TypeId::of::<T>()) {
            storage.insert(entity, component);
        }
    }

    /// Get a component from an entity
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.storages
            .get(&TypeId::of::<T>())
            .and_then(|storage| storage.get::<T>(entity))
    }

    /// Get a mutable component from an entity
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.storages
            .get_mut(&TypeId::of::<T>())
            .and_then(|storage| storage.get_mut::<T>(entity))
    }

    /// Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> bool {
        self.storages
            .get_mut(&TypeId::of::<T>())
            .map(|storage| storage.remove(entity).is_some())
            .unwrap_or(false)
    }

    /// Check if an entity has a component
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        self.storages
            .get(&TypeId::of::<T>())
            .map(|storage| storage.has_component(entity))
            .unwrap_or(false)
    }

    /// Get all entities with a specific component type
    pub fn entities_with_component<T: Component>(&self) -> Vec<Entity> {
        self.storages
            .get(&TypeId::of::<T>())
            .map(|storage| storage.entities().collect())
            .unwrap_or_default()
    }

    /// Remove all components for an entity
    pub fn remove_all_components(&mut self, entity: Entity) {
        for storage in self.storages.values_mut() {
            storage.remove(entity);
        }
    }
}

impl Default for ComponentManager {
    fn default() -> Self {
        Self::new()
    }
}
