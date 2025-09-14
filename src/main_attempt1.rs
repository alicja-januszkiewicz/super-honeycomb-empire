#![feature(trait_alias)]
#![allow(warnings)]

mod cubic;
mod game;
mod world;
mod ai;
// mod pixels;
mod mquad;
mod inputs;
mod map_editor;
mod river;
mod shapefiles;
mod fog;
mod rules;
mod network;
mod cli;
mod ui;

use clap::Parser;
use fog::*;
use ai::*;
use cli::*;
use cubic::*;
use game::*;
//use miniquad::{gl::glShaderSource, native::linux_x11::libx11::VisibilityChangeMask, UniformDesc};
use miniquad::UniformDesc;
use rules::Ruleset;
use ui::{main_menu};
use world::*;
use inputs::*;
use map_editor::*;
use network::*;
use std::{collections::HashMap, f32::consts::PI, fs::File};
// use crate::pixels::*;
use mquad::*;
// use macroquad::{file::load_file, miniquad::fs::load_file, prelude::*};
use macroquad::{file::load_file, prelude::*};
use dbase;

const WATER_FRAGMENT_SHADER: &'static str = include_str!("../assets/water_fragment_shader.glsl");
const WATER_VERTEX_SHADER: &'static str = include_str!("../assets/water_vertex_shader.glsl");
const FONT: &[u8] = include_bytes!("../assets/Iceberg-Regular.ttf");

fn load_rivers(shape: &Vec<(f32, f32)>) -> Vec<(usize, f32, f32)> {
    let x_min = shape.iter().fold(f32::NAN, |a, &b| a.min(b.0));
    let y_min = shape.iter().fold(f32::NAN, |a, &b| a.min(b.1));
    // shape = shape.iter().map(|(x, y)| (x - x_min, y - y_min)).collect();

    let mut river: Vec<(usize, f32, f32)> = Vec::new();
    let file = File::open("assets/shapes/ua-rivers.csv").unwrap();
    let mut rdr = csv::Reader::from_reader(file);
    // Iterate over each record in the CSV and parse the values
    for (idx, result) in rdr.records().enumerate() {
        let record = result.unwrap();
        let first_value: f32 = record.get(23).unwrap().parse().unwrap();
        let second_value: f32 = record.get(24).unwrap().parse().unwrap();
        let id: i32 = record.get(0).unwrap().parse().unwrap();
        let id: usize = id.abs() as usize;
        // match record.get(12).unwrap().parse() {
        //     Ok(_) => {},
        //     Err(e) => print!("{:}", e),
        // }
        // let vertex_part: i32 = record.get(12).unwrap().parse().unwrap();
        // let vertex_part_ring: i32 = record.get(13).unwrap().parse().unwrap();



        let r = 6371000.0 / 750.; //1:250 is nearly max
        let y = r * ((std::f32::consts::PI/4.) + (second_value.to_radians()/2.)).tan().ln();
        let x = r * first_value.to_radians();
        let val = (id, x - x_min, (y * -1.) - y_min);
        // if id == 4029011 {river.push((id, x, y * -1.));}
        if id == 25582 {river.push(val);}
        // river.push(val);



        // if vertex_part == 158 && vertex_part_ring == 0 {
        //     // shape.push((first_value * r, second_value*(-1.) * r));
        //     shape.push((x, y * -1.));
        // }
        //  if idx > 50000 {break}
    }
    river
}

async fn load_assets() -> Assets {
    // let mut reader = dbase::Reader::from_path("assets/ua_shp/ukr_admbnda_adm0_sspe_20230201.dbf").unwrap();
    // let f = File::open("assets/cities.json").expect("file should open read only");
    let f = include_bytes!("../assets/cities.json");
    let json: serde_json::Value = serde_json::from_reader(&f[..]).expect("file should be proper JSON");
    let locality_names: Vec<_> = json["data"].as_array().unwrap().iter().map(|el| el["asciiname"].to_string().replace("\"", "")).collect();
    // let locality_names = locality_names_v.iter().map(String::as_str).collect();
    // let locality_names: Vec<&str> = locality_names_v.iter().map(|s| &**s).collect();

    let font = load_ttf_font_from_bytes(FONT).unwrap();
    // let font = load_ttf_font("assets/Iceberg-Regular.ttf").await.unwrap();
    // let army = Texture2D::from_file_with_format(
    //     include_bytes!("../assets/army.png"),
    //     None,
    // );
    // let army: Texture2D = load_texture("assets/army.png").await.unwrap();
    let army_f = macroquad::prelude::load_file("army.png").await.unwrap();
    let army = Texture2D::from_file_with_format(&army_f, None);

    // let port: Texture2D = load_texture("assets/port.png").await.unwrap();
    let port = Texture2D::from_file_with_format(
        include_bytes!("../assets/port.png"),
        None,
    );

    let airport = Texture2D::from_file_with_format(
        include_bytes!("../assets/airport.png"),
        None,
    );
    // let airport: Texture2D = load_texture("assets/airport.png").await.unwrap();

    let fields = Texture2D::from_file_with_format(
        include_bytes!("../assets/grass.png"),
        None,
    );
    // let fields = load_texture("assets/grass.png").await.expect("Failed to load texture");

    let water_shader = crate::miniquad::ShaderSource::Glsl{
        fragment: WATER_FRAGMENT_SHADER,
        vertex: WATER_VERTEX_SHADER,
    };

    let water_material = load_material(
        water_shader,
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("Time", UniformType::Float1),
                UniformDesc::new("RectSize", UniformType::Float2),
            ],
            ..Default::default()
        },
    ).unwrap();
    let size = [32.,32.];
    let origin = [0., 0.];
    let init_layout = cubic::Layout{orientation: cubic::OrientationKind::Flat(cubic::FLAT), size, origin};
    
    water_material.set_uniform("RectSize", (init_layout.size[0], init_layout.size[1]));
    //let shape = vec!((300.,10.), (1000., 100.), (1000., 500.), (5000., 500.), (5000., 100.), (300., 10.));
    //let v: serde_json::Value = serde_json::from_str(data).unwrap();
    // let shape: Vec<(f32, f32)> = serde_json::from_str(data).unwrap();
    // Open the CSV file
    // std::fs::create_dir_all("assets/shapes");
    let vertices = shapefiles::extract_vertices("assets/ua_shp/ukr_admbnda_adm0_sspe_20230201.shp").unwrap();
    // let file = File::open("assets/shapes/ua-100k_v2.csv").unwrap();
    // let mut rdr = csv::Reader::from_reader(file);
    // let file = load_file(path);

    // Create a Vec<(f32, f32)> to store the data
    let mut shape: Vec<(f32, f32)> = Vec::new();

    // Iterate over each record in the CSV and parse the values
    for idx in 0..vertices.0.len() {
        let first_value = vertices.0.get(idx).unwrap();//row.get(0).unwrap();
        let second_value = vertices.1.get(idx).unwrap(); //row.get(1).unwrap();
        let vertex_part = vertices.2.get(idx).unwrap(); //row.get(2).unwrap().round() as i32;
        // when using qgis-derived file
        // let vertex_part: i32 = record.get(12).unwrap().parse().unwrap();
        // let vertex_part_ring: i32 = record.get(13).unwrap().parse().unwrap();

        let r = 6371000.0 / 750.; //1:250 is nearly max
        let y = r * ((std::f32::consts::PI/4.) + (second_value.to_radians()/2.)).tan().ln();
        let x = r * first_value.to_radians();
        
        if *vertex_part == 158 {//&& vertex_part_ring == 0 { // include rhs when using qgis-derived file
            // shape.push((first_value * r, second_value*(-1.) * r));
            shape.push((x, y * -1.));
        }
        //  if idx > 50000 {break}
    }

    // let river = load_rivers(&shape);
    let river = vec![];

    //println!("{:?}", shape);
    //let shape = vec!((0.,0.), (500., -950.), (1000., 0.), (1000.,-1000.), (500., -950.), (0.,-1000.));
    // let shape = vec!((0.,0.), (1000., 0.), (1000.,-1000.), (0.,-1000.));
    // let (min_x, min_y) = shape.iter().fold(0., |init: f32, (x, y)| (init.min(x), init.min(y)));
    // let min_x = shape.iter().fold(0., |init: f32, (x, y)| init.min(*x));
    // let min_y = shape.iter().fold(0., |init: f32, (x, y)| init.min(*y));

    Assets{locality_names, font, army, port, airport, fields, water_material, init_layout, shape, river}
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Super Honeycomb Empire".to_owned(),
        fullscreen: false,
        ..Default::default()
    }
}

fn new_game(rules: Ruleset, assets: &mut Assets) -> Game {
    let ai1 = AI{scores: DEFAULT_SCORES};
    let ai2 = AI{scores: DEFAULT_SCORES};
    let ai3 = AI{scores: DEFAULT_SCORES};
    let ai4 = AI{scores: DEFAULT_SCORES};

    // let player1 = Player::new("Redosia", Some(ai1));
    let player1 = Player::new("Redosia", Controller::Human);
    // let player2 = Player::new("Bluekraine", Some(ai2)); // Umberaine?
    let player2 = Player::new("Bluegaria", Controller::AI(ai2));
    let player3 = Player::new("Greenland", Controller::AI(ai3));
    // let player4 = Player::new("Violetnam", Some(ai4));

    let players: Vec<Player> = vec![player1, player2, player3];
    // let players: Vec<Player> = Vec::new();

    // let players = vec![player1, player2, player3, player4];
    // let players = vec![player1, player2, ];//player3, player4];

    let world = World::new();
    // // save_map(&game.world.world);
    // // let mut world = World::from_json("assets/maps/map.json");
    // // let mut world = World::from_json("assets/saves/quicksave.json");

    Game::new(players, world, rules, assets)
}

// async fn game_loop(game: &mut Game, layout: &mut Layout<f32>, assets: &Assets) {
//     let mut is_yet_won = false;

//     let mut time = 0.0;

//     while !is_yet_won {
//         clear_background(DARKGRAY);

//         poll_inputs(game, layout);

//         if is_key_pressed(KeyCode::F1) {
//             break
//         }

//         draw(&game, &layout, &assets, time);
    
//         game.update();

//         is_yet_won = game.rules.victory_condition.check(&game.world, game.current_player_index());

//         next_frame().await;
//         time += get_frame_time();
//     }
//     println!("Player {} won!", game.current_player_index());
// }

// struct App<T: Component>(T);

// fn get_app<T: Component>(assets: &mut Assets) -> dyn Component {
//     new_game(assets)
// }

use std::time::Instant;
use std::sync::Arc;
use wgpu::{self, SurfaceConfiguration};
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window,
};

use iced::{
    widget::Tree,
    Renderer,
    core::renderer::settings::Settings as IcedSettings,
};

enum Mode {
    Menu(crate::ui::Ui),
    Game(Box<dyn Component>),
}


fn main() {
    pollster::block_on(run());
}

async fn run() {

    // after you have `device`, `queue`, `surface_format`
    let iced_settings = IcedSettings {
        default_font: None,
        ..Default::default()
    };
    let mut debug_overlay = iced::widget::overlay::Debugger::new();
    let mut iced_renderer = iced_wgpu::Renderer::new(
        &device,
        iced_settings,
        surface_format,
        None,          // no clipboard yet
    );
    let mut iced_runtime = iced::runtime::ProgramExecutor::<Message, Renderer>::new(
        MenuProgram::new(/* assets etc. */),
        iced::executor::Default::new(),
    );

    // ── the old “pre‑loop” section ────────────────────────────────────────
    set_pc_assets_folder("assets");
    let mut assets = load_assets().await;
    let (mut exit, ui) = main_menu(&mut assets).await;
    if exit {
        return;
    }
    // everything below wants Arc<Window>, so hold the window that way
    let window = {
        // EventLoop + window
        let event_loop = EventLoop::new().unwrap();
        #[allow(deprecated)]
        let w: Window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        let window = Arc::new(w);

        // wgpu init needs &Window
        let (surface, device, queue, config) = init_wgpu(&window).await;

        // kick off main loop
        game_loop(
            event_loop,
            window.clone(),
            surface,
            device,
            queue,
            config,
            App::from_ui(ui, &mut assets),
            assets,
            exit,
        );
        // game_loop is ! (never returns), so this line is unreachable,
        // but Rust still wants a value
        window
    };
    // unreachable
    let _ = window;
}

// the “poll → draw → update” rhythm lives here
#[allow(clippy::too_many_arguments)]
fn game_loop(
    event_loop: EventLoop<()>,
    window: Arc<Window>,
    mut surface: wgpu::Surface<'_>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    mut config: SurfaceConfiguration,
    mut app: App,
    mut assets: Assets,
    mut exit: bool,
) -> () {
    let mut layout = assets.init_layout.clone();
    let mut last_frame = Instant::now();
    let mut time = 0.0_f32;

    event_loop
        .run(move |event, elwt| {
            // default to polling each cycle
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                // 1️⃣ input & window events
                Event::WindowEvent { window_id, event }
                    if window_id == window.id() =>
                {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),

                        WindowEvent::Resized(size) => {
                            config.width = size.width;
                            config.height = size.height;
                            surface.configure(&device, &config);
                        }

                        WindowEvent::KeyboardInput { event, .. } => {
                            if event.state == ElementState::Pressed
                                && event.logical_key == Key::Named(NamedKey::F1)
                            {
                                //  ⚠️ re‑enable with an interior‑mutable wrapper
                                // app = app.swap();
                            }
                            exit |= app.poll(&mut layout);
                        }

                        WindowEvent::CursorMoved { .. }
                        | WindowEvent::MouseInput { .. }
                        | WindowEvent::MouseWheel { .. } => {
                            exit |= app.poll(&mut layout);
                        }

                        // 2️⃣ redraw
                        WindowEvent::RedrawRequested => {
                            if let Ok(frame) = surface.get_current_texture() {
                                let view = frame.texture.create_view(&Default::default());

                                let mut encoder = device.create_command_encoder(
                                    &wgpu::CommandEncoderDescriptor {
                                        label: Some("clear‑pass"),
                                    },
                                );

                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("clear"),
                                    color_attachments: &[Some(
                                        wgpu::RenderPassColorAttachment {
                                            view: &view,
                                            resolve_target: None,
                                            ops: wgpu::Operations {
                                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                                    r: 0.1,
                                                    g: 0.1,
                                                    b: 0.1,
                                                    a: 1.0,
                                                }),
                                                store: wgpu::StoreOp::Store,
                                            },
                                        },
                                    )],
                                    depth_stencil_attachment: None,
                                    occlusion_query_set: None,
                                    timestamp_writes: None,
                                });
                                // later: app.draw(&mut layout,
                                //                 &mut assets,
                                //                 time,
                                //                 &device,
                                //                 &queue,
                                //                 &view);

                                queue.submit(Some(encoder.finish()));
                                frame.present();
                            }
                        }

                        _ => {}
                    }
                }

                // 3️⃣ once every loop iteration
                Event::AboutToWait => {
                    let now = Instant::now();
                    let dt = (now - last_frame).as_secs_f32();
                    last_frame = now;

                    app.update();
                    time += dt;

                    window.request_redraw();
                }

                _ => {}
            }

            if exit {
                elwt.exit();
            }
        })
        .unwrap();
}

// ── GPU bootstrap ───────────────────────────────────────────────────────────
async fn init_wgpu(
    window: &Window,
) -> (
    wgpu::Surface<'_>,
    wgpu::Device,
    wgpu::Queue,
    SurfaceConfiguration,
) {
    let size = window.inner_size();
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(window).unwrap();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .unwrap();

    let config = SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_capabilities(&adapter).formats[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    (surface, device, queue, config)
}

// ── STUBS SO THIS FILE COMPILES — REPLACE WITH REAL STUFF ───────────────────
// fn set_pc_assets_folder(_: &str) {}
// struct Ui;
// #[derive(Clone)] struct Layout;
// struct Assets { init_layout: Layout }
// async fn load_assets() -> Assets { Assets { init_layout: Layout } }
// // async fn main_menu(_: &mut Assets) -> (bool, Ui) { (false, Ui) }

// struct App;
// impl App {
//     fn from_ui(_: Ui, _: &mut Assets) -> Self { Self }
//     fn poll(&mut self, _: &mut Layout) -> bool { false }
//     fn update(&mut self) {}
//     #[allow(clippy::too_many_arguments)]
//     fn draw(
//         &mut self,
//         _: &mut Layout,
//         _: &mut Assets,
//         _: f32,
//         _: &wgpu::Device,
//         _: &wgpu::Queue,
//         _: &wgpu::TextureView,
//     ) { }
//     #[allow(dead_code)]
//     fn swap(self) -> Self { self }
// }

// let size = [0.1,0.1]; // use this if in local coords
// let camera = &Camera2D {
//     // zoom: vec2(0.001, 0.001),
//     // offset: vec2(-0.5,-0.1),
//     zoom: vec2(1., -1.),
//     ..Default::default()
// };

// // set_camera(camera);