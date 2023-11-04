use wgpu::{util::DeviceExt, TextureView};
use wgpu_bootstrap::context::Context;
use wgpu_bootstrap::runner::App;
use winit::event::WindowEvent;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, matrix: cgmath::Matrix4<f32>) {
        self.view_proj = matrix.into();
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    ///////////////////////////////////
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    //////////////////////////////
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
    //////////////////////////////
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 1.0, 0.0],
    },
    //////////////////////////////
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [0.0, 1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [0.0, 1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.0, 1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [0.0, 1.0, 1.0],
    },
    //////////////////////////////
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [1.0, 0.0, 1.0],
    },
];

#[rustfmt::skip]
const INDEXES: &[u32] = &[
     0,  1,  2,  0,  2,  3,
     4,  5,  6,  4,  6,  7,
     8,  9, 10,  8, 10, 11,
    12, 13, 14, 12, 14, 15,
    16, 17, 18, 16, 18, 19,
    20, 21, 22, 20, 22, 23,
];

pub struct CubeApp {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    num_indices: u32,
    camera_bind_group: wgpu::BindGroup,
}

impl CubeApp {
    pub fn new(context: &mut Context) -> Self {
        context.window().set_title("Cube App");

        let index_buffer = context
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDEXES),
                usage: wgpu::BufferUsages::INDEX,
            });

        let num_indices = INDEXES.len() as u32;

        let vertex_buffer =
            context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let view = cgmath::Matrix4::look_at_rh(
            (2.0, 1.0, 1.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_y(),
        );
        let proj = cgmath::perspective(
            cgmath::Deg(45.0),
            context.config().width as f32 / context.config().height as f32,
            0.1,
            100.0,
        );
        let projection_matrix = OPENGL_TO_WGPU_MATRIX * proj * view;

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(projection_matrix);

        let camera_buffer =
            context
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Buffer"),
                    contents: bytemuck::cast_slice(&[camera_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let camera_bind_group_layout =
            context
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("Camera Bind Group Layout"),
                });

        let camera_bind_group = context
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });

        let shader = context
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let pipeline_layout =
            context
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&camera_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            context
                .device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Vertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: context.config().format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: *context.depth_format(),
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });

        Self {
            vertex_buffer,
            index_buffer,
            render_pipeline,
            num_indices,
            camera_bind_group,
        }
    }
}

impl App for CubeApp {
    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn render(&mut self, context: &mut Context, view: &TextureView) {
        let mut encoder =
            context
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: context.depth_texture_view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        // submit will accept anything that implements IntoIter
        context.queue().submit(std::iter::once(encoder.finish()));
    }
}
