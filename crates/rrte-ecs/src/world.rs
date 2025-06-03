use crate::{Entity, Component, ComponentStorage};
use std::collections::HashMap;
use std::any::TypeId;

/// A world contains entities and components
pub struct World {
    entities: Vec<Entity>,
    next_entity_id: u64,
    component_managers: HashMap<TypeId, ComponentStorage>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            next_entity_id: 0,
            component_managers: HashMap::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity::new(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(entity);
        entity
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        self.entities.retain(|&e| e != entity);
        // Remove components for this entity from all managers
        for manager in self.component_managers.values_mut() {
            // This would need to be implemented properly
            // manager.remove_component(entity);
        }
    }

    pub fn add_component<T: Component + 'static>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        if !self.component_managers.contains_key(&type_id) {
            self.component_managers.insert(type_id, ComponentStorage::new::<T>());
        }
        
        if let Some(storage) = self.component_managers.get_mut(&type_id) {
            storage.insert(entity, component);
        }
    }

    pub fn get_component<T: Component + 'static>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.component_managers.get(&type_id)?.get::<T>(entity)
    }

    pub fn get_component_mut<T: Component + 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.component_managers.get_mut(&type_id)?.get_mut::<T>(entity)
    }

    pub fn remove_component<T: Component + 'static>(&mut self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.component_managers.get_mut(&type_id) {
            storage.remove(entity).is_some()
        } else {
            false
        }
    }

    pub fn get_entities_with_component<T: Component + 'static>(&self) -> Vec<Entity> {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.component_managers.get(&type_id) {
            storage.entities().collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_entities(&self) -> &[Entity] {
        &self.entities
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
