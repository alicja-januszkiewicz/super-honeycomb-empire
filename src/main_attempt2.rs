//! Minimal wgpu + winit replacement for the old `macroquad::main` entry‑point.
//! ------------------------------------------------------------------------
//! This keeps the same high‑level structure (input → update → draw
//! in a perpetual loop) but drives it with **winit** events and renders through a
//! proper *wgpu* render‑pass.  All Macroquad‑specific calls will gradually be
//! swapped out for own routines built on top of this skeleton.
//!
//! Build‑tested against **wgpu = "25"** and **winit = "0.30"**.

#![allow(dead_code)]

use std::iter;

// use env_logger;
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

/// Top‑level GPU/Window state.  Replace Macroquad globals.
struct State {
    surface: wgpu::Surface,      // Swap‑chain surface bound to the window
    device: wgpu::Device,        // Logical GPU
    queue: wgpu::Queue,          // Submission queue
    config: wgpu::SurfaceConfiguration, // Current swap‑chain format/settings
    size:  PhysicalSize<u32>,    // Window size (so we can recreate the surface on resize)

    // ────────────────────────────────────────────────────────────────────────────
    // TODO: Port the textures, pipelines, bind‑groups, buffers, etc. here.
    // ────────────────────────────────────────────────────────────────────────────
}

impl State {
    /// Create `State` **asynchronously** (wgpu has async initialisation).
    async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        // 1. Instance → Surface
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(window) }.expect("Failed to create surface");

        // 2. Adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No suitable GPU adapters found on the system!");

        // 3. Logical device + queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    features: wgpu::Features::default(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .expect("Failed to create device");

        // 4. Swap‑chain configuration
        let caps   = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width:        size.width,
            height:       size.height,
            present_mode: caps.present_modes[0],
            alpha_mode:   caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        Self { surface, device, queue, config, size }
    }

    /// Handle window resize from winit.
    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width  = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Per‑frame input handling.  Return *true* if handled so caller skips default processing.
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            // Example: replace `is_key_pressed(KeyCode::F1)` logic
            WindowEvent::KeyboardInput { input, .. } => {
                if let (&ElementState::Pressed, Some(VirtualKeyCode::F1)) = (input.state, input.virtual_keycode) {
                    // TODO: swap game/editor etc.
                    println!("F1 pressed – swap mode");
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// Update game state (was `app.update()` in Macroquad).
    fn update(&mut self) {
        // TODO: hook the simulation step here
    }

    /// Draw a frame via wgpu.  Equivalent to `app.draw()` + `next_frame().await`.
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view   = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view:           &view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: true },
                })],
                depth_stencil_attachment: None,
            });

            // TODO: issue the draw calls here (pipelines, vertex buffers, etc.)
        }

        // Submit and present
        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

fn main() {
    // Macroquad initialised logging for you; do the same here.
    env_logger::init();

    // 0. Window & event‑loop (replaces `window_conf` + `macroquad::main` attribute)
    let event_loop = EventLoop::new();
    let window     = WindowBuilder::new()
        .with_title("Super Honeycomb Empire")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .build(&event_loop)
        .expect("Failed to create window");

    // 1. GPU setup – pollster blocks on the async `State::new`.
    let mut state = pollster::block_on(State::new(&window));

    // 2. Drive the game
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll; // continuous like Macroquad

        match event {
            Event::WindowEvent { ref event, window_id } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => state.resize(*physical_size),
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => state.resize(**new_inner_size),
                        WindowEvent::KeyboardInput { input: KeyboardInput { state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::Escape), .. }, .. } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Recreate the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // This is fatal
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{e:?}"),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we *actively* request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}
