#![feature(trait_alias)]
#![allow(warnings)]
#![feature(const_mut_refs)]

use std::path::Path;

use glyphon::{FontSystem, SwashCache};
use winit::{
    event::*,
    event_loop::{EventLoop},
    window::{Window, WindowBuilder},
};
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};
use cgmath::SquareMatrix;

mod cubic;
use cubic::{Cube, Layout, OrientationKind, FLAT, Pixel};

mod world;
use world::{Army, World, Tile, TileCategory, Locality, LocalityCategory, Controller, Player};

mod ai;
use ai::{AI, DEFAULT_SCORES};

mod cli;
mod river;
mod map_editor;
mod fog;
mod network;
mod shapefiles;
mod inputs;
mod backend;

mod rules;
use rules::{Ruleset};

mod game;
use game::{Game, GameResources, VictoryCondition};

use network::{Client};

use crate::network::{App, Component};

// stubs for now

//pub fn poll_inputs(game: &mut Game, client: Option<&Client>, layout: &mut Layout<f32>) -> bool {true}

pub trait FontHandle {}
pub trait TextureHandle {}
pub trait MaterialHandle {}

pub struct Assets<F: FontHandle, T: TextureHandle, M: MaterialHandle> {
    pub font: F,
    pub army: T,
    pub port: T,
    pub airport: T,
    pub fields: T,
    pub water_material: M,
}

// pub trait AssetProvider {
//     type Font: FontHandle;
//     type Texture: TextureHandle;
//     type Material: MaterialHandle;

//     fn assets(&self) -> &Assets<Self::Font, Self::Texture, Self::Material>;
// }

// impl AssetProvider for WgpuAssets {
//     type Font = FontAsset;
// }

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }
}

fn load_resources() -> GameResources {
    // let mut reader = dbase::Reader::from_path("assets/ua_shp/ukr_admbnda_adm0_sspe_20230201.dbf").unwrap();
    // let f = File::open("assets/cities.json").expect("file should open read only");
    let f = include_bytes!("../assets/cities.json");
    let json: serde_json::Value = serde_json::from_reader(&f[..]).expect("file should be proper JSON");
    let locality_names: Vec<_> = json["data"].as_array().unwrap().iter().map(|el| el["asciiname"].to_string().replace("\"", "")).collect();
    // let locality_names = locality_names_v.iter().map(String::as_str).collect();
    // let locality_names: Vec<&str> = locality_names_v.iter().map(|s| &**s).collect();

    let size = [32.,32.];
    let origin = [0., 0.];
    let init_layout = cubic::Layout{orientation: cubic::OrientationKind::Flat(cubic::FLAT), size, origin};

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

    GameResources {locality_names, init_layout, shape, river}
}

macro_rules! font_asset {
    ($name:literal, $path:literal) => {
        FontAsset {
            name: $name,
            bytes: include_bytes!($path),
        }
    };
}

pub struct FontAsset {
    pub name: &'static str,
    pub bytes: &'static [u8], // or Vec<u8> if loading dynamically
}

// impl FontAsset {
//     fn new(path: &str) -> Self {
//         let name = Path::new(path).file_name().unwrap().to_owned().into_string().unwrap();
//         let bytes = include_bytes!(path.to_owned());
//         Self {name, bytes}
//     }
// }

pub trait FontProvider {
    type Font;
    fn get_font(&self, id: &str) -> &Self::Font;
}

pub trait TextureProvider {
    type Texture;
    fn get_texture(&self, id: &str) -> &Self::Texture;
}

pub trait MaterialProvider {
    type Material;
    fn get_material(&self, id: &str) -> &Self::Material;
}



// pub trait AssetProvider:
//     FontProvider + TextureProvider + MaterialProvider {}


impl FontProvider for WgpuAssets {
    type Font = FontSystem;
    fn get_font(&self, id: &str) -> &Self::Font {
        &self.font_system
    }
}

// impl<T> AssetProvider for T where
//     T: FontProvider + TextureProvider + MaterialProvider {}

fn load_assets() -> WgpuAssets {
    // let font = FontAsset::new("../assets/Iceberg-Regular.ttf")
    let font = font_asset!("Iceberg-Regular", "../assets/Iceberg-Regular.ttf");
    let mut font_system = FontSystem::new();
    // let data = include_bytes!("../assets/Iceberg-Regular.ttf");
    let face_id = font_system.db_mut().load_font_data(font.bytes.into());
    WgpuAssets{font_system}
}


impl FontHandle for FontAsset {}

struct WgpuAssets {
    font_system: FontSystem,
    // put wgpu pipeline handles, textures, fonts here
}


struct RealGameState<A> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    camera: Camera,
    
    //game: Game,
    app: Box<dyn Component<A>>,
    // layout: Layout<f32>,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl RealGameState {
    async fn new(window: &Window, surface: &wgpu::Surface<'_>, instance: &wgpu::Instance) -> Self {
        let size = window.inner_size();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let camera = Camera {
            eye: (0.0, 0.0, 25.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let mut resources = load_resources();
        let game = new_game(&mut resources);
        // let layout = Layout {
        //     orientation: OrientationKind::Flat(FLAT),
        //     size: [32.0, 32.0],
        //     origin: [0.0, 0.0],
        // };

        // Generate geometry from the actual game world
        let (vertices, indices) = create_world_geometry_from_game(&game, &resources.init_layout);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer: camera_buffer,
            uniform_bind_group: camera_bind_group,
            camera,
            game,
            // layout,
            vertices,
            indices,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.camera.aspect = self.config.width as f32 / self.config.height as f32;
        }
    }

    pub fn render(&mut self, surface: &wgpu::Surface) -> Result<(), wgpu::SurfaceError> {
        // Update camera uniform
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&self.camera);
        
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
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
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            
            // Draw all hexagons
            let num_indices = self.indices.len() as u32;
            render_pass.draw_indexed(0..num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn new_game(resources: &mut GameResources) -> Game {
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

    let victory_condition = VictoryCondition::Elimination;
    let rules = Ruleset::default(victory_condition, &players);

    let world = World::new();
    // // save_map(&game.world.world);
    // // let mut world = World::from_json("assets/maps/map.json");
    // // let mut world = World::from_json("assets/saves/quicksave.json");

    let mut game = Game::new(players, world, rules);
    Game::init_world(&mut game, resources);
    game
}

// fn create_real_game() -> Game {
//     // Create players using our simplified structures
//     let player1 = Player::new("Redosia", Controller::Human);
//     let player2 = Player::new("Bluegaria", Controller::AI("AI".to_string()));
//     let player3 = Player::new("Greenland", Controller::AI("AI".to_string()));
    
//     let players = vec![player1, player2, player3];
//     let world = create_test_world();
    
//     Game::new(players, world)
// }

// fn create_test_world() -> World {
//     let mut world = World::new();
    
//     // Create a hex grid of tiles
//     let radius = 5;
    
//     for q in -radius..=radius {
//         let r1 = (-radius).max(-q - radius);
//         let r2 = radius.min(-q + radius);
//         for r in r1..=r2 {
//             let cube = Cube::new(q,r);
            
//             // Determine tile type based on position
//             let category = if cube.q() == 0 && cube.r() == 0 {
//                 TileCategory::Farmland // Center
//             } else if i32::abs(cube.q()) + cube.r().abs() + cube.s().abs() == radius * 2 {
//                 TileCategory::Water // Edge
//             } else {
//                 TileCategory::Farmland // Interior
//             };
            
//             let mut tile = Tile::new(category);
            
//             // Add some localities
//             let capital_cube = Cube::new(0,0);
//             let port_cube = Cube::new(2,-2);
//             let airport_cube = Cube::new(-2,0);
            
//             if cube == capital_cube {
//                 tile.locality = Some(Locality::new("Capital City", LocalityCategory::Capital(0)));
//                 tile.owner_index = Some(0);
//                 tile.army = Some(Army::new(10, Some(0)));
//             } else if cube == port_cube {
//                 tile.locality = Some(Locality::new("Port Town", LocalityCategory::PortCity));
//                 tile.owner_index = Some(1);
//                 tile.army = Some(Army::new(5, Some(1)));
//             } else if cube == airport_cube {
//                 tile.locality = Some(Locality::new("Airport", LocalityCategory::Airport));
//                 tile.owner_index = Some(2);
//             }
            
//             // Add some armies to random tiles
//             if tile.owner_index.is_some() && tile.army.is_none() && (cube.q() + cube.r() + cube.s()) % 3 == 0 {
//                 tile.army = Some(Army::new(3, tile.owner_index));
//             }
            
//             world.insert(cube, tile);
//         }
//     }
    
//     world
// }

fn create_world_geometry_from_game(game: &Game, layout: &Layout<f32>) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let hex_radius = 0.6;
    let hex_width = hex_radius * 1.732; // sqrt(3)
    let hex_height = hex_radius * 2.0;
    
    for (cube, tile) in game.world.iter() {
        // Use the exact same coordinate system as the working hex grid
        let hex_width = hex_radius * 1.732; // sqrt(3)
        let hex_height = hex_radius * 2.0;
        
        // Convert cube coordinates to row/col for proper hex grid positioning
        // For cube coordinates, we need to convert to a grid system
        let row = cube.r() + 5; // Offset to make positive
        let col = cube.q() + 5; // Offset to make positive
        
        // Calculate hex position with proper offset for tiling (same as working hex grid)
        let offset_x = if row % 2 == 1 { hex_width * 0.5 } else { 0.0 };
        let x = (col as f32 - 5.0) * hex_width + offset_x;
        let y = (row as f32 - 5.0) * hex_height * 0.75; // 3/4 for proper tiling
        
        
        // Base color based on tile category
        let mut color = match tile.category {
            TileCategory::Farmland => [0.2, 0.6, 0.2, 1.0], // Green
            TileCategory::Water => [0.1, 0.3, 0.8, 1.0],    // Blue
        };
        
        // Modify color based on locality
        if let Some(locality) = &tile.locality {
            match locality.category {
                LocalityCategory::Capital(_) => color = [1.0, 0.0, 0.0, 1.0],      // Red for capitals
                LocalityCategory::City => color = [1.0, 0.8, 0.0, 1.0],            // Gold for cities
                LocalityCategory::PortCity => color = [0.0, 0.8, 1.0, 1.0],        // Cyan for ports
                LocalityCategory::Airport => color = [0.8, 0.8, 0.8, 1.0],         // Light grey for airports
            }
        }
        
        // Make army tiles brighter
        if tile.army.is_some() {
            color[0] = (color[0] + 0.3f32).min(1.0f32);
            color[1] = (color[1] + 0.3f32).min(1.0f32);
            color[2] = (color[2] + 0.3f32).min(1.0f32);
        }
        
        // Tint by player color
        if let Some(owner_idx) = tile.owner_index {
            let player_colors = [
                [1.0, 0.0, 0.0], // Red
                [0.0, 0.0, 1.0], // Blue  
                [0.0, 1.0, 0.0], // Green
            ];
            if owner_idx < player_colors.len() {
                let player_color = player_colors[owner_idx];
                color[0] = (color[0] + player_color[0]) * 0.5;
                color[1] = (color[1] + player_color[1]) * 0.5;
                color[2] = (color[2] + player_color[2]) * 0.5;
            }
        }
        
        let hex_start = vertices.len() as u16;
        
        // Create 6 vertices for the hexagon
        for i in 0..6 {
            let angle = i as f32 * std::f32::consts::PI / 3.0 - std::f32::consts::PI / 6.0;
            vertices.push(Vertex {
                position: [
                    x + angle.cos() * hex_radius,
                    y + angle.sin() * hex_radius,
                    0.0
                ],
                color,
            });
        }
        
        // Create 4 triangles to fill the hexagon
        indices.extend_from_slice(&[hex_start, hex_start + 1, hex_start + 2]);
        indices.extend_from_slice(&[hex_start, hex_start + 2, hex_start + 3]);
        indices.extend_from_slice(&[hex_start, hex_start + 3, hex_start + 4]);
        indices.extend_from_slice(&[hex_start, hex_start + 4, hex_start + 5]);
    }
    
    println!("Created world geometry with {} vertices and {} indices", vertices.len(), indices.len());
    (vertices, indices)
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Super Honeycomb Empire - Real Game")
        .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = instance.create_surface(&window).unwrap();
    
    let mut state = RealGameState::new(&window, &surface, &instance).await;
    let mut last_render_time = std::time::Instant::now();

    let window_id = window.id();

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id: event_window_id,
            } if event_window_id == window_id => {
                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        let now = std::time::Instant::now();
                        let _dt = now.duration_since(last_render_time);
                        last_render_time = now;

                        match state.render(&surface) {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => {
                                state.resize(state.size);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                // Request redraw
            }
            _ => {}
        }
    }).unwrap();
}

#[tokio::main]
async fn main() {
    run().await;
}
