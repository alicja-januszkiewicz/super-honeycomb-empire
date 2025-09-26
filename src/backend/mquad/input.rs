use crate::cubic::Cube;
use crate::cubic::OrientationKind;
use crate::game::Game;
use crate::network;
use crate::network::Client;
use crate::network::write_json_message;
use crate::network::Component;
use crate::network::Endpoint;
use crate::Controller;
use crate::Layout;
use crate::cubic;
use crate::map_editor::Editor;
use super::draw::Assets;
use crate::world::LocalityCategory;
use crate::world::Tile;
use crate::world::World;
use crate::network::Message;
use crate::backend::mquad::Macroquad;
use macroquad::input::*;
use macroquad::prelude::*;

const PAN_SPEED: f32 = 8.;
const ZOOM_SPEED: f32 = 1.;

pub enum InputAction {
    Message(Message),       // affects the component / game logic
    CameraPan([f32; 2]),    // directional pan
    Click(Cube<i32>),
}

pub struct KeyMapper {
    map: Vec<(KeyCode, fn() -> InputAction)>,
}

impl KeyMapper {
    pub fn new() -> Self {
        Self {
            map: vec![
                (KeyCode::Escape, || InputAction::Message(Message::Exit)),
                (KeyCode::F5, || InputAction::Message(Message::Save)),
                (KeyCode::F9, || InputAction::Message(Message::Load)),
                (KeyCode::Space, || InputAction::Message(Message::SkipTurn)),
                (KeyCode::F1, || InputAction::Message(Message::Swap)),
                (KeyCode::Right, || InputAction::CameraPan([-PAN_SPEED, 0.])) ,
                (KeyCode::Left, || InputAction::CameraPan([PAN_SPEED, 0.])) ,
                (KeyCode::Up, || InputAction::CameraPan([0., PAN_SPEED])) ,
                (KeyCode::Down, || InputAction::CameraPan([0., -PAN_SPEED])),
                (KeyCode::C, || InputAction::Message(Message::Clear)),
                (KeyCode::Tab, || InputAction::Message(Message::ToggleLayer)),
            ],
        }
    }

    pub fn poll(&self) -> Vec<InputAction> {
        self.map.iter()
            .filter_map(|(key, get_action)| if is_key_down(*key) { Some(get_action()) } else { None })
            .collect()
    }
}

// fn swtich() -> bool {
//     if is_key_pressed(KeyCode::F1) {
        
//     }
// }

pub fn poll_camera_inputs(layout: &mut Layout<f32>) {
    // WHEEL ZOOM
    let (_, mouse_wheel_y) = mouse_wheel();
    if mouse_wheel_y > 0. {
        layout.size[0] += ZOOM_SPEED;
        layout.size[1] += ZOOM_SPEED;
    } else if mouse_wheel_y < 0. {
        layout.size[0] -= ZOOM_SPEED;
        layout.size[1] -= ZOOM_SPEED;
    }
    if layout.size[0] <= 8. {layout.size[0] = 8.}
    if layout.size[1] <= 8. {layout.size[1] = 8.}

    // MOUSE PAN
    let (pos_x, pos_y) = mouse_position();
    if pos_x == 0. {
        layout.origin[0] += PAN_SPEED;
    }
    if pos_x == screen_width() - 1. {
        layout.origin[0] -= PAN_SPEED;
    }
    if pos_y == 0. {
        layout.origin[1] += PAN_SPEED;
    }
    if pos_y == screen_height() - 1. {
        layout.origin[1] -= PAN_SPEED;
    }

    // // KEY PAN
    // if is_key_down(KeyCode::Right) {
    //     layout.origin[0] -= PAN_SPEED;
    // }
    // if is_key_down(KeyCode::Left) {
    //     layout.origin[0] += PAN_SPEED;
    // }
    // if is_key_down(KeyCode::Up) {
    //     layout.origin[1] += PAN_SPEED;
    // }
    // if is_key_down(KeyCode::Down) {
    //     layout.origin[1] -= PAN_SPEED;
    // }
}

pub fn poll_map_editor_inputs(editor: &mut Editor, layout: &mut Layout<f32>) -> bool {
    if is_mouse_button_down(MouseButton::Left) {
        let pos = mouse_position().into();
        let cube = cubic::pixel_to_cube(layout, pos).round::<i32>();
        editor.click(&cube);
    }

    if is_mouse_button_pressed(MouseButton::Right) {
        editor.right_click();
    }

    if is_key_pressed(KeyCode::Tab) {
        editor.toggle_layer();
    }

    if is_key_down(KeyCode::F5) {
        std::fs::create_dir_all("assets/scenarios");
        editor.to_json("assets/scenarios/quicksave.json");
    }
    if is_key_down(KeyCode::F9) {
        std::fs::create_dir_all("assets/scenarios");
        *editor = Editor::from_json("assets/scenarios/quicksave.json");
    }

    if is_key_pressed(KeyCode::C) {
        *editor = Editor::new(World::new(), vec!())
    }

    let mut exit = false;
    if is_key_pressed(KeyCode::Escape) {
        exit = true
    }

    // if is_key_pressed(KeyCode::F1) {
        
    // }

    poll_camera_inputs(layout);

    exit
}


// // pub fn poll_inputs(game: &mut Game, client: Option<&Client>, layout: &mut Layout<f32>) -> (bool, Option<Box<dyn Component<Macroquad>>>) {
// pub fn poll_inputs_game(mut game: Box<Game>, client: Option<&Client>, layout: &mut Layout<f32>) -> (bool, Option<Box<dyn Component<Macroquad>>>)
// where Game: Component<Macroquad> {
//     // if is_key_down() {
//     //     let key = last_key_pressed();
//     // }

//     match game.current_player_index() {
//         Some(player_index) => {
//             let player = &game.players[player_index];

//             if is_mouse_button_pressed(MouseButton::Left) & matches!(player.controller, Controller::Human) {
//                 let pos = mouse_position().into();
//                 // let xyz = Cube::from::<i32>(Cube::new(1.,1.));
//                 let cube = cubic::pixel_to_cube(layout, pos).round::<i32>();
//                 if let Some(_) = game.world.get(&cube) {
//                     match game.click(&cube) {
//                         Some(command) => {
//                             match client {
//                                 Some(ref c) => {write_json_message(&c.stream, &Message::Command(command));},
//                                 None => game.execute_command(&command),
//                             }
//                         },
//                         None => {}
//                     };
//                 }
//             }
        
//             let player = &mut game.players[player_index];
//             if is_key_pressed(KeyCode::Space) & matches!(player.controller, Controller::Human) {
//                 match client {
//                     Some(ref c) => {write_json_message(&c.stream, &Message::SkipTurn);},
//                     None => player.skip_turn(),
//                 }
//             }
//         },
//         None => {},
//     }

//     poll_camera_inputs(layout);

//     match client {
//         Some(_) => {},
//         None => {
//             if is_key_down(KeyCode::F5) {
//                 //save_map(&game.world.world, "assets/saves/quicksave.json");
//                 std::fs::create_dir_all("assets/saves");
//                 game.to_json("assets/saves/quicksave.json");
//             }
//             if is_key_down(KeyCode::F9) {
//                 std::fs::create_dir_all("assets/saves");
//                 game = Box::new(Game::from_json("assets/saves/quicksave.json"));
//             }
//         }
//     }

//     let mut exit = false;
//     if is_key_pressed(KeyCode::Escape) {
//         exit = true
//     }

//     let mut app = None;
//     if is_key_pressed(KeyCode::F1) { app = Some(Box::new(game).swap()); } else {
//         app = Some(game as Box<dyn Component<Macroquad>>);
//     }

//     (exit, app)
// }

// pub fn poll_inputs<C: Component<Macroquad>, E: Endpoint>(
//     component: Box<C>,
//     endpoint: E,
//     layout: &mut Layout<f32>,
//     backend: &Macroquad,
// ) -> (bool, Option<Box<dyn Component<Macroquad>>>)
// {
//     poll_camera_inputs(layout);

//     for action in mapper.poll() {
//         match action {
//             InputAction::CameraPan(delta) => {
//                 layout.origin[0] += delta[0];
//                 layout.origin[1] += delta[1];
//             }
//             InputAction::Message(msg) => {
//                 if first_msg.is_none() {
//                     first_msg = Some(msg);
//                 }
//             }
//         }
//     }

//     let (mut exit, mut swap) = component.poll(layout, backend);

//     if is_key_pressed(KeyCode::Escape) {
//         exit = true;
//     }

//     if is_key_pressed(KeyCode::F1) {
//         swap = Some(swap.unwrap().swap());
//     }

//     (exit, swap)
// }




















// pub fn poll_inputs_client(mut game: Game, client: Client, layout: &mut Layout<f32>) -> bool {
//     // if is_key_down() {
//     //     let key = last_key_pressed();
//     // }

//     let player_index = game.current_player_index().unwrap(); // todo: unwrap safe here? will client never have empty players vec?
//     let player = &game.players[player_index];

//     if is_mouse_button_pressed(MouseButton::Left) & matches!(player.controller, Controller::Human) {
//         let pos = mouse_position().into();
//         // let xyz = Cube::from::<i32>(Cube::new(1.,1.));
//         let cube = cubic::pixel_to_cube(layout, pos).round::<i32>();
//         if let Some(_) = game.world.get(&cube) {
//             match game.click(&cube) {
//                 Some(command) => {
//                     // rely on server broadcasting it back to you to execute it
//                     write_json_message(&client.stream, &Message::Command(command));
//                     // match client.send_command(command) {
//                     //     Ok(command) => {
//                     //         client.app.execute_command(&command);
//                     //     },
//                     //     Err(_) => println!("command rejected by server")
//                     // }
                    
//                 },
//                 None => {}
//             };
//         }
//     }

//     let player = &mut game.players[player_index];
//     if is_key_pressed(KeyCode::Space) & matches!(player.controller, Controller::Human) {
//         write_json_message(&client.stream, &Message::SkipTurn);
//         // player.skip_turn();
//     }

//     poll_camera_inputs(layout);

//     // if is_key_down(KeyCode::F5) {
//     //     //save_map(&game.world.world, "assets/saves/quicksave.json");
//     //     std::fs::create_dir_all("assets/saves");
//     //     game.to_json("assets/saves/quicksave.json");
//     // }
//     // if is_key_down(KeyCode::F9) {
//     //     std::fs::create_dir_all("assets/saves");
//     //     *game = Game::from_json("assets/saves/quicksave.json");
//     // }

//     let mut exit = false;
//     if is_key_pressed(KeyCode::Escape) {
//         exit = true
//     }
//     exit
// }

