use wgpu::{Device, Queue, Surface, SurfaceConfiguration, TextureFormat};
use winit::window::Window;
use anyhow::Result;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use glam::{Vec3, Mat4};
// use crate::RendererConfig; // Commented out to investigate usage
use crate::camera::Camera as RendererCamera; // Added import for RendererCamera
use crate::material::Material; // Added for material handling
use crate::primitives::Sphere; // Added for sphere handling
use crate::light::PointLight; // Added for light handling
use std::collections::HashMap; // Added for material map
use log::{info, warn, error};

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
            format: TextureFormat::Rgba8UnormSrgb,
            present_mode: wgpu::PresentMode::Fifo,
            samples: 1,
        }
    }
}

// NEW GPU DATA STRUCTURES
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraGpu {
    pub position: [f32; 4], // Using [f32; 4] for alignment (vec3 needs padding in std140/std430)
    pub view_projection: [[f32; 4]; 4], // Mat4
    pub inv_projection: [[f32; 4]; 4], // Mat4 for reconstructing view direction
    pub inv_view: [[f32; 4]; 4], // Mat4 for reconstructing view direction
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SphereGpu {
    pub center: [f32; 4], // vec3 + padding for radius or material_id alignment
    pub radius: f32,
    pub material_index: u32,
    _padding: [u32; 2], // Ensure alignment to 16 bytes if needed, or for future fields
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialGpu {
    pub color: [f32; 4], // rgba
    pub material_type: u32, // 0: Lambertian, 1: Metal, etc.
    pub smoothness: f32, // For metal, roughness etc.
    _padding: [u32; 2], // Ensure alignment
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightGpu {
    pub position: [f32; 4], // xyz + padding or intensity
    pub color: [f32; 4], // rgba
    pub intensity: f32,
    pub range: f32, // Maximum distance the light affects
    _padding: [u32; 2], // Ensure alignment to 16 bytes
}


// END NEW GPU DATA STRUCTURES

/// GPU-based renderer using wgpu
pub struct GpuRenderer {
    config: GpuRendererConfig,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: Arc<wgpu::Surface<'static>>,
    
    // Compute pass resources
    camera_buffer: wgpu::Buffer,
    sphere_buffer: wgpu::Buffer,
    material_buffer: wgpu::Buffer,
    light_buffer: wgpu::Buffer, // Added for point lights
    output_texture: wgpu::Texture,          // Stores the result of the compute shader (Rgba8Unorm)
    output_texture_view: wgpu::TextureView,
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group_layout: wgpu::BindGroupLayout, // Renamed for clarity
    compute_bind_group: wgpu::BindGroup,           // Renamed for clarity

    // Blit pass resources (for copying output_texture to swap chain)
    sampler: wgpu::Sampler,
    blit_bind_group_layout: wgpu::BindGroupLayout,
    blit_bind_group: wgpu::BindGroup,
    blit_pipeline: wgpu::RenderPipeline,
}

impl GpuRenderer {
    /// Create a new GPU renderer
    pub async fn new(
        config: &GpuRendererConfig,
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_config: wgpu::SurfaceConfiguration,
        surface: Arc<wgpu::Surface<'static>>,
        _window: Option<Arc<Window>> // May be needed for aspect ratio, etc.
    ) -> anyhow::Result<Self> {
        info!("Initializing GpuRenderer");

        // --- Compute Pass Resources ---

        // Create Camera Buffer
        let default_camera_gpu = CameraGpu {
            position: [0.0, 0.0, 0.0, 1.0],
            view_projection: Mat4::IDENTITY.to_cols_array_2d(),
            inv_projection: Mat4::IDENTITY.to_cols_array_2d(),
            inv_view: Mat4::IDENTITY.to_cols_array_2d(),
        };
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::bytes_of(&default_camera_gpu),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let initial_spheres_gpu: Vec<SphereGpu> = vec![SphereGpu {
            center: [0.0, 0.0, 0.0, 0.0], radius: 1.0, material_index: 0, _padding: [0,0]
        }; 1];
        let sphere_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Buffer (Initial)"),
            contents: bytemuck::cast_slice(&initial_spheres_gpu),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        
        let initial_materials_gpu: Vec<MaterialGpu> = vec![MaterialGpu {
            color: [0.8, 0.8, 0.8, 1.0], material_type: 0, smoothness: 0.5, _padding: [0,0]
        }; 1];
        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Buffer (Initial)"),
            contents: bytemuck::cast_slice(&initial_materials_gpu),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let initial_lights_gpu: Vec<PointLightGpu> = vec![PointLightGpu {
            position: [0.0, 10.0, 0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0], intensity: 100.0, range: 50.0, _padding: [0,0]
        }; 1];
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer (Initial)"),
            contents: bytemuck::cast_slice(&initial_lights_gpu),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let output_texture_descriptor = wgpu::TextureDescriptor {
            label: Some("Output Texture (Rgba8Unorm)"),
            size: wgpu::Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC, // COPY_SRC might not be needed if only blitting
            view_formats: &[],
        };
        let output_texture = device.create_texture(&output_texture_descriptor);
        let output_texture_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let compute_shader_source = include_str!("shaders/raytrace.wgsl");
        let compute_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Raytrace Shader Module"),
            source: wgpu::ShaderSource::Wgsl(compute_shader_source.into()),
        });

        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Raytrace Compute Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { // CameraGpu
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<CameraGpu>() as u64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { // Spheres
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<SphereGpu>() as u64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { // Materials
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<MaterialGpu>() as u64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { // Lights
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<PointLightGpu>() as u64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { // Output Texture (Storage)
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Raytrace Compute Pipeline Layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Raytrace Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader_module,
            entry_point: "main",
        });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Raytrace Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sphere_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&output_texture_view),
                },
            ],
        });

        // --- Blit Pass Resources ---
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blit Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear, // or Nearest
            min_filter: wgpu::FilterMode::Linear, // or Nearest
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let blit_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blit Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry { // Input Texture (output_texture from compute pass)
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { // Sampler
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let blit_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blit Bind Group"),
            layout: &blit_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&output_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        
        let blit_shader_source = include_str!("shaders/blit.wgsl");
        let blit_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blit Shader Module"),
            source: wgpu::ShaderSource::Wgsl(blit_shader_source.into()),
        });

        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Pipeline Layout"),
            bind_group_layouts: &[&blit_bind_group_layout],
            push_constant_ranges: &[],
        });

        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit Render Pipeline"),
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader_module,
                entry_point: "vs_main",
                buffers: &[], // No vertex buffers, vertices generated in shader
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format, // Target the swap chain format
                    blend: Some(wgpu::BlendState::REPLACE), // Opaque
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for a fullscreen triangle/quad
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Ok(Self {
            config: config.clone(),
            device,
            queue,
            surface_config,
            surface,
            camera_buffer,
            sphere_buffer,
            material_buffer,
            light_buffer,
            output_texture,
            output_texture_view,
            compute_pipeline,
            compute_bind_group_layout, // Renamed
            compute_bind_group,      // Renamed
            sampler,
            blit_bind_group_layout,
            blit_bind_group,
            blit_pipeline,
        })
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
        self.device = Arc::new(device);
        self.queue = Arc::new(queue);
        self.surface = Arc::new(surface);
        self.surface_config = surface_config;

        Ok(())
    }

    /// Render a frame
    pub fn render(
        &mut self,
        target_swap_chain_texture: &wgpu::Texture, // This is the actual swap chain texture
        spheres: &[Arc<Sphere>], // Pass spheres directly instead of Scene
        lights: &[Arc<PointLight>], // Added lights parameter
        renderer_camera: &RendererCamera
    ) -> anyhow::Result<()> {
        // 1. Update Camera Buffer
        let view_matrix = renderer_camera.view_matrix();
        let projection_matrix = renderer_camera.projection_matrix();
        let camera_world_pos = renderer_camera.transform.position;

        let camera_gpu = CameraGpu {
            position: [camera_world_pos.x, camera_world_pos.y, camera_world_pos.z, 1.0],
            view_projection: (projection_matrix * view_matrix).to_cols_array_2d(),
            inv_projection: projection_matrix.inverse().to_cols_array_2d(),
            inv_view: view_matrix.inverse().to_cols_array_2d(),
        };
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&camera_gpu));

        // 2. Update Sphere and Material Buffers
        let mut material_map: HashMap<usize, u32> = HashMap::new(); // Using usize from Arc pointer for Material
        let mut materials_gpu_list: Vec<MaterialGpu> = Vec::new();
        let mut spheres_gpu_list: Vec<SphereGpu> = Vec::new();

        // Add a default material for objects without one, or if lookup fails
        let default_material_gpu = MaterialGpu {
            color: [1.0, 0.0, 1.0, 1.0], // Magenta for error/default
            material_type: 0, // Lambertian
            smoothness: 0.5,
            _padding: [0,0],
        };
        materials_gpu_list.push(default_material_gpu);
        let default_material_idx = 0u32;

        for sphere_arc in spheres { // Use passed spheres slice
            let sphere_item: &Sphere = sphere_arc; // Deref Arc<Sphere> to &Sphere

            let material_idx = if let Some(mat_arc) = &sphere_item.material {
                // Use data pointer of Arc as key for uniqueness.
                // Arc::as_ptr returns *const dyn Material (fat pointer), we need just the data part.
                let mat_ptr = Arc::as_ptr(mat_arc) as *const () as usize; 
                
                *material_map.entry(mat_ptr).or_insert_with(|| {
                    let new_idx = materials_gpu_list.len() as u32;
                    // Attempt to downcast or identify material type
                    // For now, only supporting Lambertian explicitly from scene.
                    // In a real scenario, you'd check mat_arc.is::<LambertianMaterial>() etc.
                    // Or have Material trait provide MaterialGpu directly.
                    let albedo = mat_arc.albedo(); // From Material trait
                    let material_gpu = MaterialGpu {
                        color: [albedo.r, albedo.g, albedo.b, albedo.a],
                        material_type: 0, // Assume Lambertian
                        smoothness: mat_arc.get_properties().roughness, // Example
                        _padding: [0,0],
                    };
                    materials_gpu_list.push(material_gpu);
                    new_idx
                })
            } else {
                default_material_idx
            };
            
            // Assuming sphere_item.center is world-space
            let sphere_gpu = SphereGpu {
                center: [sphere_item.center.x, sphere_item.center.y, sphere_item.center.z, 0.0], // w = 0 for position vector
                radius: sphere_item.radius,
                material_index: material_idx,
                _padding: [0,0],
            };
            spheres_gpu_list.push(sphere_gpu);
        }

        // Recreate sphere buffer if data exists
        if !spheres_gpu_list.is_empty() {
            self.sphere_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sphere Buffer (Dynamic)"),
                contents: bytemuck::cast_slice(&spheres_gpu_list),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        } else {
            // Handle no spheres: create a minimal buffer to satisfy binding
            let dummy_sphere = SphereGpu { center: [0.0,0.0,0.0,0.0], radius: 0.0, material_index: 0, _padding: [0,0]};
             self.sphere_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sphere Buffer (Empty Placeholder)"),
                contents: bytemuck::bytes_of(&dummy_sphere),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        }

        // Recreate material buffer (even if only default material exists)
        self.material_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Buffer (Dynamic)"),
            contents: bytemuck::cast_slice(&materials_gpu_list),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        
        // 3. Update Light Buffer
        let mut lights_gpu_list: Vec<PointLightGpu> = Vec::new();
        
        for light_arc in lights {
            let light_item: &PointLight = light_arc;
            let light_gpu = PointLightGpu {
                position: [light_item.position.x, light_item.position.y, light_item.position.z, 0.0],
                color: [light_item.color.r, light_item.color.g, light_item.color.b, light_item.color.a],
                intensity: light_item.intensity,
                range: light_item.range, // Use the range from PointLight
                _padding: [0, 0],
            };
            lights_gpu_list.push(light_gpu);
        }
        
        // Handle case with no lights - add a default disabled light
        if lights_gpu_list.is_empty() {
            let default_light = PointLightGpu {
                position: [0.0, 0.0, 0.0, 0.0],
                color: [0.0, 0.0, 0.0, 0.0], // Black light (disabled)
                intensity: 0.0,
                range: 0.0,
                _padding: [0, 0],
            };
            lights_gpu_list.push(default_light);
        }
        
        // Recreate light buffer
        self.light_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer (Dynamic)"),
            contents: bytemuck::cast_slice(&lights_gpu_list),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Recreate compute bind group
        self.compute_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Raytrace Compute Bind Group (Recreated)"),
            layout: &self.compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.sphere_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.output_texture_view),
                },
            ],
        });

        // 3. Create Command Encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // 4. Run Compute Pass (Raytracing)
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Raytrace Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            
            // Dispatch based on output texture dimensions
            // Divide by workgroup size (e.g., 8x8 as defined in raytrace.wgsl)
            let workgroup_size_x = 8; 
            let workgroup_size_y = 8;
            let num_workgroups_x = (self.output_texture.width() + workgroup_size_x - 1) / workgroup_size_x;
            let num_workgroups_y = (self.output_texture.height() + workgroup_size_y - 1) / workgroup_size_y;
            compute_pass.dispatch_workgroups(num_workgroups_x, num_workgroups_y, 1);
        } // compute_pass is dropped, releasing the borrow on encoder

        // 5. Blit Pass (Copy compute output_texture to swap_chain_texture via render pipeline)
        let target_swap_chain_view = target_swap_chain_texture.create_view(&wgpu::TextureViewDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blit Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_swap_chain_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }), // Clear to black
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.blit_pipeline);
            render_pass.set_bind_group(0, &self.blit_bind_group, &[]);
            render_pass.draw(0..3, 0..1); // Draw 3 vertices for the fullscreen triangle
        } // render_pass is dropped

        // 6. Submit command buffer
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    /// Resize GPU resources (e.g., output texture) when window size changes
    pub fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        if width == 0 || height == 0 {
            warn!("Attempted to resize GpuRenderer to zero dimensions, skipping.");
            return Ok(());
        }
        info!("Resizing GpuRenderer to {}x{}", width, height);
        self.config.width = width;
        self.config.height = height;

        // Update surface configuration with new size
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);

        // Recreate output texture with new size
        let output_texture_descriptor = wgpu::TextureDescriptor {
            label: Some("Output Texture (Rgba8Unorm)"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm, 
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        self.output_texture = self.device.create_texture(&output_texture_descriptor);
        self.output_texture_view = self.output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Recreate compute bind group because output_texture_view changed
        self.compute_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Raytrace Compute Bind Group (resized)"),
            layout: &self.compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { // Camera
                    binding: 0,
                    resource: self.camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry { // Spheres
                    binding: 1,
                    resource: self.sphere_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry { // Materials
                    binding: 2,
                    resource: self.material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry { // Lights
                    binding: 3,
                    resource: self.light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry { // Output Texture View
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&self.output_texture_view),
                },
            ],
        });
        
        // Recreate blit bind group because output_texture_view changed
        self.blit_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blit Bind Group (resized)"),
            layout: &self.blit_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.output_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        Ok(())
    }

    /// Get renderer configuration
    pub fn config(&self) -> &GpuRendererConfig {
        &self.config
    }

    /// Check if the renderer is initialized
    pub fn is_initialized(&self) -> bool {
        // If the GpuRenderer instance exists, we assume it's initialized
        // as device, queue, surface are not Options in the struct definition.
        true 
    }

    /// Get the device reference
    pub fn device(&self) -> Option<&Device> {
        Some(self.device.as_ref())
    }

    /// Get the queue reference
    pub fn queue(&self) -> Option<&Queue> {
        Some(self.queue.as_ref())
    }

    /// Get the surface configuration
    pub fn surface_config(&self) -> Option<&SurfaceConfiguration> {
        Some(&self.surface_config) // Assuming this should also be wrapped if consistent
    }

    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        if self.surface_config.height == 0 { return 1.0; } // Avoid division by zero
        self.surface_config.width as f32 / self.surface_config.height as f32
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
