use std::sync::Arc;

use super::system::System;
use anyhow::{anyhow, Context, Result};
use winit::event_loop::EventLoop;

pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

pub struct Window {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    texture_format: wgpu::TextureFormat,
    window: Arc<winit::window::Window>,
    event_loop: Option<EventLoop<()>>,
    width: u32,
    height: u32,
}

impl Window {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn size(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        }
    }

    pub fn texture_format(&self) -> wgpu::TextureFormat {
        self.texture_format
    }

    pub fn surface_texture(&self) -> Result<wgpu::SurfaceTexture> {
        self.surface
            .get_current_texture()
            .context("Could not get current texture")
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub(crate) fn take_event_loop(&mut self) -> Option<EventLoop<()>> {
        self.event_loop.take()
    }
}

impl<'context> System<'context> for Window {
    type Dependencies = &'context WindowConfig;

    fn create(config: &'context WindowConfig) -> Result<Self> {
        let events = EventLoop::new().context("Error creating window")?;

        let window = Arc::new(
            events
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_inner_size(winit::dpi::LogicalSize::new(config.width, config.height))
                        .with_title(&config.title),
                )
                .context("Error creating window")?,
        );

        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .context("Error creating window")?;

        let instance = create_instance();
        let surface = instance
            .create_surface(window.clone())
            .context("Could not create surface")?;
        let (device, adapter, queue) = pollster::block_on(create_device(instance, &surface))
            .context("Could not create WGPU device")?;
        let (width, height) = (window.inner_size().width, window.inner_size().height);
        let configuration = surface
            .get_default_config(&adapter, width, height)
            .ok_or(anyhow!("Could not get default surface configuration"))?;
        surface.configure(&device, &configuration);

        Ok(Window {
            device,
            queue,
            surface,
            texture_format: configuration.format,
            window,
            event_loop: Some(events),
            width,
            height,
        })
    }

    fn debug_name() -> &'static str {
        "window"
    }
}

fn create_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor::default())
}

async fn create_device(
    instance: wgpu::Instance,
    surface: &wgpu::Surface<'static>,
) -> Result<(wgpu::Device, wgpu::Adapter, wgpu::Queue)> {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        )
        .await
        .unwrap();

    Ok((device, adapter, queue))
}
