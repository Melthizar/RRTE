use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin manifest describing plugin metadata and dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub engine_version: String,
    pub dependencies: Vec<PluginDependency>,
    pub entry_points: HashMap<String, String>,
    pub permissions: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version_requirement: String,
    pub optional: bool,
}

impl PluginManifest {
    /// Load manifest from TOML string
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Save manifest to TOML string
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Check if this plugin is compatible with a given engine version
    pub fn is_compatible_with_engine(&self, engine_version: &str) -> bool {
        // Simple version matching - can be enhanced with semver
        self.engine_version == engine_version || self.engine_version == "*"
    }

    /// Check if a dependency is satisfied by available plugins
    pub fn check_dependency(&self, dep: &PluginDependency, available_plugins: &[&PluginManifest]) -> bool {
        if dep.optional {
            return true;
        }

        available_plugins.iter().any(|plugin| {
            plugin.name == dep.name && self.version_matches(&plugin.version, &dep.version_requirement)
        })
    }

    /// Simple version matching (can be enhanced with proper semver)
    fn version_matches(&self, available: &str, requirement: &str) -> bool {
        requirement == "*" || available == requirement
    }
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            name: "unnamed-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "A RRTE engine plugin".to_string(),
            author: "Unknown".to_string(),
            engine_version: "0.1.0".to_string(),
            dependencies: Vec::new(),
            entry_points: HashMap::new(),
            permissions: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}
