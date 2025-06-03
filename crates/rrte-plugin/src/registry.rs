//! Plugin registry (stub)

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct PluginRegistry {
    plugins: HashMap<String, ()>,
}

impl PluginRegistry {
    pub fn new() -> Self { Self { plugins: HashMap::new() } }

    /// Return the number of registered plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}
