use crate::context::Context;

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
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
}

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32
}

impl Camera {
    pub fn build_view_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up)
    }

    pub fn build_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar)
    }

    pub fn create_camera_uniform(&self) -> CameraUniform {
        CameraUniform { view: self.build_view_matrix().into(), proj: self.build_projection_matrix().into() }
    }

    pub fn create_camera_buffer(&self, context: &Context) -> wgpu::Buffer {
        let camera_uniform = self.create_camera_uniform();
        context.create_buffer(&[camera_uniform], wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)
    }

    pub fn create_camera_bind_group_layout(&self, context: &Context) -> wgpu::BindGroupLayout {
        context.create_bind_group_layout("Camera Bind Group Layout", &[
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
        ])
    }

    pub fn create_camera_bind_group(&self, context: &Context) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let camera_buffer = self.create_camera_buffer(context);
        
        let bind_group_layout = context.create_bind_group_layout("Camera Bind Group Layout", &[
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

        let bind_group = context.create_bind_group("Matrix Bind Group", &bind_group_layout, &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }
        ]);

        (camera_buffer, bind_group_layout, bind_group)
    }

    pub fn update_camera_buffer(&self, camera_buffer: &wgpu::Buffer, context: &Context) {
        let camera_uniform = self.create_camera_uniform();
        context.update_buffer(camera_buffer, &[camera_uniform]);
    }
}