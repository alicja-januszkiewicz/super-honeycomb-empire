use csv::Position;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    window::WindowId,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Window,
    keyboard::Key
};

use ::wgpu;
use wgpu::util::DeviceExt;

use cgmath::{Matrix4, Point3, Vector3, perspective, Rad};
use std::mem;

use crate::{backend, cubic::Cube};
use backend::*;
use backend::Backend;

static HEX_VERTICES: [f32; 6] = [
    0.0,  0.5,  // top
   -0.5, -0.5,  // bottom-left
    0.5, -0.5,  // bottom-right
];
static HEX_INDICES : [usize;1] = [1];

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: [f32;2],
    color: [f32;4],
}

pub struct Wgpu {
    layout: Layout<f32>,
    window: Option<&'static Window>,
    renderer: Option<Renderer>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    last_frame_time: std::time::Instant,
    frame_callback: Option<Box<dyn FnMut(&mut Self, f32) -> bool>>,
}

impl Backend for Wgpu {
    fn new(_layout: Layout<f32>) -> Self {
        Self {
            layout: _layout,
            window: None,
            renderer: None,
            surface: None,
            device: None,
            queue: None,
            config: None,
            last_frame_time: std::time::Instant::now(),
            frame_callback: None,
        }
    }

    fn run_loop<F>(mut self, f: F)
    where
        F: 'static + FnMut(&mut Self, f32) -> bool,
    {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        self.frame_callback = Some(Box::new(f));
        event_loop.run_app(&mut self).unwrap();
    }
    
    fn render_ui<'a>(&mut self, messages: &mut Vec<ui::Message>, view: ui::View<'a>) {
        //todo!()
    }
    
    fn set_buffer(&mut self, data: Vec<u8>) {
        //todo!()
    }
    
    fn take_buffer(&self) -> Option<Vec<u8>> {
        todo!()
    }
    
    fn poll_inputs(&self, layout: &mut Layout<f32>) -> Message {
        *layout = self.layout;
        Message::Tick//todo!()
    }
    
    fn poll_click_inputs(&self, layout: &mut Layout<f32>) -> Option<crate::cubic::Cube<i32>> {
        None//todo!()
    }
    
    fn poll_right_click_inputs(&self, layout: &mut Layout<f32>) -> Option<crate::cubic::Cube<i32>> {
        None//todo!()
    }
    
    fn get_frame_time() -> f32 {
        0.0//todo!()
    }
    
    async fn next_frame() {
        //todo!()
    }
    
    fn clear(color: Color) {
        //todo!()
    }
    
    fn draw_base_tiles(&mut self, world: &World, layout: &Layout<f32>, time: f32) {
        // self.layout = layout.clone();
        let device = self.device.as_ref().unwrap();
        let queue = self.queue.as_ref().unwrap();

        let instances: Vec<Instance> = world.iter().map(|(cube, tile)| {
            let pixel = Cube::<f32>::from(*cube).to_pixel(layout);
            let position = [pixel.0, pixel.1];
            let position = [cube.q() as f32, cube.r() as f32];
            // println!("{:?}", pixel);
            Instance {
                position: position,
                color: [0.2,0.7,1.0,1.0],//tile.category.color(), // implement a color() helper
            }
        }).collect();

        // let instances = vec![Instance {
        //     position: [0.0, 0.0],
        //     color: [0.2, 0.7, 1.0, 1.0],
        // }];

        let byte_size = (instances.len() * std::mem::size_of::<Instance>()) as u64;
        if byte_size > self.renderer.as_ref().unwrap().tile.instance_buffer.size() {
            self.renderer.as_mut().unwrap().tile.instance_buffer =
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Instance buffer"),
                    size: byte_size.next_power_of_two(),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
        }

        queue.write_buffer(
            &self.renderer.as_ref().unwrap().tile.instance_buffer,
            0,
            bytemuck::cast_slice(&instances),
        );

        self.renderer.as_mut().unwrap().tile.instance_count = instances.len() as u32;
        self.renderer.as_mut().unwrap().layout = *layout;
    }
    
    fn draw_game_tiles(&self, view: &World, layout: &Layout<f32>) {
        //todo!()
    }
    
    fn draw_army_legal_moves(game: &Game, layout: &Layout<f32>) {
        //todo!()
    }
    
    fn draw_army_can_move_indicator(game: &Game, layout: &Layout<f32>) {
        //todo!()
    }
    
    fn draw_army_info(world: &World, layout: &Layout<f32>) {
        //todo!()
    }
    
    fn draw_fps_counter(x: f32, y: f32, font_size: f32, color: Color) {
        //todo!()
    }
    
    fn draw_map_control_summary(game: &Game) {
        //todo!()
    }
    
    fn draw_river(segment: &CubeSide, layout: &Layout<f32>) {
        //todo!()
    }
    
    fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
        //todo!()
    }
    
    async fn get_map_thumbnail(&self, world: &crate::World, width: f32, height: f32, resources: &GameResources) -> Vec<u8> {
        todo!()
    }
}

impl ApplicationHandler for Wgpu {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let raw_window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        let window: &'static winit::window::Window = {
            let boxed = Box::new(raw_window);
            Box::leak(boxed) // leaks the Box, returning &'static
        };

        // --- GPU init ---
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(window) }.unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .unwrap();

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

        // --- create renderer ---
        let renderer = Renderer::new(&device, &config, self.layout);

        // store everything
        self.window = Some(window);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.renderer = Some(renderer);
        self.last_frame_time = std::time::Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) {
                    event_loop.exit();
                }

                if let Some(window) = self.window {
                    if event.state.is_pressed() {
                        match event.logical_key.as_ref() {
                            Key::Character("w") => self.layout.origin[1] += 0.1,
                            Key::Character("s") => self.layout.origin[1] -= 0.1,
                            Key::Character("a") => self.layout.origin[0] -= 0.1,
                            Key::Character("d") => self.layout.origin[0] += 0.1,
                            Key::Character("m") => {self.layout.size[0] += 0.05;
                                                    self.layout.size[1] += 0.05},
                            Key::Character("n") => {self.layout.size[0] -= 0.05;
                                                    self.layout.size[1] -= 0.05},
                            Key::Character("q") => self.layout.orientation.apply_mut(|o| o.start_angle -= 0.01),
                            Key::Character("e") => self.layout.orientation.apply_mut(|o| o.start_angle += 0.01),
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let dt = now.duration_since(self.last_frame_time).as_secs_f32();
                self.last_frame_time = now;

                if let Some(mut cb) = self.frame_callback.take() {
                    let exit = cb(self, dt);
                    self.frame_callback = Some(cb);
                    if exit {
                        event_loop.exit();
                        return;
                    }
                }

                if let (Some(renderer), Some(surface), Some(device), Some(queue), Some(config)) = (
                    &mut self.renderer,
                    &self.surface,
                    &self.device,
                    &self.queue,
                    &self.config,
                ) {
                    renderer.render_frame(&self.layout, surface, device, queue, config, dt);
                }

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {}
        }
    }
}

struct TileRenderer {
    hex_vertex_buffer: wgpu::Buffer,
    hex_index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    instance_count: u32,
}

struct OverlayRenderer();
struct UiRenderer();

pub trait RenderLayer {
    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, layout: &Layout<f32>) -> Self
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
    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, layout: &Layout<f32>) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("../../assets/shader_dummy.wgsl"));

        // let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("Uniform Buffer"),
        //     size: 64, // 4x4 matrix of f32 = 64 bytes
        //     usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        //     mapped_at_creation: false,
        // });

        // let uniform_bind_group_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         label: Some("Uniform Bind Group Layout"),
        //         entries: &[wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::VERTEX,
        //             ty: wgpu::BindingType::Buffer {
        //                 ty: wgpu::BufferBindingType::Uniform,
        //                 has_dynamic_offset: false,
        //                 min_binding_size: None,
        //             },
        //             count: None,
        //         }],
        //     });

        // let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("Uniform Bind Group"),
        //     layout: &uniform_bind_group_layout,
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: uniform_buffer.as_entire_binding(),
        //     }],
        // });

        const SQRT3: f32 = 1.732050807568877293527446341505872366942805253810380628055806; // sqrt(3)
        let orientation = OrientationUniform {
            f0: SQRT3,
            f1: SQRT3 / 2.0,
            f2: 0.0,
            f3: 3.0 / 2.0,
        };

        let orientation_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Orientation Buffer"),
            contents: bytemuck::cast_slice(&[orientation]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let orientation = match layout.orientation {
            crate::cubic::OrientationKind::Flat(o) => o,
            crate::cubic::OrientationKind::Pointy(o) => o,
        };

        let layout_uniform = LayoutUniform {
            hex_size: layout.size[0],
            _pad0: 0.0,
            origin: layout.origin,
            rotation_angle: orientation.start_angle,// * std::f32::consts::PI,
            _pad1: 0.0,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Layout Uniform Buffer"),
            contents: bytemuck::cast_slice(&[layout_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Layout Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
                wgpu::BindGroupLayoutEntry {  // <-- add this
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
        }
            ],
            
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: orientation_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("Layout Bind Group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Tile Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tile Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 2]>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            1 => Float32x2, // position
                            2 => Float32x4  // color
                        ],
                    }
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });


        let vertices: Vec<[f32; 2]> = {
            let mut v = vec![[0.0, 0.0]];
            for i in 0..6 {
                let a = std::f32::consts::PI / 3.0 * i as f32;
                v.push([0.5 * a.cos(), 0.5 * a.sin()]);
            }
            v
        };
        let hex_radius = layout.size[0]; // or whichever axis defines hex radius
        let start_angle = match layout.orientation {
            crate::OrientationKind::Pointy(_) => -std::f32::consts::PI / 6.0,
            crate::OrientationKind::Flat(_) => -std::f32::consts::PI / 6.0,
        };

        let vertices: Vec<[f32; 2]> = {
            let mut v = vec![[0.0, 0.0]];
            for i in 0..6 {
                let a = std::f32::consts::PI / 3.0 * i as f32 + start_angle;
                v.push([hex_radius * a.cos(), hex_radius * a.sin()]);
            }
            v
        };
        // let vertices: Vec<[f32; 2]> = {
        //     let mut v = vec![[0.0, 0.0]];
        //     for i in 0..6 {
        //         // Use same orientation as your layout (pointy-top = -π/6)
        //         let a = std::f32::consts::PI / 3.0 * i as f32 - std::f32::consts::PI / 6.0;
        //         v.push([hex_radius * a.cos(), hex_radius * a.sin()]);
        //     }
        //     v
        // };
        let indices: Vec<u16> = (0..6)
            .flat_map(|i| [0, i as u16 + 1, ((i + 1) % 6 + 1) as u16])
            .collect();

        let hex_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hex vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let hex_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hex index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            hex_vertex_buffer,
            hex_index_buffer,
            instance_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance buffer"),
                size: 0, // placeholder
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            uniform_buffer,
            uniform_bind_group,
            render_pipeline,
            instance_count: 0,
        }
    }

    fn render<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        _queue: &wgpu::Queue,
        _time: f32,
    ) {
        let index_count = (self.hex_index_buffer.size() / 2) as u32;
        pass.set_pipeline(&self.render_pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, self.hex_vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.set_index_buffer(self.hex_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..index_count, 0, 0..self.instance_count);
    }
}

impl RenderLayer for OverlayRenderer {
    fn new(_: &wgpu::Device, _: &wgpu::SurfaceConfiguration, layout: &Layout<f32>) -> Self {
        Self()
    }

    fn render<'a>(&'a self, _pass: &mut wgpu::RenderPass<'a>, _: &wgpu::Queue, _: f32) {}
}

impl RenderLayer for UiRenderer {
    fn new(_: &wgpu::Device, _: &wgpu::SurfaceConfiguration, layout: &Layout<f32>) -> Self {
        Self()
    }

    fn render<'a>(&'a self, _pass: &mut wgpu::RenderPass<'a>, _: &wgpu::Queue, _: f32) {}
}

struct Renderer {
    tile: TileRenderer,
    overlay: OverlayRenderer,
    ui: UiRenderer,
    layout: Layout<f32>,
}

impl Renderer {
    fn build_mvp(layout: &Layout<f32>, window_size: (u32, u32)) -> Matrix4<f32> {
        let aspect = window_size.0 as f32 / window_size.1 as f32;
        let fov = 45.0_f32.to_radians();

        // Camera setup
        let eye = Point3::new(layout.origin[0], layout.origin[1], 1.0);
        let target = Point3::new(layout.origin[0], layout.origin[1], 0.0);
        let up = Vector3::unit_y();

        let view = Matrix4::look_at_rh(eye, target, up);
        //let proj = perspective(Rad(fov), aspect, 0.1, 100.0);
        let w = window_size.0 as f32;
        let h = window_size.1 as f32;
        // let proj = cgmath::ortho(-w / 1000.0, w / 1000.0, -h / 1000.0, h / 1000.0, -1.0, 1.0);
        //let proj = cgmath::ortho(-2.0, 2.0, -2.0, 2.0, -1.0, 1.0);
        let proj = cgmath::ortho(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0);
        //proj * view
        proj


    }
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, layout: Layout<f32>) -> Self {
        Self {
            tile: TileRenderer::new(device, config, &layout),
            overlay: OverlayRenderer::new(device, config, &layout),
            ui: UiRenderer::new(device, config, &layout),
            layout,
        }
    }
    pub fn render_frame(
        &mut self,
        layout: &Layout<f32>,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        time: f32,
    ) {
        let frame = surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let size = (config.width, config.height);
        let mvp = Self::build_mvp(&layout, size);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Main Encoder"),
        });
        // queue.write_buffer(
        //     &self.tile.uniform_buffer,
        //     0,
        //     bytemuck::cast_slice(AsRef::<[f32; 16]>::as_ref(&mvp)),
        // );
        let orientation = match layout.orientation {
            crate::cubic::OrientationKind::Flat(o) => o,
            crate::cubic::OrientationKind::Pointy(o) => o,
        };
        let updated_layout = LayoutUniform {
            hex_size: layout.size[0],
            _pad0: 0.0,
            origin: layout.origin,
            rotation_angle: orientation.start_angle,
            _pad1: 0.0,
        };

        queue.write_buffer(
            &self.tile.uniform_buffer,
            0,
            bytemuck::cast_slice(&[updated_layout]),
        );

        let pass_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &pass_view,
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

            // Call each RenderLayer, giving it access to the active pass.
            self.tile.render(&mut pass, queue, time);
            self.overlay.render(&mut pass, queue, time);
            self.ui.render(&mut pass, queue, time);
        }

        queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct OrientationUniform {
    f0: f32,
    f1: f32,
    f2: f32,
    f3: f32,
}


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LayoutUniform {
    hex_size: f32,           // 4
    _pad0: f32,              // 4 → padding to align next vec2
    origin: [f32; 2],        // 8
    rotation_angle: f32,     // 4
    _pad1: f32,              // 4 → pad to 24 bytes
}