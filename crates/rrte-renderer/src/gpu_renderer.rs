use wgpu::{Device, Queue, Surface, SurfaceConfiguration, TextureFormat};
use winit::window::Window;
use anyhow::Result;
use std::sync::Arc;

/// GPU renderer configuration
#[derive(Debug, Clone)]
pub struct GpuRendererConfig {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub present_mode: wgpu::PresentMode,
    pub samples: u32,
}

impl Default for GpuRendererConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            format: TextureFormat::Bgra8UnormSrgb,
            present_mode: wgpu::PresentMode::Fifo,
            samples: 1,
        }
    }
}

/// GPU-based renderer using wgpu
pub struct GpuRenderer {
    config: GpuRendererConfig,
    device: Option<Device>,
    queue: Option<Queue>,
    surface: Option<Surface<'static>>,
    surface_config: Option<SurfaceConfiguration>,
}

impl GpuRenderer {
    /// Create a new GPU renderer
    pub fn new(config: GpuRendererConfig) -> Self {
        Self {
            config,
            device: None,
            queue: None,
            surface: None,
            surface_config: None,
        }
    }

    /// Initialize the GPU renderer with a window
    pub async fn initialize(&mut self, window: Arc<Window>) -> Result<()> {
        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance.create_surface(window.clone())?;
        
        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to request adapter"))?;

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("RRTE Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        // Get surface capabilities
        let surface_caps = surface.get_capabilities(&adapter);
        
        // Choose the first supported format, or fall back to the first one
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| **f == self.config.format)
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        // Create surface configuration
        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: self.config.width,
            height: self.config.height,
            present_mode: self.config.present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        // Store the initialized components
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.surface_config = Some(surface_config);

        Ok(())
    }

    /// Render a frame
    pub fn render(&mut self) -> Result<()> {
        let device = self.device.as_ref().ok_or_else(|| anyhow::anyhow!("Device not initialized"))?;
        let queue = self.queue.as_ref().ok_or_else(|| anyhow::anyhow!("Queue not initialized"))?;
        let surface = self.surface.as_ref().ok_or_else(|| anyhow::anyhow!("Surface not initialized"))?;

        // Get the next frame
        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Begin render pass
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // TODO: Add actual rendering commands here
        }

        // Submit commands
        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Resize the renderer
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.config.width = width;
        self.config.height = height;

        if let (Some(surface), Some(device), Some(surface_config)) = 
            (&self.surface, &self.device, &mut self.surface_config) {
            surface_config.width = width;
            surface_config.height = height;
            surface.configure(device, surface_config);
        }

        Ok(())
    }

    /// Get renderer configuration
    pub fn config(&self) -> &GpuRendererConfig {
        &self.config
    }

    /// Check if the renderer is initialized
    pub fn is_initialized(&self) -> bool {
        self.device.is_some() && self.queue.is_some() && self.surface.is_some()
    }

    /// Get the device reference
    pub fn device(&self) -> Option<&Device> {
        self.device.as_ref()
    }

    /// Get the queue reference
    pub fn queue(&self) -> Option<&Queue> {
        self.queue.as_ref()
    }

    /// Get the surface configuration
    pub fn surface_config(&self) -> Option<&SurfaceConfiguration> {
        self.surface_config.as_ref()
    }
}

impl std::fmt::Debug for GpuRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuRenderer")
            .field("config", &self.config)
            .field("initialized", &self.is_initialized())
            .finish()
    }
}
