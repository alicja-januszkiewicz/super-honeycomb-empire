use glyphon::cosmic_text;
use ::wgpu;

use crate::backend;
use backend::*;

pub struct WgpuBackend<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'a>,
    textures: Vec<wgpu::Texture>,
    font_system: cosmic_text::FontSystem,
}

impl<'a> WgpuBackend<'a> {
    pub async fn new(window: &winit::window::Window) -> Self {
        // boilerplate setup...
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(window) }.unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.unwrap();
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();

        Self {
            device,
            queue,
            surface,
            textures: vec![],
            font_system: cosmic_text::FontSystem::new(),
        }
    }

    pub fn load_texture(&mut self, bytes: &[u8]) -> TextureId {
        // Simplify: real code uploads to GPU
        let id = self.textures.len() as u32;
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: wgpu::Extent3d { width: 64, height: 64, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.textures.push(tex);
        TextureId(id)
    }

    pub fn load_font(&mut self, bytes: &[u8]) -> FontId {
        // Cosmic text doesn’t really give you a "font handle".
        // Just pretend FontId=0 for now, manage multiple in Vec if needed.
        FontId(0)
    }
}

impl Backend for WgpuBackend {
    fn begin_frame(&mut self, viewport: [u32; 2]) {
        // acquire swapchain frame, begin encoder
    }

    fn end_frame(&mut self) {
        // submit commands, present frame
    }

    fn draw_hex(&mut self, pos: [f32; 2], size: f32, color: Color) {
        // push vertices into a buffer for instanced hex rendering
    }

    fn draw_texture(&mut self, texture: TextureId, pos: [f32; 2], size: [f32; 2], tint: Color) {
        // encode draw with bind group for texture
    }

    fn draw_text(&mut self, text: &str, pos: [f32; 2], size: f32, color: Color) {
        // use cosmic-text shaping, then render glyphs with wgpu
    }
}
