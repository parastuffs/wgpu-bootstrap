use cgmath::SquareMatrix;
use wgpu_bootstrap::{
    window::Window,
    frame::Frame,
    cgmath,
    application::Application,
    texture::Texture,
    context::Context
};
use winit::event::{KeyboardInput, WindowEvent};


#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MatrixUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>{
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
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], },
    Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], },
    Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], },
    Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], },
    Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], },
];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

struct MyApp {
    texture: Texture,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    diffuse_bind_group: wgpu::BindGroup,
    eye: cgmath::Point3<f32>,
    matrix_uniform: MatrixUniform,
    matrix_buffer: wgpu::Buffer,
    matrix_bind_group_layout: wgpu::BindGroupLayout,
    matrix_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer
}

impl MyApp {
    fn new(context: &Context) -> Self {
        let texture = context.create_texture("happy-tree.png", include_bytes!("happy-tree.png"));
    
        let texture_bind_group_layout =context.create_bind_group_layout( "Bind Group Layout", &[
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
    
        let diffuse_bind_group = context.create_bind_group("Bind Group", &texture_bind_group_layout, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            }
        ]);
    
        let mut eye = cgmath::Point3::new(0.0, 1.0, 2.0);
    
        let mut matrix_uniform = MatrixUniform {
            model: cgmath::Matrix4::identity().into(),
            view: cgmath::Matrix4::look_at_rh(eye, (0.0, 0.0, 0.0).into(), cgmath::Vector3::unit_y()).into(),
            proj: (OPENGL_TO_WGPU_MATRIX * cgmath::perspective(cgmath::Deg(45.0), context.config.width as f32 / context.config.height as f32, 0.1, 100.0)).into(),
        };
    
        let matrix_buffer = context.create_buffer(&[matrix_uniform], wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
    
        let matrix_bind_group_layout = context.create_bind_group_layout("Matrix Bind Group Layout", &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }
        ]);
    
        let matrix_bind_group = context.create_bind_group("Matrix Bind Group", &matrix_bind_group_layout, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: matrix_buffer.as_entire_binding(),
            }
        ]);
    
        let pipeline = context.create_render_pipeline("Render Pipeline", include_str!("shader.wgsl"), Vertex::desc(), &[
            &texture_bind_group_layout,
            &matrix_bind_group_layout,
        ]);
    
        let vertex_buffer = context.create_buffer(VERTICES, wgpu::BufferUsages::VERTEX);
        let index_buffer = context.create_buffer(INDICES, wgpu::BufferUsages::INDEX);

        Self {
            texture,
            texture_bind_group_layout,
            diffuse_bind_group,
            eye,
            matrix_uniform,
            matrix_buffer,
            matrix_bind_group_layout,
            matrix_bind_group,
            pipeline,
            vertex_buffer,
            index_buffer,
        }
    }
}

impl Application for MyApp {
    fn render(&self, context: &Context) -> Result<(), wgpu::SurfaceError> {
        let mut frame = Frame::new(context)?;

        {
            let mut render_pass = frame.begin_render_pass();

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.matrix_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1);
        }

        frame.present();

        Ok(())
    }
}

fn main() {
    let window = Window::new();

    let context = window.get_context();

    let myApp = MyApp::new(context);

    window.run(myApp);
}
