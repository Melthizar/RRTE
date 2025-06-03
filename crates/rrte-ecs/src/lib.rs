//! ECS (Entity Component System) utilities.
//!
//! This crate provides a minimal ECS implementation used by the engine.

pub mod entity;
pub mod component;
pub mod system;
pub mod world;
pub mod query;

pub use entity::*;
pub use component::*;
pub use system::*;
pub use world::*;
pub use query::*;
