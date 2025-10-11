#![feature(trait_alias)]
#![allow(warnings)]
#![feature(trivial_bounds)]

mod backend;
mod cubic;
mod game;
mod world;
mod ai;
mod inputs;
mod map_editor;
mod river;
mod shapefiles;
mod fog;
mod rules;
mod network;
mod cli;
mod ui;

use ui::Ui;
use clap::Parser;
use fog::*;
use ai::*;
use cli::*;
use cubic::*;
use game::*;
use glyphon::cosmic_text::ttf_parser::gpos::MarkArray;
use rules::Ruleset;
// use ui::{main_menu};
use world::*;
use inputs::*;
use map_editor::*;
use network::*;
use std::{collections::HashMap, f32::consts::PI, fs::File};
use dbase;

use crate::backend::{mquad::Macroquad, Backend};

// pub struct Assets<F: FontHandle, T: TextureHandle, M: MaterialHandle> {
//     pub locality_names: Vec<String>,
//     pub font: F,
//     pub army: T,
//     pub port: T,
//     pub airport: T,
//     pub fields: T,
//     pub water_material: M,
//     pub init_layout: Layout<f32>,
//     pub shape: Vec<(f32, f32)>,
//     pub river: Vec<(usize, f32, f32)>,
// }

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
    println!("world initialised!");
    game
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

fn get_game<B: Backend, T: Component<B>> (resources: &mut GameResources) -> Box<dyn Component<B, Message = network::Message>> {
    Box::new(new_game(resources))
}

fn get_app<M, B>(resources: &mut GameResources) -> Box<dyn ErasedComponent<Macroquad>>
where
    B: Backend,
    M: network::Mode + 'static,
    M::Endpoint: Chat + 'static,
    App<M, Macroquad, Game>: Component<Macroquad, Message = network::Message>, 
    // <network::App<M, B, C> as network::Component<B>>::Message: ,
{
    let mut game = new_game(resources);
    let args = Cli::parse();
    match args.mode {
        cli::Mode::Client => println!("Running in client mode"),
        cli::Mode::Server => println!("Running in server mode"),
        cli::Mode::Offline => println!("Running in offline mode"),
    }

    // Construct a placeholder endpoint
    let mut endpoint: Box<dyn Chat> = Box::new(Offline);

    // Replace it with the runtime choice
    let replacement: Box<dyn Chat> = match args.mode {
        cli::Mode::Client => Box::new(Client::new(&args.addrs).unwrap()),
        cli::Mode::Server => Box::new(Server::new(&args.addrs).unwrap()),
        cli::Mode::Offline => Box::new(Offline),
    };

    let endpoint_ = std::mem::replace(&mut endpoint, replacement);

    // Downcast to the concrete endpoint type
    let endpoint = match args.mode {
        cli::Mode::Client => *endpoint_.into_any().downcast::<M::Endpoint>().unwrap(),
        cli::Mode::Server => *endpoint_.into_any().downcast::<M::Endpoint>().unwrap(),
        cli::Mode::Offline => *endpoint_.into_any().downcast::<M::Endpoint>().unwrap(),
    };

    Box::new(App::<M, Macroquad, Game>::new(game, endpoint))
}

// fn get_app(resources: &mut GameResources) -> Box<dyn Component<Macroquad>> {
//     let game: Game = new_game(resources);
//     println!("about to create Box(Game)");
//     let b = Box::new(game);
//     println!("about to return Box(Game)");
//     b
// }



// #[macroquad::main(window_conf)]
// async fn main() {
//     set_pc_assets_folder("assets");
//     let mut assets = load_assets().await;

//     let (mut exit, ui) = main_menu(&mut assets).await;

//     if exit {return};

//     // let mut app = App::<Offline, Game>::from_ui(ui, &mut assets);

//     // let mut app: Box<App<Offline, Game>>= Box::new(App::<Offline, Game>::from_ui(ui, &mut assets));
//     // let swapped_app: Box<App<Offline, Editor>>= app.swap_component();

//     // let swapped_app: Box<App<Offline, <Game as Component>::Swap>> = app.swap_component();
//     // let app: Box<dyn Component> = Box::new(App::<Offline, Game>::from_ui(ui, &mut assets));

//     // let mut app: Box<dyn Component> = Box::new(App::<Offline, Game>::from_ui(ui, &mut assets));
//     let mut app = App::from_ui(ui, &mut assets);


//     // let args = Cli::parse();
//     // match args.mode {
//     //     Mode::Client => println!("Running in client mode"),
//     //     Mode::Server => println!("Running in server mode"),
//     //     Mode::Offline => println!("Running in offline mode"),
//     // }

//     // let mut endpoint: Box<dyn Endpoint> = match args.mode { // possibly replace Box<dyn Endpoint> with trait Endpoint if and when existential types are stabilised
//     //     Mode::Client => Box::new(ClientApp::new(game, &args.addrs).unwrap()),
//     //     Mode::Server => Box::new(ServerApp::new(game, &args.addrs).unwrap()),
//     //     Mode::Offline => Box::new(NullEndpoint::new(game)),
//     // };

//     // endpoint = Box::new(endpoint.swap_app())
    
//     // let mut client = Client::new(game, "").unwrap();
//     // let mut client_e = Client::new(Editor::new(World::new(), Vec::new()), "").unwrap();
//     // let mut server = Server::new(new_game(&mut assets), "").unwrap();
//     // let mut server_e = Server::new(Editor::new(World::new(), Vec::new()), "").unwrap();
//     // let mut endpoint: &mut dyn Endpoint = &mut client;

//     // endpoint.app = endpoint.app.swap();
//     // either box and getters and setters or
//     // enum and matching 


//     // let app: &mut dyn Component = &mut match state {
//     //     State::Game(game) => game,
//     //     State::Editor(editor) => editor,
//     // };//&mut game;

//     // let app: &mut dyn Component = &mut game;

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


// let size = [0.1,0.1]; // use this if in local coords
// let camera = &Camera2D {
//     // zoom: vec2(0.001, 0.001),
//     // offset: vec2(-0.5,-0.1),
//     zoom: vec2(1., -1.),
//     ..Default::default()
// };

// // set_camera(camera);





// #[macroquad::main(window_conf)]
// async fn main() {
//     // let (mut exit, ui) = main_menu(&mut assets).await;
//     // if exit {return};
//     // let mut app = App::from_ui(ui, &mut assets);

//     let mut exit = false;

//     let mut resources = load_resources();
//     let backend = Macroquad::new(resources.init_layout).await;
//     // let app: &mut dyn Component<crate::backend::mquad::Macroquad> = &mut get_app(&mut resources);
//     let mut app: Box<dyn Component<crate::backend::mquad::Macroquad>> = get_app::<Macroquad, Game>(&mut resources);

//     let mut layout = resources.init_layout.clone();

//     let mut time = 0.0;
//     while !exit {
//         app.draw(&mut layout, &backend, &resources, time);
//         app.update();
//         // if is_key_pressed(KeyCode::F1) {
//         //     app = app.swap();
//         // }
//         time += Macroquad::get_frame_time();
//         exit = app.poll(&mut layout, &backend);
//         // exit = backend.poll_events(|event| {
//         //     app.poll(event, &mut layout);
//         // });

//         Macroquad::next_frame().await;
//     }
// }

fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");
    // let window_config = WindowConfig {
    //     title: "Super Honeycomb Empire".into(),
    //     fullscreen: false,
    // };

    // let mut endpoint: Box<dyn Endpoint> = Box::new(Offline);
    // let mut endpoint = Offline;


    let mut resources = load_resources();
    let init_layout = resources.init_layout.clone();
    let resources_box = std::rc::Rc::new(std::cell::RefCell::new(load_resources()));
    let init_layout = resources_box.borrow().init_layout.clone();
    let backend_box = Box::new(Macroquad::new(init_layout.clone()));
    // let app: &mut dyn Component<crate::backend::mquad::Macroquad, Message = Message> = &mut get_app(&mut resources);
    // let component = Some(&mut get_app(&mut resources));
    // let mut component = Some(get_app::<network::Offline, Macroquad>(&mut resources));
    let mut component: Option<Box<dyn ErasedComponent<Macroquad>>> = Some(Box::new(Ui::new()));

    // let mut component: Option<Box<dyn Component<crate::backend::mquad::Macroquad>>> = Some(get_app::<Macroquad, Game>(&mut resources));
    // let component = new_game(&mut resources);

    // let mut app= Some(Box::new(App::<Offline, Macroquad, Game>::new(component, endpoint)));
    // app = app as Box<dyn Component<Macroquad>>;
    
    let resources_box_clone = std::rc::Rc::clone(&resources_box);




    // let component_box: std::rc::Rc<std::cell::RefCell<Box<dyn Component<Macroquad>>>> = std::rc::Rc::new(std::cell::RefCell::new(Box::new(network::Empty)));

    // let mut exit = false;
    // // let mut state = ui::Ui::<Macroquad>::new();

    // let (exit, ui) = backend_box.run_loop(move |backend, time| {
    //     let component_box = std::rc::Rc::clone(&component_box);
    //     let resources_box = &resources_box_clone;
    //     let mut resources = resources_box.borrow_mut();
    //     let (exit, ui) = crate::ui::main_menu(backend, &mut resources);
    //     // resources_box = Box::new(resources);
    //     // let res = App::from_ui(ui);
    //     // *component_box.borrow_mut() = res;
    //     (exit, ui)
    // });




    // let mut resources = resources_box;
    let backend_box = Box::new(Macroquad::new(init_layout.clone()));
    // if exit {return};
    // let mut app = App::from_ui(ui, &mut resources);

    // let mut component = Some(get_app::<Macroquad, _>(&mut resources.borrow_mut()));
    // let mut app = Some(get_app::<Macroquad, _>(&mut resources));

    let mut layout = init_layout.clone();
    
    // let mut component = Some(get_app::<Macroquad, _>(&mut resources.borrow_mut()));
    let mut exit = false;
    backend_box.run_loop(move |backend, _time| {
        let (next_component, exit) = component.take().unwrap().step(&mut layout, backend, &resources, _time);
        component = Some(next_component);
        exit
    });
}
