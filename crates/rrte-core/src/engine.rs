use crate::{Time, Camera, Scene, Events, Input};
use rrte_renderer::{Raytracer, RaytracerConfig};
use anyhow::Result;
use log::{info, warn, error};
use std::time::Instant;

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub renderer_config: RaytracerConfig,
    pub target_fps: f32,
    pub enable_vsync: bool,
    pub log_level: log::LevelFilter,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            renderer_config: RaytracerConfig::default(),
            target_fps: 60.0,
            enable_vsync: true,
            log_level: log::LevelFilter::Info,
        }
    }
}

/// Main engine state
#[derive(Debug)]
pub enum EngineState {
    Initializing,
    Running,
    Paused,
    Stopping,
    Stopped,
}

/// Core engine struct that manages all subsystems
pub struct Engine {
    config: EngineConfig,
    state: EngineState,
    time: Time,
    renderer: Raytracer,
    scene: Scene,
    camera: Camera,
    events: Events,
    input: Input,
    frame_buffer: Vec<u8>,
}

impl Engine {
    /// Create a new engine instance
    pub fn new(config: EngineConfig) -> Result<Self> {
        // Initialize logging
        env_logger::Builder::from_default_env()
            .filter_level(config.log_level)
            .init();

        info!("Initializing RRTE Engine...");

        let renderer = Raytracer::new(config.renderer_config.clone());
        let scene = Scene::new();
        let camera = Camera::default();
        let events = Events::new();
        let input = Input::new();
        let time = Time::new();

        let buffer_size = (config.renderer_config.width * config.renderer_config.height * 4) as usize;
        let frame_buffer = vec![0u8; buffer_size];

        Ok(Self {
            config,
            state: EngineState::Initializing,
            time,
            renderer,
            scene,
            camera,
            events,
            input,
            frame_buffer,
        })
    }

    /// Initialize the engine systems
    pub fn initialize(&mut self) -> Result<()> {
        info!("Engine initialization starting...");
        
        self.state = EngineState::Running;
        self.time.start();
        
        info!("Engine initialized successfully");
        Ok(())
    }

    /// Main engine run loop
    pub fn run(&mut self) -> Result<()> {
        info!("Starting engine main loop...");
        
        let mut last_frame_time = Instant::now();
        let target_frame_duration = std::time::Duration::from_secs_f32(1.0 / self.config.target_fps);

        while self.is_running() {
            let frame_start = Instant::now();
            
            // Update timing
            self.time.update();
            
            // Process events and input
            self.events.poll();
            self.input.update();
            
            // Update scene
            self.scene.update(self.time.delta_time());
            
            // Render frame
            if let Err(e) = self.render() {
                error!("Render error: {}", e);
            }
            
            // Handle frame rate limiting
            let frame_duration = frame_start.elapsed();
            if frame_duration < target_frame_duration {
                std::thread::sleep(target_frame_duration - frame_duration);
            }
            
            last_frame_time = frame_start;
        }

        info!("Engine main loop ended");
        Ok(())
    }

    /// Render a frame
    fn render(&mut self) -> Result<()> {
        // Get scene objects and render
        let objects = self.scene.get_objects();
        let lights = self.scene.get_lights();
        let materials = self.scene.get_materials();
        
        self.frame_buffer = self.renderer.render(&objects, &lights, &materials, &self.camera);
        
        Ok(())
    }

    /// Check if the engine should continue running
    pub fn is_running(&self) -> bool {
        matches!(self.state, EngineState::Running)
    }

    /// Pause the engine
    pub fn pause(&mut self) {
        if matches!(self.state, EngineState::Running) {
            self.state = EngineState::Paused;
            info!("Engine paused");
        }
    }

    /// Resume the engine
    pub fn resume(&mut self) {
        if matches!(self.state, EngineState::Paused) {
            self.state = EngineState::Running;
            info!("Engine resumed");
        }
    }

    /// Stop the engine
    pub fn stop(&mut self) {
        self.state = EngineState::Stopping;
        info!("Engine stopping...");
    }

    /// Shutdown the engine
    pub fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down engine...");
        
        self.state = EngineState::Stopped;
        
        info!("Engine shutdown complete");
        Ok(())
    }

    /// Get the current frame buffer
    pub fn get_frame_buffer(&self) -> &[u8] {
        &self.frame_buffer
    }

    /// Get engine configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Get engine state
    pub fn state(&self) -> &EngineState {
        &self.state
    }

    /// Get mutable reference to scene
    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    /// Get reference to scene
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// Get mutable reference to camera
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Get reference to camera
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Get time information
    pub fn time(&self) -> &Time {
        &self.time
    }

    /// Get input state
    pub fn input(&self) -> &Input {
        &self.input
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if !matches!(self.state, EngineState::Stopped) {
            let _ = self.shutdown();
        }
    }
}
