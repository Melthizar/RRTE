//! Plugin infrastructure for extending the engine.
//!
//! Provides loading and registration utilities for dynamic plugins.

pub mod plugin;
pub mod loader;
pub mod registry;
pub mod manifest;

pub use plugin::*;
pub use loader::*;
pub use registry::*;
pub use manifest::*;
