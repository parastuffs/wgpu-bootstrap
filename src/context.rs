use std::sync::Arc;

use pollster::FutureExt;
use winit::window::Window;

fn create_depth_texture(
    size: winit::dpi::PhysicalSize<u32>,
    depth_format: wgpu::TextureFormat,
    device: &wgpu::Device,
) -> wgpu::TextureView {
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture Descriptor"),
        size: wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: depth_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub struct Context {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth_format: wgpu::TextureFormat,
    depth_texture_view: Option<wgpu::TextureView>,
    size: winit::dpi::PhysicalSize<u32>,
    window: Arc<Window>,
}

impl Context {
    pub fn new(window: Window) -> Self {
        let window_arc = Arc::new(window);
        let size = window_arc.inner_size();
        let instance = Self::create_gpu_instance();
        let surface = instance.create_surface(window_arc.clone()).unwrap();
        let adapter = Self::create_adapter(instance, &surface);
        let (device, queue) = Self::create_device(&adapter);
        let surface_caps = surface.get_capabilities(&adapter);
        let config = Self::create_surface_config(size, surface_caps);
        surface.configure(&device, &config);

        let depth_format: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            depth_format,
            depth_texture_view: None,
            window: window_arc,
        }
    }

    fn create_surface_config(
        size: winit::dpi::PhysicalSize<u32>,
        capabilities: wgpu::SurfaceCapabilities,
    ) -> wgpu::SurfaceConfiguration {
        let surface_format = capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(capabilities.formats[0]);

        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

    fn create_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .block_on()
            .unwrap()
    }

    fn create_adapter(instance: wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .block_on()
            .unwrap()
    }

    fn create_gpu_instance() -> wgpu::Instance {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;

            self.config.width = new_size.width;
            self.config.height = new_size.height;

            self.surface.configure(&self.device, &self.config);
            match self.depth_texture_view {
                None => {}
                Some(_) => {
                    self.depth_texture_view = Some(create_depth_texture(
                        self.size,
                        self.depth_format,
                        self.device(),
                    ));
                }
            };

            println!("Resized to {:?} from state!", new_size);
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn size(&self) -> &winit::dpi::PhysicalSize<u32> {
        &self.size
    }

    pub fn depth_format(&self) -> &wgpu::TextureFormat {
        &self.depth_format
    }

    pub fn depth_texture_view(&mut self) -> &wgpu::TextureView {
        match self.depth_texture_view {
            Some(_) => {}
            None => {
                println!("Depth Texture Created");
                self.depth_texture_view = Some(create_depth_texture(
                    self.size,
                    self.depth_format,
                    self.device(),
                ));
            }
        };
        self.depth_texture_view.as_ref().unwrap()
    }
}

// pub struct Context<'a> {
//     surface: wgpu::Surface<'a>,
//     device: wgpu::Device,
//     queue: wgpu::Queue,
//     config: wgpu::SurfaceConfiguration,
//     size: winit::dpi::PhysicalSize<u32>,
//     depth_format: wgpu::TextureFormat,
//     depth_texture_view: Option<wgpu::TextureView>,
//     window: &'a Window,
// }
//
// impl<'a> Context<'a> {
//     // Creating some of the wgpu types requires async code
//     pub async fn new(window: &'a Window) -> Self {
//         let size = window.inner_size();
//
//         // The instance is a handle to our GPU
//         // Backends::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
//         let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
//             backends: wgpu::Backends::PRIMARY,
//             ..Default::default()
//         });
//
//         // # Safety
//         //
//         // The surface needs to live as long as the window that created it.
//         // State owns the window so this should be safe.
//         let surface = instance.create_surface(window).unwrap();
//
//         let adapter = instance
//             .request_adapter(&wgpu::RequestAdapterOptions {
//                 power_preference: wgpu::PowerPreference::default(),
//                 compatible_surface: Some(&surface),
//                 force_fallback_adapter: false,
//             })
//             .await
//             .unwrap();
//
//         let (device, queue) = adapter
//             .request_device(
//                 &wgpu::DeviceDescriptor {
//                     required_features: wgpu::Features::empty(),
//                     // WebGL doesn't support all of wgpu's features, so if
//                     // we're building for the web we'll have to disable some.
//                     required_limits: wgpu::Limits::default(),
//                     label: None,
//                     memory_hints: Default::default(),
//                 },
//                 None, // Trace path
//             )
//             .await
//             .unwrap();
//
//         let surface_caps = surface.get_capabilities(&adapter);
//         // Shader code in this tutorial assumes an sRGB surface texture. Using a different
//         // one will result all the colors coming out darker. If you want to support non
//         // sRGB surfaces, you'll need to account for that when drawing to the frame.
//         let surface_format = surface_caps
//             .formats
//             .iter()
//             .copied()
//             .find(|f| f.is_srgb())
//             .unwrap_or(surface_caps.formats[0]);
//         let config = wgpu::SurfaceConfiguration {
//             usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
//             format: surface_format,
//             width: size.width,
//             height: size.height,
//             present_mode: surface_caps.present_modes[0],
//             alpha_mode: surface_caps.alpha_modes[0],
//             view_formats: vec![],
//             desired_maximum_frame_latency: 2,
//         };
//         surface.configure(&device, &config);
//
//         let depth_format: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
//
//         Self {
//             surface,
//             device,
//             queue,
//             config,
//             size,
//             depth_format,
//             depth_texture_view: None,
//             window,
//         }
//     }
//
//     pub fn window(&self) -> &Window {
//         &self.window
//     }
//
//     pub fn surface(&self) -> &wgpu::Surface {
//         &self.surface
//     }
//
//     pub fn device(&self) -> &wgpu::Device {
//         &self.device
//     }
//
//     pub fn queue(&self) -> &wgpu::Queue {
//         &self.queue
//     }
//
//     pub fn config(&self) -> &wgpu::SurfaceConfiguration {
//         &self.config
//     }
//
//     pub fn size(&self) -> &winit::dpi::PhysicalSize<u32> {
//         &self.size
//     }
//
//     pub fn depth_format(&self) -> &wgpu::TextureFormat {
//         &self.depth_format
//     }
//
//     pub fn depth_texture_view(&mut self) -> &wgpu::TextureView {
//         match self.depth_texture_view {
//             Some(_) => {}
//             None => {
//                 println!("Depth Texture Created");
//                 self.depth_texture_view = Some(create_depth_texture(
//                     self.size,
//                     self.depth_format,
//                     self.device(),
//                 ));
//             }
//         };
//         self.depth_texture_view.as_ref().unwrap()
//     }
//
//     pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
//         if new_size.width > 0 && new_size.height > 0 {
//             self.size = new_size;
//             self.config.width = new_size.width;
//             self.config.height = new_size.height;
//             self.surface.configure(&self.device, &self.config);
//             match self.depth_texture_view {
//                 None => {}
//                 Some(_) => {
//                     self.depth_texture_view = Some(create_depth_texture(
//                         self.size,
//                         self.depth_format,
//                         self.device(),
//                     ));
//                 }
//             };
//         }
//     }
// }
