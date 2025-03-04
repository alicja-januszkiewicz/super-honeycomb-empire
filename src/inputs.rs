use crate::cubic::Cube;
use crate::cubic::OrientationKind;
use crate::game::Game;
use crate::network;
use crate::network::Client;
use crate::write_json_message;
use crate::Controller;
use crate::Layout;
use crate::cubic;
use crate::map_editor::Editor;
use crate::mquad::Assets;
use crate::world::LocalityCategory;
use crate::world::Tile;
use crate::world::World;
use crate::Message;
use macroquad::input::*;
use macroquad::prelude::*;

const PAN_SPEED: f32 = 8.;
const ZOOM_SPEED: f32 = 1.;

// fn swtich() -> bool {
//     if is_key_pressed(KeyCode::F1) {
        
//     }
// }

fn poll_camera_inputs(layout: &mut Layout<f32>) {
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

    // KEY PAN
    if is_key_down(KeyCode::Right) {
        layout.origin[0] -= PAN_SPEED;
    }
    if is_key_down(KeyCode::Left) {
        layout.origin[0] += PAN_SPEED;
    }
    if is_key_down(KeyCode::Up) {
        layout.origin[1] += PAN_SPEED;
    }
    if is_key_down(KeyCode::Down) {
        layout.origin[1] -= PAN_SPEED;
    }
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

pub fn poll_inputs(game: &mut Game, client: Option<&Client>, layout: &mut Layout<f32>) -> bool {
    // if is_key_down() {
    //     let key = last_key_pressed();
    // }

    match game.current_player_index() {
        Some(player_index) => {
            let player = &game.players[player_index];

            if is_mouse_button_pressed(MouseButton::Left) & matches!(player.controller, Controller::Human) {
                let pos = mouse_position().into();
                // let xyz = Cube::from::<i32>(Cube::new(1.,1.));
                let cube = cubic::pixel_to_cube(layout, pos).round::<i32>();
                if let Some(_) = game.world.get(&cube) {
                    match game.click(&cube) {
                        Some(command) => {
                            match client {
                                Some(ref c) => {write_json_message(&c.stream, &Message::Command(command));                                },
                                None => game.execute_command(&command),
                            }
                        },
                        None => {}
                    };
                }
            }
        
            let player = &mut game.players[player_index];
            if is_key_pressed(KeyCode::Space) & matches!(player.controller, Controller::Human) {
                match client {
                    Some(ref c) => {write_json_message(&c.stream, &Message::SkipTurn);},
                    None => player.skip_turn(),
                }
            }
        },
        None => {},
    }

    poll_camera_inputs(layout);

    match client {
        Some(_) => {},
        None => {
            if is_key_down(KeyCode::F5) {
                //save_map(&game.world.world, "assets/saves/quicksave.json");
                std::fs::create_dir_all("assets/saves");
                game.to_json("assets/saves/quicksave.json");
            }
            if is_key_down(KeyCode::F9) {
                std::fs::create_dir_all("assets/saves");
                *game = Game::from_json("assets/saves/quicksave.json");
            }
        }
    }

    let mut exit = false;
    if is_key_pressed(KeyCode::Escape) {
        exit = true
    }
    exit
}

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

pub fn draw_tile_selector(&layout: &Layout<f32>) {
    let vertical = match layout.orientation {
        OrientationKind::Pointy(_) => true,
        OrientationKind::Flat(_) => false,
    };
    let size = layout.size[0];
    let pos = mouse_position().into();
    // let pos = mouse_position_local().into();
    let cube = cubic::pixel_to_cube(&layout, pos).round();
    let p = cube.to_pixel(&layout);
    let [x, y] = [p.0, p.1];
    // println!("{:?}", pos);
    let color = Color::from_rgba(224, 208, 64, 136); // 0x88d0e040
    draw_hexagon(x, y, size, size/10., vertical, BLACK, color);
}

fn draw_locality_name(layout: &Layout<f32>, cube: &Cube<i32>, tile: &Tile, font: Font) {
    let params = TextParams{
        font: Some(&font),
        font_size: (0.9 * layout.size[0]) as u16, // 6,
        //font_scale: (0.7 * layout.size[0]),
        ..Default::default()
    };
    if let Some(locality) = &tile.locality {
        // if let Some(category) = locality.category {}
        let color = match locality.category {
            LocalityCategory::Capital(_) => WHITE, // tile.locality.starting_owner.color,
            _ => WHITE,
        };

        let p = Cube::<f32>::from(*cube).to_pixel(&layout);
        let [mut x, mut y] = [p.0, p.1];
        let center: Vec2 = get_text_center(locality.name.as_str(), params.font, params.font_size, params.font_scale, params.rotation);
        x -= center.x;
        y -= layout.size[1] + center.y;
        draw_text_ex(locality.name.as_str(), x, y, params);
    }

}


// Draws names of all localities within 5 cubes of the cursor.
pub fn draw_all_locality_names(world: &World, layout: &Layout<f32>, assets: &Assets) {
    let pos = mouse_position().into();
    let cube = cubic::pixel_to_cube(&layout, pos).round();

    let cubes_within_draw_range = cube.disc(5);
    for cube in cubes_within_draw_range {
        match world.get(&cube) {
            Some(tile) if tile.locality.is_some() => draw_locality_name(layout, &cube, tile, assets.font.clone()),
            _ => {},
        }
    }
}
