mod draw;
mod input;

use std::io::Read;
use std::sync::Mutex;

use macroquad::camera::{set_camera, set_default_camera, Camera2D};
use macroquad::color::{*};
use macroquad::input::{mouse_position, is_mouse_button_pressed};
use macroquad::math::{Rect, Vec2};
use macroquad::prelude::{Conf, MaterialParams, UniformType, UniformDesc, ShaderSource, Texture2D};
use macroquad::prelude::{get_frame_time, load_material, gl_use_material, gl_use_default_material, get_fps, load_ttf_font_from_bytes, set_pc_assets_folder};
use macroquad::shapes::{draw_circle, draw_hexagon, draw_rectangle};
use macroquad::text::draw_text;
use macroquad::texture::{draw_texture_ex, render_target, DrawTextureParams};
use macroquad::window::next_frame;

use crate::backend::mquad::input::KeyMapper;
use crate::game::{Game, GameResources};
use crate::network::{Component, Message};
use crate::world::{self, LocalityCategory};
use world::TileCategory;

use crate::cubic;
use cubic::{Cube, Layout, OrientationKind, pixel_to_cube};

use crate::backend;
use backend::{Backend, Color};

use crate::ui::{self, get_ui_theme};

use draw::{Assets, owner_to_color};

macro_rules! bytes_asset {
    ($path:expr) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path))
    };
}

macro_rules! text_asset {
    ($path:expr) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path))
    };
}

const WATER_FRAGMENT_SHADER: &'static str = text_asset!("water_fragment_shader.glsl");
const WATER_VERTEX_SHADER: &'static str = text_asset!("water_vertex_shader.glsl");
const FONT: &[u8] = bytes_asset!("Iceberg-Regular.ttf");

async fn load_assets(init_layout: Layout<f32>) -> Assets {
    print!("loading assets!");
    set_pc_assets_folder(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/"));
    let font = load_ttf_font_from_bytes(FONT).unwrap();
    // let font = load_ttf_font("assets/Iceberg-Regular.ttf").await.unwrap();
    // let army = Texture2D::from_file_with_format(
    //     include_bytes!("../assets/army.png"),
    //     None,
    // );
    // let army: Texture2D = load_texture("assets/army.png").await.unwrap();
    let army_bytes = bytes_asset!("army.png");
    let army = Texture2D::from_file_with_format(army_bytes, None);

    // let port: Texture2D = load_texture("assets/port.png").await.unwrap();
    let port = Texture2D::from_file_with_format(
        bytes_asset!("port.png"),
        // include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/port.png")),
        None,
    );

    let airport = Texture2D::from_file_with_format(
        bytes_asset!("airport.png"),
        None,
    );
    // let airport: Texture2D = load_texture("assets/airport.png").await.unwrap();

    let fields = Texture2D::from_file_with_format(
        bytes_asset!("grass.png"),
        None,
    );
    // let fields = load_texture("assets/grass.png").await.expect("Failed to load texture");

    let water_shader = ShaderSource::Glsl{
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

    water_material.set_uniform("RectSize", (init_layout.size[0], init_layout.size[1]));

    print!("finished loading assets!");
    Assets{font, army, port, airport, fields, water_material}
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Super Honeycomb Empire".to_owned(),
        fullscreen: false,
        ..Default::default()
    }
}

// // #[macroquad::main(window_conf)]
// async fn main() {
//     set_pc_assets_folder("assets");
//     let mut assets = load_assets().await;

//     let mut layout = assets.init_layout.clone();

//     let mut time = 0.0;
//     while !exit {
//         exit = app.poll(&mut layout);
//         app.draw(&mut layout, &mut assets, time);
//         next_frame().await;
//         app.update();
//         if is_key_pressed(KeyCode::F1) {
//             app = app.swap();
//         }
//         time += get_frame_time();
//     }

// }

impl From<[f32; 4]> for Color {
    fn from(value: [f32; 4]) -> Self {
        Color {r: value[0], g: value[1], b: value[2] , a: value[3]}
    }
}

impl From<backend::Color> for macroquad::prelude::Color {
    fn from(value: backend::Color) -> Self {
        Self {r: value.r, g: value.g, b: value.b, a: value.a}
    }
}

pub struct Macroquad {
    assets: Option<Assets>,
    init_layout: Layout<f32>,
    key_mapper: KeyMapper,
    buffer: Mutex<Option<Vec<u8>>>,
    interface: iced_macroquad::Interface<ui::Message, iced_macroquad::iced::Theme>
}

impl Macroquad {
    pub async fn init(&mut self) {
        self.assets = Some(load_assets(self.init_layout.clone()).await);
    }
    fn assets(&self) -> &Assets {
        self.assets.as_ref().expect("assets not initialized")
    }
}

impl Backend for Macroquad {
    // type Assets = Assets;
    fn init(&'static mut self) {
        
    }
    fn render_ui<'a>(&mut self, messages: &mut Vec<ui::Message>, view: ui::View<'a>) {
        self.interface.view(messages, view);
    }
    fn set_buffer(&mut self, data: Vec<u8>) {
        *self.buffer.lock().unwrap() = Some(data);
    }
    fn take_buffer(&self) -> Option<Vec<u8>> {
        self.buffer.lock().unwrap().take()
    }
    fn new(init_layout: Layout<f32>) -> Self {
        let assets = None;
        let key_mapper = input::KeyMapper::new();
        let buffer = Mutex::new(None);
        let mut interface = iced_macroquad::Interface::<ui::Message>::new();
        let theme = get_ui_theme();
        interface.set_theme(theme);
        Self { assets, init_layout, key_mapper, buffer, interface }
    }
    fn run_loop<F>(self, mut f: F)
    where
        F: 'static + FnMut(&mut Self, f32) -> bool,
    {
        let conf = Conf {
            window_title: "My Game App".to_owned(),
            fullscreen: false,
            ..Default::default()
        };

        macroquad::Window::from_config(conf, async move {
            let mut backend = self;
            backend.init().await;
            loop {
                let dt = macroquad::time::get_frame_time();
                let exit = f(&mut backend, dt);
                if exit { break; }
                next_frame().await;
            }
        });
    }
    fn poll_inputs(&self, layout: &mut Layout<f32>) -> Message {
        input::poll_camera_inputs(layout);

        let mut message = Message::Tick;
        for action in self.key_mapper.poll() {
            match action {
                input::InputAction::CameraPan(delta) => {
                    layout.origin[0] += delta[0];
                    layout.origin[1] += delta[1];
                }
                input::InputAction::Message(msg) => {
                    message = msg;
                }
                input::InputAction::Click(cube) => {}
            }
        }

        message
    }
    fn poll_click_inputs(&self, layout: &mut Layout<f32>) -> Option<Cube<i32>> {
        let mut click = None;
        if is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
            let pos = mouse_position().into();
            let cube = cubic::pixel_to_cube(layout, pos).round::<i32>();
            click = Some(cube);
        }
        click
    }
    fn poll_right_click_inputs(&self, layout: &mut Layout<f32>) -> Option<Cube<i32>> {
        let mut click = None;
        if is_mouse_button_pressed(macroquad::input::MouseButton::Right) {
            let pos = mouse_position().into();
            let cube = cubic::pixel_to_cube(layout, pos).round::<i32>();
            click = Some(cube);
        }
        click
    }
    fn get_frame_time() -> f32 {
        get_frame_time()
    }
    async fn next_frame() {
        next_frame().await
    }
    fn clear(color: Color) {
        macroquad::prelude::clear_background(color.into());
    }
    fn draw_base_tiles(&self, view: &crate::world::World, layout: &Layout<f32>, time: f32) {
        // let lens_center = get_frame_time();
        self.assets().water_material.set_uniform("Time", time);
        let size = layout.size[0] as f32;
        for (cube, tile) in view.iter() {
            let pixel = Cube::<f32>::from(*cube).to_pixel(&layout);
            // let color = match tile.category {
            //     TileCategory::Farmland => LIGHTGRAY,
            //     TileCategory::Water => SKYBLUE,
            // };
            let x = pixel.0;
            let y = pixel.1;
            let vertical = match layout.orientation {
                OrientationKind::Pointy(_) => true,
                OrientationKind::Flat(_) => false,
            };
            match tile.category {
                TileCategory::Farmland => {
                    // set_texture("texture", &assets.fields);
                    // gl_use_material(assets.water_material);
                    draw_hexagon(x, y, size, layout.size[0]/20., vertical, BLACK, LIGHTGRAY);
                    // gl_use_default_material();
                },
                TileCategory::Water => {
                    gl_use_material(&self.assets().water_material);
                    draw_hexagon(x, y, size, 0., vertical, BLACK, SKYBLUE);
                    gl_use_default_material();
                }
            }
        }
    }
    fn draw_game_tiles(&self, view: &world::World, layout: &Layout<f32>) {
        let size = layout.size[0] as f32;
        let mut army_params = DrawTextureParams::default();
        army_params.dest_size = Some(Vec2{x: layout.size[0] as f32*1.5, y: layout.size[1] as f32*1.5});
        let mut airport_params = DrawTextureParams::default();
        airport_params.dest_size = Some(Vec2{x: layout.size[0] as f32, y: layout.size[1] as f32});
        let mut port_params = DrawTextureParams::default();
        port_params.dest_size = Some(Vec2{x: layout.size[0] as f32 * 0.9, y: layout.size[1] as f32 * 0.9});
        let airport_offset = layout.size[0] * 0.5;
        let port_offset = layout.size[0] * 0.5 * 0.9;
        let x_army_offset = layout.size[0] as f32 * 0.7;
        let y_army_offset = layout.size[1] as f32 * 0.7;
        for (cube, tile) in view.iter() {
            let pixel = Cube::<f32>::from(*cube).to_pixel(&layout);
            let x = pixel.0;
            let y = pixel.1;
            if tile.owner_index.is_some() {
                let color = owner_to_color(&tile.owner_index);
                let vertical = match layout.orientation {
                    OrientationKind::Pointy(_) => true,
                    OrientationKind::Flat(_) => false,
                };
                draw_hexagon(x, y, size, 0., vertical, BLACK, color);
                // match tile.category {
                //     TileCategory::Farmland => draw_hexagon(x, y, size, layout.size[0]/10., true, BLACK, color),
                //     TileCategory::Water => draw_hexagon(x, y, size, layout.size[0]/10., true, BLACK, SKYBLUE)
                // }
            }

            if tile.locality.is_some() {
                match tile.locality.as_ref().unwrap().category {
                    LocalityCategory::Capital(i) => {
                        if i == tile.owner_index.unwrap() {
                            draw_circle(x, y, size/2., RED)
                        } else {
                            draw_circle(x, y, size/2., PINK)
                        }
                    }
                    //LocalityCategory::SatelliteCapital => draw_circle(x, y, size/2., PINK),
                    LocalityCategory::City => draw_circle(x, y, size/2., DARKBROWN),
                    LocalityCategory::PortCity => {
                        draw_circle(x, y, size/2., BLUE);
                        draw_texture_ex(&self.assets().port, x - port_offset, y - port_offset, WHITE, port_params.clone());
                    },
                    LocalityCategory::Airport => {
                        draw_rectangle(x - size/2., y - size/2., size, size, DARKGREEN);
                        draw_texture_ex(&self.assets().airport, x - airport_offset, y - airport_offset, WHITE, airport_params.clone());
                    }
                }
            }
            if tile.army.is_some() {
                let color = owner_to_color(&tile.army.as_ref().unwrap().owner_index);
                // draw_texture(assets.army, x - x_army_offset, y - y_army_offset, color);
                draw_texture_ex(&self.assets().army, x - x_army_offset, y - y_army_offset, color, army_params.clone());
            }
            // if let Some(tile.locality) = locality {
            //     draw_circle(x, y, size, DARKBROWN)
            // }
        }
    }
    fn draw_army_legal_moves(game: &Game, layout: &Layout<f32>) {
        // let selection = game.current_player().selection;
        let Some(player) = game.current_player() else {return};
        let size = layout.size[0];
        if let Some(selection) = player.selection {
            let color = macroquad::prelude::Color::from_rgba(255, 255, 0, 136);//0x8800ffff
            let vertical = match layout.orientation {
                OrientationKind::Pointy(_) => true,
                OrientationKind::Flat(_) => false,
            };

            game.world.get_all_legal_moves(&selection, &game.current_player_index().unwrap()).iter().for_each(|cube| {
                let p = Cube::<f32>::from(*cube).to_pixel(&layout);
                draw_hexagon(p.0, p.1, size, size/10., vertical, BLACK, color);
            });
        }
    }
    fn draw_army_can_move_indicator(game: &Game, layout: &Layout<f32>) {
        let Some(current_player_index) = game.current_player_index() else {return};
        let size = layout.size[0];
        game.world.iter().for_each(|(cube, tile)|
            if tile.army.as_ref().is_some_and(|x| x.can_move & x.owner_index.is_some_and(|x| x == current_player_index)) 
            {
                let vertical = match layout.orientation {
                    OrientationKind::Pointy(_) => true,
                    OrientationKind::Flat(_) => false,
                };
                let p = Cube::<f32>::from(*cube).to_pixel(&layout);
                let color = macroquad::prelude::Color::from_rgba(255, 255, 0, 136);//0x8800ffff
                draw_hexagon(p.0, p.1, size, size/10., vertical, BLACK, color);
            }
        )
    }
    fn draw_army_info(world: &world::World, layout: &Layout<f32>) {
        let pos = mouse_position();
        let cube = pixel_to_cube(&layout, pos.into()).round();
        let mut nearest_cubes = cube.disc(2);
        nearest_cubes.push(cube);
        nearest_cubes.iter().for_each(|cube|
            if world.get(cube).is_some_and(|tile| tile.army.is_some()) {
                let p = Cube::<f32>::from(*cube).to_pixel(&layout);
                draw::army_info(p, &layout, &world.get(cube).unwrap());
            }
        )
    }
    fn draw_fps_counter(x: f32, y: f32, font_size: f32, color: Color) {
        draw_text(&get_fps().to_string(), 50.0, 50.0, 40., BLACK);
    }
    fn draw_map_control_summary(game: &Game) {
        let width = macroquad::window::screen_width();
        let ratio = 0.83; // 1700 / 2048
        let mut dy = 0.;
        for (idx, player) in game.players.iter().enumerate() {
            let color = owner_to_color(&Some(idx));
            let no_owned = game.world.cubes_by_ownership.get(&idx).unwrap().len();
            let percentage = no_owned as f32 / game.world.len() as f32 * 100.;
            let text = format!("{}: {:.2}%", player.name, percentage);
            let (x, mut y) = (ratio * width, 50.);
            y += dy;
            dy += 40.;
            draw_text(&text, x, y, 40., color);
        }
    }
    fn draw_river(segment: &crate::river::CubeSide, layout: &Layout<f32>) {
        let thickness = layout.size[0] / 4.;
        let color = BLUE;
        draw::draw_tile_side(segment, layout, thickness, color);
    }
    fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
        draw_circle(x, y, r, color.into());
    }
    /// Function to render a scene to an image
    async fn get_map_thumbnail(&self, world: &crate::World, width: f32, height: f32, resources: &GameResources) -> Vec<u8> {
        println!("getting thumb");
        // let scenario = Game::from_json(path);
        // let game = Game::from_json(path);
        // let scenario = game.swap();

        // texture size
        // let width = 900.0.x(); // 2560;
        // let height = 900.0.y(); // 1440;

        let mut layout = resources.init_layout.clone();

        // 2a. bounds with the original logical hex size (whatever init_layout is)
        let (min_x, min_y, max_x, max_y) = world::get_world_bounds(world, &layout);
        let map_w = max_x - min_x;
        let map_h = max_y - min_y;


        // 2b. uniform scale that preserves aspect ratio
        let scale = (width / map_w).min(height / map_h);

        layout.size = [
            resources.init_layout.size[0] * scale,
            resources.init_layout.size[1] * scale,
        ];

        // 2c. shift so the (scaled) map is centred in the texture
        layout.origin = [
            (width  - map_w * scale) * 0.5 - min_x * scale,
            (height - map_h * scale) * 0.5 - min_y * scale,
        ];

        // Create a render target (offscreen texture)
        let render_target = render_target(width as u32, height as u32);

        let mut render_target_cam = Camera2D::from_display_rect(Rect::new(0., 0., width, height));
        render_target_cam.render_target = Some(render_target.clone());
        set_camera(&render_target_cam);

        // Draw something
        backend::draw_thumb(world, &layout, self, resources, 1.);

        set_default_camera();

        // Retrieve the pixel data
        let image_data = render_target.texture.get_texture_data();

        // (Optional) Save the image
        image_data.export_png("output.png");

        //image_data
        let mut raw_bytes = image_data.bytes;
        flip_image_vertically(&mut raw_bytes, width as usize, height as usize);
        raw_bytes
        //Handle::from_rgba(width as u32, height as u32, raw_bytes) // Convert to Iced Image Handle
    }
}

fn flip_image_vertically(raw_bytes: &mut [u8], width: usize, height: usize) {
    let row_size = width * 4; // 4 bytes per pixel (RGBA)
    for y in 0..(height / 2) {
        let top_index = y * row_size;
        let bottom_index = (height - 1 - y) * row_size;
        for i in 0..row_size {
            raw_bytes.swap(top_index + i, bottom_index + i);
        }
    }
}
