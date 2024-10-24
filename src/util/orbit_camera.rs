use std::f32::consts::PI;

use cgmath::prelude::*;
use eframe::{
    egui::{self, PointerButton},
    wgpu::{self, util::DeviceExt},
};

use crate::runner::Context;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view: cgmath::Matrix4::identity().into(),
            proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_proj(&mut self, matrix: cgmath::Matrix4<f32>) {
        self.proj = matrix.into();
    }

    pub fn update_view(&mut self, matrix: cgmath::Matrix4<f32>) {
        self.view = matrix.into();
    }

    pub fn desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Camera Bind Group Layout"),
        }
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OrbitCamera {
    fovy: f32,
    aspect: f32,
    near: f32,
    far: f32,
    polar: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    uniform: CameraUniform,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    orbiting: bool,
}

impl OrbitCamera {
    pub fn new(context: &Context, fovy: f32, aspect: f32, near: f32, far: f32) -> Self {
        let uniform = CameraUniform::new();
        let buffer = context
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let bind_group_layout = context
            .device()
            .create_bind_group_layout(&CameraUniform::desc());
        let bind_group = context
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: Some("Camera Bind Group"),
            });

        let mut res = Self {
            fovy,
            aspect,
            near,
            far,
            polar: cgmath::point3(1.0, 0.0, 0.0),
            target: cgmath::point3(0.0, 0.0, 0.0),
            up: cgmath::Vector3::unit_y(),
            uniform,
            buffer,
            bind_group,
            orbiting: false,
        };
        res.update(context);
        res
    }

    pub fn update(&mut self, context: &Context) {
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.near, self.far);

        let pos = cgmath::point3(
            self.polar.x * self.polar.z.cos() * self.polar.y.cos(),
            self.polar.x * self.polar.z.sin(),
            self.polar.x * self.polar.z.cos() * self.polar.y.sin(),
        );

        let pos = cgmath::Point3::from_vec(pos.to_vec() + self.target.to_vec());

        let view = cgmath::Matrix4::look_at_rh(pos, self.target, self.up);
        let projection_matrix = OPENGL_TO_WGPU_MATRIX * proj;
        self.uniform.update_proj(projection_matrix);
        self.uniform.update_view(view);
        context
            .queue()
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }

    pub fn set_target(&mut self, target: cgmath::Point3<f32>) -> &mut Self {
        self.target = target;
        self
    }

    pub fn set_aspect(&mut self, aspect: f32) -> &mut Self {
        self.aspect = aspect;
        self
    }

    pub fn set_polar(&mut self, polar: cgmath::Point3<f32>) -> &mut Self {
        self.polar = polar;
        self
    }

    pub fn set_radius(&mut self, value: f32) -> &mut Self {
        if value >= 0.0 {
            self.polar.x = value;
        } else {
            self.polar.x = 0.0;
        }
        self
    }

    pub fn radius(&self) -> f32 {
        self.polar.x
    }

    pub fn set_longitude(&mut self, value: f32) -> &mut Self {
        let mut value = value;
        while value > PI {
            value -= 2.0 * PI;
        }
        while value < -PI {
            value += 2.0 * PI;
        }

        self.polar.y = value;

        self
    }

    pub fn longitude(&self) -> f32 {
        self.polar.y
    }

    pub fn set_latitude(&mut self, value: f32) -> &mut Self {
        self.polar.z = value.clamp(-PI / 2.0, PI / 2.0);
        self
    }

    pub fn latitude(&self) -> f32 {
        self.polar.z
    }

    pub fn start_orbiting(&mut self) {
        self.orbiting = true;
    }

    pub fn stop_orbiting(&mut self) {
        self.orbiting = false;
    }

    pub fn delta_angles(&mut self, context: &Context, angles: (f32, f32)) {
        if self.orbiting {
            let longitude = self.longitude();
            let latitude = self.latitude();
            self.set_longitude(longitude + 0.01 * angles.0)
                .set_latitude(latitude + 0.01 * angles.1)
                .update(context);
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn input(&mut self, input: egui::InputState, context: &Context) {
        if input.pointer.button_down(PointerButton::Primary) {
            self.start_orbiting();
        }
        if input.pointer.button_released(PointerButton::Primary) {
            self.stop_orbiting();
        }
        let delta = input.pointer.motion();
        if let Some(delta) = delta {
            self.delta_angles(context, (delta.x, delta.y));
        }
    }
}
