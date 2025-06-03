//! Plugin registry (stub)

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct PluginRegistry {
    plugins: HashMap<String, ()>,
}

impl PluginRegistry {
    pub fn new() -> Self { Self { plugins: HashMap::new() } }
}
