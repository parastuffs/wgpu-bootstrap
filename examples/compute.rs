use wgpu_bootstrap::{
    window::Window,
    frame::Frame,
    cgmath::{ self, prelude::* },
    application::Application,
    texture::create_texture_bind_group,
    context::Context,
    camera::Camera,
    default::{ Vertex, Instance, InstanceRaw },
    geometry::icosphere,
    computation::Computation,
    wgpu,
};

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 1.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 1.5);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ComputeData {
    delta_time: f32,
    nb_instances: u32,
    rotation_speed: f32,
}

struct MyApp {
    diffuse_bind_group: wgpu::BindGroup,
    camera_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    compute_instances_bind_group: wgpu::BindGroup,
    compute_data_buffer: wgpu::Buffer,
    compute_data_bind_group: wgpu::BindGroup,
    nb_indices: usize,
}

impl MyApp {
    fn new(context: &Context) -> Self {
        let texture = context.create_srgb_texture("happy-tree.png", include_bytes!("happy-tree.png"));
    
        let diffuse_bind_group = create_texture_bind_group(context, &texture);
    
        let camera = Camera {
            eye: (0.0, 30.0, 45.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: context.get_aspect_ratio(),
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let (_camera_buffer, camera_bind_group) = camera.create_camera_bind_group(context);
    
        let pipeline = context.create_render_pipeline(
            "Render Pipeline",
            include_str!("shader_instances.wgsl"),
            &[Vertex::desc(), InstanceRaw::desc()],
            &[
                &context.texture_bind_group_layout,
                &context.camera_bind_group_layout,
            ],
            wgpu::PrimitiveTopology::TriangleList
        );


        let (vertices, indices) = icosphere(1);
    
        let vertex_buffer = context.create_buffer(vertices.as_slice(), wgpu::BufferUsages::VERTEX);
        let index_buffer = context.create_buffer(indices.as_slice(), wgpu::BufferUsages::INDEX);

        let instances = (0..NUM_INSTANCES_PER_ROW*NUM_INSTANCES_PER_ROW).map(|index| {
            let x = index % NUM_INSTANCES_PER_ROW;
            let z = index / NUM_INSTANCES_PER_ROW;
            let position = cgmath::Vector3 { x: x as f32 * 3.0, y: 0.0, z: z as f32 * 3.0 } - INSTANCE_DISPLACEMENT;
            let rotation = if position.is_zero() {
                // this is needed so an object at (0, 0, 0) won't get scaled to zero
                // as Quaternions can effect scale if they're not created correctly
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
            } else {
                cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
            };

            Instance {
                position, rotation,
            }
        }).collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = context.create_buffer(instance_data.as_slice(), wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE);
        
        // This compute pipeline will use 2 bind group as declared in its source
        let compute_pipeline = context.create_compute_pipeline("Compute Pipeline", include_str!("compute.wgsl"));

        // This is the first bind group for the compute pipeline
        let compute_instances_bind_group = context.create_bind_group(
            "Compute Bind Group",
            &compute_pipeline.get_bind_group_layout(0),
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: instance_buffer.as_entire_binding(),
                }
            ]
        );

        // This is the second bind group for the compute pipeline. The buffer will be updated each
        // frame
        let compute_data = ComputeData {
            delta_time: 0.016,
            nb_instances: 100,
            rotation_speed: 1.0,
        };
        let compute_data_buffer = context.create_buffer(&[compute_data], wgpu::BufferUsages::UNIFORM);
        let compute_data_bind_group = context.create_bind_group(
            "Compute Data", 
            &compute_pipeline.get_bind_group_layout(1), 
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: compute_data_buffer.as_entire_binding(),
                }
            ]
        );

        Self {
            diffuse_bind_group,
            camera_bind_group,
            pipeline,
            compute_pipeline,
            vertex_buffer,
            index_buffer,
            instances,
            instance_buffer,
            compute_instances_bind_group,
            compute_data_buffer,
            compute_data_bind_group,
            nb_indices: indices.len(),
        }
    }
}

impl Application for MyApp {
    fn render(&self, context: &Context) -> Result<(), wgpu::SurfaceError> {
        let mut frame = Frame::new(context)?;

        {
            let mut render_pass = frame.begin_render_pass(wgpu::Color {r: 0.1, g: 0.2, b: 0.3, a: 1.0});

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..(self.nb_indices as u32), 0, 0..self.instances.len() as _);
        }

        frame.present();

        Ok(())
    }

    fn update(&mut self, context: &Context, delta_time: f32) {
        // Update the Buffer that contains the delta_time
        let compute_data = ComputeData {
            delta_time,
            nb_instances: 100,
            rotation_speed: 2.0,
        }; 
        context.update_buffer(&self.compute_data_buffer, &[compute_data]);

        let mut computation = Computation::new(context);

        {
            let mut compute_pass = computation.begin_compute_pass();

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_instances_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.compute_data_bind_group, &[]);
            compute_pass.dispatch_workgroups(2, 1, 1);
        }

        computation.run();
    }
}

fn main() {
    let window = Window::new();

    let context = window.get_context();

    let my_app = MyApp::new(context);

    window.run(my_app);
}
