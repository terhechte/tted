use std::num::NonZeroU32;

use forma::{cpu, gpu, prelude::*};
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::{App, RunContext, Runner};

#[derive(Debug)]
pub struct CpuRunner {
    composition: Composition,
    renderer: cpu::Renderer,
    buffer: Vec<u8>,
    layer_cache: BufferLayerCache,
    window: Window,
    layout: LinearLayout,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
}

impl CpuRunner {
    pub fn new(event_loop: &EventLoop<()>, width: u32, height: u32) -> Self {
        let composition = Composition::new();
        let mut renderer = cpu::Renderer::new();
        let layer_cache = renderer.create_buffer_layer_cache().unwrap();

        let window = WindowBuilder::new()
            .with_title("test")
            .with_inner_size(PhysicalSize::new(width, height))
            .build(event_loop)
            .unwrap();

        let layout = LinearLayout::new(width as usize, width as usize * 4, height as usize);

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            ..Default::default()
        }))
        .expect("failed to find an appropriate adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .expect("failed to get device");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        };

        surface.configure(&device, &config);

        Self {
            composition,
            renderer,
            layer_cache,
            window,
            buffer: vec![0; (width * 4 * height) as usize],
            layout,
            device,
            queue,
            surface,
            config,
        }
    }
}

impl Runner for CpuRunner {
    fn resize(&mut self, width: u32, height: u32) {
        self.buffer.resize((width * 4 * height) as usize, 0);
        self.layout = LinearLayout::new(width as usize, width as usize * 4, height as usize);

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render<'a>(&mut self, app: &mut dyn App, context: RunContext<'a>) {
        app.update(&context);

        app.compose(&mut self.composition, &context);

        self.renderer.render(
            &mut self.composition,
            &mut BufferBuilder::new(&mut self.buffer, &mut self.layout)
                .layer_cache(self.layer_cache.clone())
                .build(),
            BGR1,
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            },
            None,
        );

        let frame = self.surface.get_current_texture().unwrap();

        self.queue.write_texture(
            frame.texture.as_image_copy(),
            &self.buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(self.config.width * 4),
                rows_per_image: NonZeroU32::new(self.config.height),
            },
            wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(None);

        frame.present();
    }
}

pub struct GpuRunner {
    composition: Composition,
    renderer: gpu::Renderer,
    window: Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
}

impl GpuRunner {
    pub fn new(
        event_loop: &EventLoop<()>,
        width: u32,
        height: u32,
        power_preference: wgpu::PowerPreference,
    ) -> Self {
        let composition = Composition::new();

        let window = WindowBuilder::new()
            .with_title("test")
            .with_inner_size(PhysicalSize::new(width, height))
            .build(event_loop)
            .unwrap();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference,
            ..Default::default()
        }))
        .expect("failed to find an appropriate adapter");

        let adapter_features = adapter.features();
        let has_timestamp_query = adapter_features.contains(wgpu::Features::TIMESTAMP_QUERY);

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::TIMESTAMP_QUERY & adapter_features,
                limits: wgpu::Limits {
                    max_texture_dimension_2d: 4096,
                    max_storage_buffer_binding_size: 1 << 30,
                    ..wgpu::Limits::downlevel_defaults()
                },
            },
            None,
        ))
        .expect("failed to get device");

        let swap_chain_format = surface.get_supported_formats(&adapter)[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swap_chain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        };

        surface.configure(&device, &config);

        let renderer = gpu::Renderer::new(&device, swap_chain_format, has_timestamp_query);

        Self {
            composition,
            renderer,
            window,
            device,
            queue,
            surface,
            config,
        }
    }
}

impl Runner for GpuRunner {
    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    fn render<'a>(&mut self, app: &mut dyn App, context: RunContext<'a>) {
        app.update(&context);

        // let compose_duration = measure(|| {
        app.compose(&mut self.composition, &context);
        // });

        self.renderer.render(
            &mut self.composition,
            &self.device,
            &self.queue,
            &self.surface,
            self.config.width,
            self.config.height,
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.0,
            },
        );
    }
}
