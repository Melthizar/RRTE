use crate::{Entity, ComponentStorage};

/// A trait for systems that operate on entities and components
pub trait System {
    fn run(&mut self, entities: &[Entity], components: &mut ComponentStorage);
}



/// A basic system scheduler
pub struct SystemScheduler {
    systems: Vec<Box<dyn System>>,
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }    pub fn run_systems(&mut self, entities: &[Entity], components: &mut ComponentStorage) {
        for system in &mut self.systems {
            system.run(entities, components);
        }
    }
}

impl Default for SystemScheduler {
    fn default() -> Self {
        Self::new()
    }
}
