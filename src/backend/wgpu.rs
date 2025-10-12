use std::sync::Arc;

use glyphon::cosmic_text;
use macroquad::conf;
use ::wgpu;
use wgpu::util::DeviceExt;

use crate::backend;
use backend::*;
use backend::Backend;

static HEX_VERTICES: [f32;1] = [0.1];
static HEX_INDICES : [usize;1] = [1];

pub struct Wgpu {
    // window must be 'static for Surface
    window: &'static winit::window::Window,
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
    textures: Vec<wgpu::Texture>,
    font_system: cosmic_text::FontSystem,
}

struct TileRenderer {
    hex_vertex_buffer: wgpu::Buffer,
    hex_index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
}

struct OverlayRenderer();
struct UiRenderer();

pub trait RenderLayer {
    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self
    where
        Self: Sized;

    fn render<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        queue: &wgpu::Queue,
        time: f32,
    );
}

impl RenderLayer for TileRenderer {
    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        // create vertex/index buffers, pipeline, etc.
        Self {
            hex_vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Hex vertex buffer"),
                contents: bytemuck::cast_slice(&HEX_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            hex_index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Hex index buffer"),
                contents: bytemuck::cast_slice(&HEX_INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }),
            instance_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance buffer"),
                size: 0, // placeholder
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    fn render<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        _queue: &wgpu::Queue,
        _time: f32,
    ) {
        pass.set_vertex_buffer(0, self.hex_vertex_buffer.slice(..));
        pass.set_index_buffer(self.hex_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        // ... draw something here
    }
}

impl RenderLayer for OverlayRenderer {
    fn new(_: &wgpu::Device, _: &wgpu::SurfaceConfiguration) -> Self {
        Self()
    }

    fn render<'a>(&'a self, _pass: &mut wgpu::RenderPass<'a>, _: &wgpu::Queue, _: f32) {}
}

impl RenderLayer for UiRenderer {
    fn new(_: &wgpu::Device, _: &wgpu::SurfaceConfiguration) -> Self {
        Self()
    }

    fn render<'a>(&'a self, _pass: &mut wgpu::RenderPass<'a>, _: &wgpu::Queue, _: f32) {}
}

struct Renderer {
    tile: TileRenderer,
    overlay: OverlayRenderer,
    ui: UiRenderer,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            tile: TileRenderer::new(device, config),
            overlay: OverlayRenderer::new(device, config),
            ui: UiRenderer::new(device, config),
        }
    }

    fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let time = 0.0; // placeholder for now
        self.tile.render(&mut pass, queue, time);
        self.overlay.render(&mut pass, queue, time);
        self.ui.render(&mut pass, queue, time);
    }
}

// impl<'a> Wgpu<'a> {
//     pub async fn new(window: &winit::window::Window) -> Self {
//         // boilerplate setup...
//         let surface = unsafe { instance.create_surface(window) }.unwrap();
//         let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.unwrap();
//         let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();
//         todo!()
//     }

//     pub fn load_texture(&mut self, bytes: &[u8]) -> TextureId {
//         // Simplify: real code uploads to GPU
//         let id = self.textures.len() as u32;
//         let tex = self.device.create_texture(&wgpu::TextureDescriptor {
//             label: Some("texture"),
//             size: wgpu::Extent3d { width: 64, height: 64, depth_or_array_layers: 1 },
//             mip_level_count: 1,
//             sample_count: 1,
//             dimension: wgpu::TextureDimension::D2,
//             format: wgpu::TextureFormat::Rgba8UnormSrgb,
//             usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
//             view_formats: &[],
//         });
//         self.textures.push(tex);
//         TextureId(id)
//     }

//     pub fn load_font(&mut self, bytes: &[u8]) -> FontId {
//         // Cosmic text doesn’t really give you a "font handle".
//         // Just pretend FontId=0 for now, manage multiple in Vec if needed.
//         FontId(0)
//     }
// }

impl Wgpu {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    fn get_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.inner_size()
    }
    fn render_frame(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());

        // This is where you’ll call your Renderer
        self.renderer
            .render(&self.device, &self.queue, &mut encoder, &view);

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }
}

impl Backend for Wgpu {
    fn render_ui<'b>(&mut self, messages: &mut Vec<ui::Message>, view: ui::View<'b>) {
        todo!()
    }

    fn set_buffer(&mut self, data: Vec<u8>) {
        todo!()
    }

    fn take_buffer(&self) -> Option<Vec<u8>> {
        todo!()
    }

    fn new(init_layout: Layout<f32>) -> Self {
        let textures = vec![];
        let font_system = cosmic_text::FontSystem::new();

        // 1. Create window (via winit)
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        let raw_window = winit::window::WindowBuilder::new()
            .with_title("SHE")
            .build(&event_loop)
            .unwrap();
        // 1.2 Wrap it in ManuallyDrop
        let window: &'static winit::window::Window = {
            let boxed = Box::new(raw_window);
            Box::leak(boxed) // leaks the Box, returning &'static
        };

        // 2. Create WGPU instance + surface
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(window) }.unwrap();

        // 3. Pick adapter + create device/queue
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        })).expect("No suitable GPU adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor::default(),
            None,
        )).unwrap();

        // 4. Configure surface (swapchain)
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        // 5. Initialize Renderer (TileRenderer + Overlay + UI)
        let renderer = Renderer::new(&device, &config);

        Self { surface, adapter, device, queue, config, renderer, window, instance, textures, font_system }
    }

    fn run_loop<F>(mut self, mut f: F)
    where
        F: 'static + FnMut(&mut Self, f32) -> bool,
    {
        use winit::event::{Event, WindowEvent};
        use winit::event_loop::EventLoop;

        let event_loop = EventLoop::new().unwrap();
        let window_id = self.window.id();
        let mut last_render_time = std::time::Instant::now();

        event_loop
            .run(move |event, elwt| {
                match event {
                    Event::WindowEvent {
                        ref event,
                        window_id: event_window_id,
                    } if event_window_id == window_id => {
                        match event {
                            WindowEvent::CloseRequested => {
                                elwt.exit();
                            }
                            WindowEvent::Resized(physical_size) => {
                                self.resize(*physical_size);
                            }
                            WindowEvent::RedrawRequested => {
                                let now = std::time::Instant::now();
                                let dt = now.duration_since(last_render_time).as_secs_f32();
                                last_render_time = now;

                                // user-provided callback (update function)

                                if f(&mut self, dt) {
                                    elwt.exit();
                                    return
                                };

                                match self.render_frame() {
                                    Ok(_) => {}
                                    Err(wgpu::SurfaceError::Lost) => self.resize(self.get_size()),
                                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                    Err(e) => eprintln!("Surface error: {:?}", e),
                                }
                            }
                            _ => {}
                        }
                    }
                    Event::AboutToWait => {
                        // Trigger continuous rendering
                        self.window.request_redraw();
                    }
                    _ => {}
                }
            })
            .unwrap();
    }

    fn poll_inputs(&self, layout: &mut Layout<f32>) -> Message {
        todo!()
    }

    fn poll_click_inputs(&self, layout: &mut Layout<f32>) -> Option<crate::cubic::Cube<i32>> {
        todo!()
    }

    fn poll_right_click_inputs(&self, layout: &mut Layout<f32>) -> Option<crate::cubic::Cube<i32>> {
        todo!()
    }

    fn get_frame_time() -> f32 {
        todo!()
    }

    async fn next_frame() {
        todo!()
    }

    fn clear(color: Color) {
        todo!()
    }

    fn draw_base_tiles(&self, view: &World, layout: &Layout<f32>, time: f32) {
        todo!()
    }

    fn draw_game_tiles(&self, view: &World, layout: &Layout<f32>) {
        todo!()
    }

    fn draw_army_legal_moves(game: &Game, layout: &Layout<f32>) {
        todo!()
    }

    fn draw_army_can_move_indicator(game: &Game, layout: &Layout<f32>) {
        todo!()
    }

    fn draw_army_info(world: &World, layout: &Layout<f32>) {
        todo!()
    }

    fn draw_fps_counter(x: f32, y: f32, font_size: f32, color: Color) {
        todo!()
    }

    fn draw_map_control_summary(game: &Game) {
        todo!()
    }

    fn draw_river(segment: &CubeSide, layout: &Layout<f32>) {
        todo!()
    }

    fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
        todo!()
    }

    async fn get_map_thumbnail(&self, world: &crate::World, width: f32, height: f32, resources: &GameResources) -> Vec<u8> {
        todo!()
    }
}
