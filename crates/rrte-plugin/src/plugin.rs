use crate::PluginManifest;
use anyhow::Result;
use std::any::Any;

/// Plugin lifecycle hooks
pub trait Plugin: Send + Sync + 'static {
    /// Get plugin manifest
    fn manifest(&self) -> &PluginManifest;

    /// Initialize the plugin
    fn initialize(&mut self, context: &mut PluginContext) -> Result<()>;

    /// Update the plugin (called every frame)
    fn update(&mut self, context: &mut PluginContext, delta_time: f32) -> Result<()>;

    /// Shutdown the plugin
    fn shutdown(&mut self, context: &mut PluginContext) -> Result<()>;

    /// Handle plugin events
    fn handle_event(&mut self, context: &mut PluginContext, event: &PluginEvent) -> Result<()> {
        let _ = (context, event);
        Ok(())
    }

    /// Get plugin as Any for downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Context provided to plugins for engine interaction
pub struct PluginContext {
    pub engine_version: String,
    pub world: Option<*mut rrte_ecs::World>,
    pub resources: std::collections::HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl PluginContext {
    pub fn new(engine_version: String) -> Self {
        Self {
            engine_version,
            world: None,
            resources: std::collections::HashMap::new(),
        }
    }

    /// Add a resource to the context
    pub fn add_resource<T: Any + Send + Sync>(&mut self, name: String, resource: T) {
        self.resources.insert(name, Box::new(resource));
    }

    /// Get a resource from the context
    pub fn get_resource<T: Any + Send + Sync>(&self, name: &str) -> Option<&T> {
        self.resources
            .get(name)
            .and_then(|res| res.downcast_ref::<T>())
    }

    /// Get a mutable resource from the context
    pub fn get_resource_mut<T: Any + Send + Sync>(&mut self, name: &str) -> Option<&mut T> {
        self.resources
            .get_mut(name)
            .and_then(|res| res.downcast_mut::<T>())
    }

    /// Remove a resource from the context
    pub fn remove_resource(&mut self, name: &str) -> Option<Box<dyn Any + Send + Sync>> {
        self.resources.remove(name)
    }
}

/// Events that can be sent to plugins
#[derive(Debug, Clone)]
pub enum PluginEvent {
    /// Engine is starting up
    EngineStartup,
    /// Engine is shutting down
    EngineShutdown,
    /// A frame has started
    FrameStart,
    /// A frame has ended
    FrameEnd,
    /// Scene is being loaded
    SceneLoad { scene_path: String },
    /// Scene is being unloaded
    SceneUnload { scene_path: String },
    /// Custom event with arbitrary data
    Custom {
        event_type: String,
        data: serde_json::Value,
    },
}

/// Plugin state
#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Unloaded,
    Loaded,
    Initialized,
    Running,
    Error(String),
}

/// Base plugin implementation for easier plugin development
pub struct BasePlugin {
    manifest: PluginManifest,
    state: PluginState,
}

impl BasePlugin {
    pub fn new(manifest: PluginManifest) -> Self {
        Self {
            manifest,
            state: PluginState::Unloaded,
        }
    }

    pub fn state(&self) -> &PluginState {
        &self.state
    }

    pub fn set_state(&mut self, state: PluginState) {
        self.state = state;
    }
}

impl Plugin for BasePlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn initialize(&mut self, _context: &mut PluginContext) -> Result<()> {
        self.state = PluginState::Initialized;
        log::info!("Plugin '{}' initialized", self.manifest.name);
        Ok(())
    }

    fn update(&mut self, _context: &mut PluginContext, _delta_time: f32) -> Result<()> {
        if self.state == PluginState::Initialized {
            self.state = PluginState::Running;
        }
        Ok(())
    }

    fn shutdown(&mut self, _context: &mut PluginContext) -> Result<()> {
        self.state = PluginState::Unloaded;
        log::info!("Plugin '{}' shutdown", self.manifest.name);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
