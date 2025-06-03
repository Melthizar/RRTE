use rrte_math::{Transform, Vec3, Color};
use rrte_renderer::{SceneObject, Material, Light};
use rrte_ecs::{Entity, World, Component};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Scene configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneConfig {
    pub name: String,
    pub ambient_light: Color,
    pub fog_color: Color,
    pub fog_density: f32,
    pub gravity: Vec3,
}

impl Default for SceneConfig {
    fn default() -> Self {
        Self {
            name: "Default Scene".to_string(),
            ambient_light: Color::new(0.1, 0.1, 0.1, 1.0),
            fog_color: Color::new(0.5, 0.5, 0.5, 1.0),
            fog_density: 0.0,
            gravity: Vec3::new(0.0, -9.81, 0.0),
        }
    }
}

/// Scene component for objects that exist in 3D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneComponent {
    pub transform: Transform,
    pub visible: bool,
    pub layer: u32,
}

impl Default for SceneComponent {
    fn default() -> Self {
        Self {
            transform: Transform::identity(),
            visible: true,
            layer: 0,
        }
    }
}

/// Scene management system
pub struct Scene {
    config: SceneConfig,
    world: World,
    objects: Vec<Arc<dyn SceneObject>>,
    materials: Vec<Arc<dyn Material>>,
    lights: Vec<Arc<dyn Light>>,
    dirty: bool,
}

impl Scene {    /// Create a new empty scene
    pub fn new() -> Self {
        Self {
            config: SceneConfig::default(),
            world: World::new(),
            objects: Vec::new(),
            materials: Vec::new(),
            lights: Vec::new(),
            dirty: true,
        }
    }

    /// Create a new scene with configuration
    pub fn with_config(config: SceneConfig) -> Self {
        Self {
            config,
            world: World::new(),
            objects: Vec::new(),
            materials: Vec::new(),
            lights: Vec::new(),
            dirty: true,
        }
    }

    /// Update the scene
    pub fn update(&mut self, _delta_time: f32) {
        // Update entity systems
        // self.entity_manager_mut().update(delta_time); // FIXME: World has no update method
        
        // Mark as clean after update
        self.dirty = false;
    }

    /// Add an object to the scene
    pub fn add_object(&mut self, object: Arc<dyn SceneObject>) {
        self.objects.push(object);
        self.dirty = true;
    }

    /// Remove an object from the scene by index
    pub fn remove_object(&mut self, index: usize) -> Option<Arc<dyn SceneObject>> {
        if index < self.objects.len() {
            self.dirty = true;
            Some(self.objects.remove(index))
        } else {
            None
        }
    }

    /// Add a material to the scene
    pub fn add_material(&mut self, material: Arc<dyn Material>) {
        self.materials.push(material);
        self.dirty = true;
    }

    /// Add a light to the scene
    pub fn add_light(&mut self, light: Arc<dyn Light>) {
        self.lights.push(light);
        self.dirty = true;
    }

    /// Remove a light from the scene by index
    pub fn remove_light(&mut self, index: usize) -> Option<Arc<dyn Light>> {
        if index < self.lights.len() {
            self.dirty = true;
            Some(self.lights.remove(index))
        } else {
            None
        }
    }

    /// Get all objects in the scene
    pub fn get_objects(&self) -> &[Arc<dyn SceneObject>] {
        &self.objects
    }

    /// Get all materials in the scene
    pub fn get_materials(&self) -> &[Arc<dyn Material>] {
        &self.materials
    }

    /// Get all lights in the scene
    pub fn get_lights(&self) -> &[Arc<dyn Light>] {
        &self.lights
    }

    /// Get mutable reference to objects
    pub fn get_objects_mut(&mut self) -> &mut Vec<Arc<dyn SceneObject>> {
        self.dirty = true;
        &mut self.objects
    }

    /// Get mutable reference to lights
    pub fn get_lights_mut(&mut self) -> &mut Vec<Arc<dyn Light>> {
        self.dirty = true;
        &mut self.lights
    }

    /// Clear all objects from the scene
    pub fn clear_objects(&mut self) {
        self.objects.clear();
        self.dirty = true;
    }

    /// Clear all lights from the scene
    pub fn clear_lights(&mut self) {
        self.lights.clear();
        self.dirty = true;
    }

    /// Clear all materials from the scene
    pub fn clear_materials(&mut self) {
        self.materials.clear();
        self.dirty = true;
    }

    /// Clear the entire scene
    pub fn clear(&mut self) {
        self.clear_objects();
        self.clear_lights();
        self.clear_materials();
        self.world = World::new();
        self.dirty = true;
    }

    /// Create a new entity in the scene
    pub fn create_entity(&mut self) -> Entity {
        let entity = self.entity_manager_mut().create_entity();
        
        // Add default scene component
        self.entity_manager_mut().add_component(entity, SceneComponent::default());
        
        self.dirty = true;
        entity
    }

    /// Remove an entity from the scene
    pub fn remove_entity(&mut self, entity: Entity) {
        self.entity_manager_mut().destroy_entity(entity);
        self.dirty = true;
    }

    /// Add a component to an entity
    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) {
        self.entity_manager_mut().add_component(entity, component);
        self.dirty = true;
    }

    /// Get a component from an entity
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        self.entity_manager().get_component(entity)
    }

    /// Get a mutable component from an entity
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        self.entity_manager_mut().get_component_mut(entity)
    }

    /// Check if an entity has a component
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        self.entity_manager().get_component::<T>(entity).is_some()
    }

    /// Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> bool {
        let removed = self.entity_manager_mut().remove_component::<T>(entity);
        if removed {
            self.dirty = true;
        }
        removed
    }

    /// Get all entities with a specific component
    pub fn get_entities_with_component<T: Component>(&self) -> Vec<Entity> {
        self.entity_manager().get_entities_with_component::<T>()
    }

    /// Get scene configuration
    pub fn config(&self) -> &SceneConfig {
        &self.config
    }

    /// Get mutable scene configuration
    pub fn config_mut(&mut self) -> &mut SceneConfig {
        self.dirty = true;
        &mut self.config
    }

    /// Set scene configuration
    pub fn set_config(&mut self, config: SceneConfig) {
        self.config = config;
        self.dirty = true;
    }

    /// Check if the scene has been modified
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark the scene as clean
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Mark the scene as dirty
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Get the number of objects in the scene
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// Get the number of lights in the scene
    pub fn light_count(&self) -> usize {
        self.lights.len()
    }

    /// Get the number of materials in the scene
    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    /// Get the total number of entities in the scene
    pub fn entity_count(&self) -> usize {
        self.entity_manager().get_entities().len()
    }

    /// Get a reference to the entity manager (now World)
    pub fn entity_manager(&self) -> &World {
        &self.world
    }

    /// Get a mutable reference to the entity manager (now World)
    pub fn entity_manager_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("config", &self.config)
            .field("objects", &self.objects.len())
            .field("materials", &self.materials.len())
            .field("lights", &self.lights.len())
            .field("entities", &self.entity_manager().get_entities().len())
            .field("dirty", &self.dirty)
            .finish()
    }
}
