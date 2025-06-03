use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique entity identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u64);

impl Entity {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Entity manager for creating and tracking entities
pub struct EntityManager {
    next_id: AtomicU64,
    alive_entities: HashMap<u64, Entity>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            alive_entities: HashMap::new(),
        }
    }

    /// Create a new entity
    pub fn create_entity(&mut self) -> Entity {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let entity = Entity(id);
        self.alive_entities.insert(id, entity);
        entity
    }

    /// Destroy an entity
    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        self.alive_entities.remove(&entity.0).is_some()
    }

    /// Check if an entity is alive
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.alive_entities.contains_key(&entity.0)
    }

    /// Get all alive entities
    pub fn alive_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.alive_entities.values().copied()
    }

    /// Get the count of alive entities
    pub fn entity_count(&self) -> usize {
        self.alive_entities.len()
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}
