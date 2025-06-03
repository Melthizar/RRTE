use crate::{Time, Events, Input};
use rrte_renderer::{
    Raytracer, RaytracerConfig, Camera as RendererCamera, GpuRenderer, GpuRendererConfig,
};
use rrte_scene::Scene;
use anyhow::Result;
use log::{info, warn, error};
use std::time::Instant;
use std::sync::Arc;
use winit::window::Window;
use wgpu;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererMode {
    Cpu,
    Gpu,
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub renderer_mode: RendererMode,
    pub renderer_config: RaytracerConfig,
    pub gpu_renderer_config: GpuRendererConfig,
    pub target_fps: f32,
    pub enable_vsync: bool,
    pub log_level: log::LevelFilter,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            renderer_mode: RendererMode::Cpu,
            renderer_config: RaytracerConfig::default(),
            gpu_renderer_config: GpuRendererConfig::default(),
            target_fps: 60.0,
            enable_vsync: true,
            log_level: log::LevelFilter::Info,
        }
    }
}

/// Main engine state
#[derive(Debug, PartialEq)]
pub enum EngineState {
    Uninitialized,
    InitializingRenderer,
    Running,
    Paused,
    Stopping,
    Stopped,
}

/// Holds the active renderer instance
pub enum ActiveRenderer {
    Cpu(Raytracer),
    Gpu(GpuRenderer),
    None,
}

impl std::fmt::Debug for ActiveRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActiveRenderer::Cpu(_) => f.debug_tuple("Cpu").finish(),
            ActiveRenderer::Gpu(_) => f.debug_tuple("Gpu").finish(),
            ActiveRenderer::None => f.debug_tuple("None").finish(),
        }
    }
}

/// Core engine struct that manages all subsystems
#[derive(Debug)]
pub struct Engine {
    config: EngineConfig,
    state: EngineState,
    time: Time,
    renderer: ActiveRenderer,
    scene: rrte_scene::Scene,
    camera: RendererCamera,
    events: Events,
    input: Input,
    frame_buffer: Vec<u8>,
}

impl Engine {
    /// Create a new engine instance (renderer is not initialized yet)
    pub fn new(config: EngineConfig) -> Result<Self> {
        // Initialize logging
        if env_logger::try_init_from_env(env_logger::Env::default().default_filter_or(config.log_level.as_str())).is_err() {
            // Logger might be already set, which is fine.
            // Alternatively, log a warning or handle as appropriate if this is unexpected.
        }

        info!("Initializing RRTE Engine (Renderer pending)...");

        let scene = rrte_scene::Scene::new();
        let aspect_ratio = config.renderer_config.width as f32 / config.renderer_config.height as f32;
        let camera = RendererCamera::new_perspective(45.0_f32.to_radians(), aspect_ratio, 0.1, 100.0);
        let events = Events::new();
        let input = Input::new();
        let time = Time::new();

        let buffer_size = (config.renderer_config.width * config.renderer_config.height * 4) as usize;
        let frame_buffer = vec![0u8; buffer_size];

        Ok(Self {
            config,
            state: EngineState::Uninitialized,
            time,
            renderer: ActiveRenderer::None,
            scene,
            camera,
            events,
            input,
            frame_buffer,
        })
    }

    /// Initialize the chosen renderer. This needs to be called after window creation for GPU.
    pub async fn initialize_renderer(&mut self, window: Option<Arc<Window>>) -> Result<()> {
        if !matches!(self.state, EngineState::Uninitialized) {
            warn!("Renderer already initialized or initialization in progress.");
            return Ok(());
        }
        self.state = EngineState::InitializingRenderer;
        info!("Initializing {:?} renderer...", self.config.renderer_mode);

        match self.config.renderer_mode {
            RendererMode::Cpu => {
                let cpu_renderer = Raytracer::new(self.config.renderer_config.clone());
                self.renderer = ActiveRenderer::Cpu(cpu_renderer);
                info!("CPU Renderer initialized.");
            }
            RendererMode::Gpu => {
                let window_arc = window.ok_or_else(|| anyhow::anyhow!("Window handle required for GPU renderer initialization"))?;
                
                let mut gpu_config = self.config.gpu_renderer_config.clone();
                // Ensure GPU config dimensions match the main config if not already set
                // (These might have been set by update_resolution before renderer init)
                if gpu_config.width == 0 { gpu_config.width = self.config.renderer_config.width; }
                if gpu_config.height == 0 { gpu_config.height = self.config.renderer_config.height; }
                if gpu_config.width == 0 || gpu_config.height == 0 {
                    return Err(anyhow::anyhow!("GPU renderer dimensions are zero."));
                }

                // WGPU Instance
                let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::all(),
                    dx12_shader_compiler: Default::default(),
                    flags: wgpu::InstanceFlags::default(),
                    gles_minor_version: wgpu::Gles3MinorVersion::default(),
                });

                // Surface
                // Safety: The window is kept alive by the main application loop.
                let surface = unsafe { instance.create_surface_unsafe(
                    wgpu::SurfaceTargetUnsafe::from_window(&window_arc)?
                )}.map_err(|e| anyhow::anyhow!("Failed to create wgpu surface: {}", e))?;
                let surface_arc = Arc::new(surface);

                // Adapter
                let adapter = instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::HighPerformance,
                        compatible_surface: Some(&surface_arc),
                        force_fallback_adapter: false,
                    })
                    .await
                    .ok_or_else(|| anyhow::anyhow!("Failed to find a suitable GPU adapter."))?;
                info!("Selected GPU: {}", adapter.get_info().name);

                // Device and Queue
                let (device, queue) = adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            required_features: wgpu::Features::empty(), // Add features as needed
                            required_limits: wgpu::Limits::default(),
                            label: Some("RRTE Device"),
                        },
                        None, // Trace path
                    )
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create logical device and queue: {}", e))?;
                
                let device_arc = Arc::new(device);
                let queue_arc = Arc::new(queue);

                // Surface Configuration
                let surface_caps = surface_arc.get_capabilities(&adapter);
                // Shader code in GpuRenderer uses Bgra8UnormSrgb or similar, ensure it matches.
                // GpuRendererConfig also has a format, use that.
                let surface_format = gpu_config.format; 
                if !surface_caps.formats.contains(&surface_format) {
                    warn!("Preferred surface format {:?} not supported. Falling back to first supported format: {:?}", 
                           surface_format, surface_caps.formats[0]);
                    gpu_config.format = surface_caps.formats[0];
                }

                // Use actual window size instead of config size for surface configuration
                let window_size = window_arc.inner_size();
                info!("Configuring GPU surface with actual window size: {}x{}", window_size.width, window_size.height);

                let surface_config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: gpu_config.format,
                    width: window_size.width,  // Use actual window width
                    height: window_size.height, // Use actual window height
                    present_mode: gpu_config.present_mode, 
                    alpha_mode: surface_caps.alpha_modes[0], // Use first supported alpha mode
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2, // Default value
                };
                surface_arc.configure(&device_arc, &surface_config);
                
                // Update GPU config to match actual window size
                gpu_config.width = window_size.width;
                gpu_config.height = window_size.height;
                self.config.gpu_renderer_config = gpu_config.clone(); // Store potentially updated format

                let gpu_renderer_instance = GpuRenderer::new(
                    &gpu_config, 
                    device_arc, 
                    queue_arc, 
                    surface_config, 
                    surface_arc, 
                    Some(window_arc.clone())
                ).await?;
                
                self.renderer = ActiveRenderer::Gpu(gpu_renderer_instance);
                
                // Update camera aspect ratio to match actual window size
                if let rrte_renderer::camera::ProjectionType::Perspective { aspect_ratio, .. } = &mut self.camera.projection {
                    *aspect_ratio = window_size.width as f32 / window_size.height as f32;
                }
                
                info!("GPU Renderer initialized.");
            }
        }
        self.state = EngineState::Running;
        self.time.start();
        info!("Engine systems and renderer initialized successfully.");
        Ok(())
    }

    /// Main engine run loop (conceptual, actual loop is in main.rs)
    /// This method is kept for potential non-windowed/headless operation or future refactor.
    pub fn run_headless_loop(&mut self) -> Result<()> {
        if !matches!(self.config.renderer_mode, RendererMode::Cpu) {
            error!("Headless loop is only supported for CPU renderer.");
            return Err(anyhow::anyhow!("Headless loop not supported for GPU renderer"));
        }
        info!("Starting engine headless loop (CPU only)...");
        
        let target_frame_duration = std::time::Duration::from_secs_f32(1.0 / self.config.target_fps);

        while self.is_running() {
            let frame_start = Instant::now();
            
            self.time.update();
            self.events.poll();
            self.input.update();
            // self.scene.update(self.time.delta_time()); // Commented out: Scene::update not yet on rrte_scene::Scene
            
            if let Err(e) = self.render_frame() {
                error!("Render error in headless loop: {}", e);
            }
            
            let frame_duration = frame_start.elapsed();
            if frame_duration < target_frame_duration {
                std::thread::sleep(target_frame_duration - frame_duration);
            }
        }
        info!("Engine headless loop ended");
        Ok(())
    }

    /// Render a frame.
    /// For CPU, it renders to an internal buffer.
    /// For GPU, it renders directly to the screen/surface.
    pub fn render_frame(&mut self) -> Result<()> {
        match &mut self.renderer {
            ActiveRenderer::Cpu(raytracer) => {
                // Convert Vec<Arc<Sphere>> to Vec<Arc<dyn SceneObject>> for the CPU raytracer
                let scene_objects: Vec<Arc<dyn rrte_renderer::primitives::SceneObject>> = 
                    self.scene.objects().iter().map(|s| s.clone() as Arc<dyn rrte_renderer::primitives::SceneObject>).collect();
                
                // Convert Vec<Arc<PointLight>> to Vec<Arc<dyn Light>> for the CPU raytracer
                let scene_lights: Vec<Arc<dyn rrte_renderer::light::Light>> = 
                    self.scene.lights().iter().map(|l| l.clone() as Arc<dyn rrte_renderer::light::Light>).collect();
                
                // TODO: The Scene struct should also store directional lights if needed by CPU raytracer.
                // For now, passing an empty vec for directional lights.
                self.frame_buffer = raytracer.render(&scene_objects, &scene_lights, &Vec::new(), &self.camera);
            }
            ActiveRenderer::Gpu(gpu_renderer) => {
                let output_surface_texture = gpu_renderer.get_current_texture()?;
                
                // GpuRenderer::render now takes spheres and lights directly to avoid cyclic dependency
                gpu_renderer.render(
                    &output_surface_texture.texture, // This is the swap chain texture
                    self.scene.objects(), // Pass spheres directly via accessor
                    self.scene.lights(), // Pass lights directly via accessor
                    &self.camera
                )?;
                output_surface_texture.present();
            }
            ActiveRenderer::None => {
                return Err(anyhow::anyhow!("Renderer not initialized before render_frame call."));
            }
        }
        Ok(())
    }

    /// Updates the engine and renderer resolution.
    pub fn update_resolution(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            warn!("Attempted to update resolution to zero size, skipping.");
            return;
        }
        info!("Updating resolution to {}x{}", width, height);
        
        match &mut self.renderer {
            ActiveRenderer::Cpu(raytracer) => {
                self.config.renderer_config.width = width;
                self.config.renderer_config.height = height;
                raytracer.update_config(self.config.renderer_config.clone());
                let buffer_size = (width * height * 4) as usize;
                self.frame_buffer.resize(buffer_size, 0u8);
            }
            ActiveRenderer::Gpu(gpu_renderer) => {
                if let Err(e) = gpu_renderer.resize(width, height) {
                    error!("GpuRenderer resize error: {}", e);
                }
                self.config.gpu_renderer_config.width = width;
                self.config.gpu_renderer_config.height = height;
            }
            ActiveRenderer::None => {
                warn!("update_resolution called before renderer initialization. Storing in CPU config for now.");
                self.config.renderer_config.width = width;
                self.config.renderer_config.height = height;
                self.config.gpu_renderer_config.width = width;
                self.config.gpu_renderer_config.height = height;
            }
        }
        
        if let rrte_renderer::camera::ProjectionType::Perspective { aspect_ratio, .. } = &mut self.camera.projection {
            *aspect_ratio = width as f32 / height as f32;
        }
    }

    /// Get the current frame buffer (only Some for CPU renderer)
    pub fn get_frame_buffer(&self) -> Option<&[u8]> {
        match self.config.renderer_mode {
            RendererMode::Cpu => Some(&self.frame_buffer),
            RendererMode::Gpu => None,
        }
    }
    
    /// Initialize the engine systems (excluding renderer, which is now separate)
    pub fn initialize_core_systems(&mut self) -> Result<()> {
        info!("Core engine systems initialization (excluding renderer)...");
        if self.state != EngineState::Uninitialized {
            warn!("Core systems already initialized or initialization in progress.");
            return Ok(());
        }
        info!("Core engine systems ready (pending renderer initialization).");
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, EngineState::Running)
    }

    pub fn pause(&mut self) {
        if matches!(self.state, EngineState::Running) {
            self.state = EngineState::Paused;
            info!("Engine paused");
        }
    }

    pub fn resume(&mut self) {
        if matches!(self.state, EngineState::Paused) {
            self.state = EngineState::Running;
            info!("Engine resumed");
        }
    }

    pub fn stop(&mut self) {
        if self.state != EngineState::Stopping && self.state != EngineState::Stopped {
            self.state = EngineState::Stopping;
            info!("Engine stopping...");
        }
    }

    pub fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down engine...");
        self.state = EngineState::Stopped;
        info!("Engine shutdown complete");
        Ok(())
    }
    
    pub fn config(&self) -> &EngineConfig { &self.config }
    pub fn config_mut(&mut self) -> &mut EngineConfig { &mut self.config }
    pub fn state(&self) -> &EngineState { &self.state }
    pub fn scene_mut(&mut self) -> &mut rrte_scene::Scene { &mut self.scene }
    pub fn scene(&self) -> &rrte_scene::Scene { &self.scene }
    pub fn camera_mut(&mut self) -> &mut RendererCamera { &mut self.camera }
    pub fn camera(&self) -> &RendererCamera { &self.camera }
    pub fn time(&self) -> &Time { &self.time }
    pub fn time_mut(&mut self) -> &mut Time { &mut self.time }
    pub fn input(&self) -> &Input { &self.input }
    pub fn input_mut(&mut self) -> &mut Input { &mut self.input }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if !matches!(self.state, EngineState::Stopped) {
            let _ = self.shutdown();
        }
    }
}
