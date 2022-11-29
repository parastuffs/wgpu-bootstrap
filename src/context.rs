use winit::{
    window::{Window as WinitWindow},
};
use crate::texture::Texture;
use bytemuck::Pod;
use wgpu::util::DeviceExt;

fn create_bind_group_layout(device: &wgpu::Device, label:&str, bind_group_layout_entries: &[wgpu::BindGroupLayoutEntry]) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: bind_group_layout_entries,
        label: Some(label),
    })
}

pub struct Context {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub depth_texture: Texture,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl Context {
    pub async fn new(window: &WinitWindow) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let camera_bind_group_layout = create_bind_group_layout(&device, "Camera Bind Group Layout", &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ]);


        let texture_bind_group_layout = create_bind_group_layout(&device, "Texture Bind Group Layout", &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ]);

        Self {
            surface,
            device,
            queue,
            size,
            config,
            depth_texture,
            camera_bind_group_layout,
            texture_bind_group_layout,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }

        self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
    }

    pub fn create_compute_pipeline(&self, label: &str, source: &str, /* bind_group_layouts: &[&wgpu::BindGroupLayout] */) -> wgpu::ComputePipeline {
        // let layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //     label: Some(format!("{} {}", label, "Layout").as_str()),
        //     bind_group_layouts,
        //     push_constant_ranges: &[],
        // });
        self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(label),
            layout: None, //Some(&layout),
            module: &self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(format!("{} Shader", label).as_str()),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            }),
            entry_point: "main",
        })
    }

    pub fn create_render_pipeline(&self, label: &str, source: &str, vertex_buffer_layouts: &[wgpu::VertexBufferLayout], bind_group_layouts: &[&wgpu::BindGroupLayout], topology: wgpu::PrimitiveTopology) -> wgpu::RenderPipeline {
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(format!("{} Shader", label).as_str()),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
    
        let render_pipeline_layout =
            self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(format!("{} Layout", label).as_str()),
                bind_group_layouts,
                push_constant_ranges: &[],
            });
    
        self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: vertex_buffer_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        })
    }

    pub fn create_buffer<T>(&self, data: &[T], usage: wgpu::BufferUsages) -> wgpu::Buffer
    where T: Pod
    {
        self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(data),
                usage: usage | wgpu::BufferUsages::COPY_DST,
            }
        )
    }

    pub fn update_buffer<T>(&self, buffer: &wgpu::Buffer, data: &[T])
    where T: Pod
    {
        self.queue.write_buffer(buffer, 0, bytemuck::cast_slice(data));
    }

    pub fn create_texture(&self, label: &str, bytes: &[u8]) -> Texture {
        Texture::from_bytes(&self.device, &self.queue, bytes, label, false).unwrap()
    }

    pub fn create_srgb_texture(&self, label: &str, bytes: &[u8]) -> Texture {
        Texture::from_bytes(&self.device, &self.queue, bytes, label, true).unwrap()
    }

    pub fn create_bind_group_layout(&self, label:&str, bind_group_layout_entries: &[wgpu::BindGroupLayoutEntry]) -> wgpu::BindGroupLayout {
        create_bind_group_layout(&self.device, label, bind_group_layout_entries)
    }

    pub fn create_bind_group(&self, label: &str, bind_group_layout: &wgpu::BindGroupLayout, bind_group_entries: &[wgpu::BindGroupEntry]) -> wgpu::BindGroup {
        self.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: bind_group_layout,
                entries: bind_group_entries,
                label: Some(label),
            }
        )
    }

    pub fn get_aspect_ratio(&self) -> f32{
        self.config.width as f32 / self.config.height as f32
    }
}
