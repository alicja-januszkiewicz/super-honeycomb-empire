use crate::cubic::Cube;
use crate::cubic::OrientationKind;
use crate::game::Game;
use crate::network;
use crate::network::Client;
use crate::network::write_json_message;
use crate::Controller;
use crate::Layout;
use crate::cubic;
use crate::map_editor::Editor;
use crate::world::LocalityCategory;
use crate::world::Tile;
use crate::world::World;
use crate::network::Message;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

const PAN_SPEED: f32 = 8.;
const ZOOM_SPEED: f32 = 1.;

#[derive(Debug, Default)]
pub struct InputState {
    // Mouse state
    pub mouse_position: (f32, f32),
    pub mouse_wheel_delta: f32,
    pub mouse_buttons: std::collections::HashMap<MouseButton, bool>,
    pub mouse_buttons_pressed: std::collections::HashMap<MouseButton, bool>,
    
    // Keyboard state
    pub keys_down: std::collections::HashSet<KeyCode>,
    pub keys_pressed: std::collections::HashSet<KeyCode>,
    
    // Window state
    pub window_size: (f32, f32),
    
    // Frame state (cleared each frame)
    pub should_exit: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = (position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let is_pressed = *state == ElementState::Pressed;
                self.mouse_buttons_pressed.insert(*button, is_pressed && !self.mouse_buttons.get(button).unwrap_or(&false));
                self.mouse_buttons.insert(*button, is_pressed);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        self.mouse_wheel_delta = *y;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        self.mouse_wheel_delta = pos.y as f32 * 0.01; // Convert to line-like units
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            let was_pressed = self.keys_down.contains(&key_code);
                            self.keys_pressed.insert(key_code);
                            self.keys_down.insert(key_code);
                        }
                        ElementState::Released => {
                            self.keys_down.remove(&key_code);
                        }
                    }
                }
            }
            WindowEvent::Resized(size) => {
                self.window_size = (size.width as f32, size.height as f32);
            }
            WindowEvent::CloseRequested => {
                self.should_exit = true;
            }
            _ => {}
        }
    }
    
    pub fn end_frame(&mut self) {
        // Clear per-frame state
        self.mouse_buttons_pressed.clear();
        self.keys_pressed.clear();
        self.mouse_wheel_delta = 0.0;
        self.should_exit = false;
    }
    
    // Helper methods to match macroquad API
    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }
    
    pub fn mouse_wheel(&self) -> (f32, f32) {
        (0.0, self.mouse_wheel_delta)
    }
    
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons.get(&button).unwrap_or(&false).clone()
    }
    
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.get(&button).unwrap_or(&false).clone()
    }
    
    pub fn is_key_down(&self, key_code: KeyCode) -> bool {
        self.keys_down.contains(&key_code)
    }
    
    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        self.keys_pressed.contains(&key_code)
    }
    
    pub fn screen_width(&self) -> f32 {
        self.window_size.0
    }
    
    pub fn screen_height(&self) -> f32 {
        self.window_size.1
    }
}

fn poll_camera_inputs(layout: &mut Layout<f32>, input_state: &InputState) {
    // WHEEL ZOOM
    let (_, mouse_wheel_y) = input_state.mouse_wheel();
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
    let (pos_x, pos_y) = input_state.mouse_position();
    if pos_x == 0. {
        layout.origin[0] += PAN_SPEED;
    }
    if pos_x == input_state.screen_width() - 1. {
        layout.origin[0] -= PAN_SPEED;
    }
    if pos_y == 0. {
        layout.origin[1] += PAN_SPEED;
    }
    if pos_y == input_state.screen_height() - 1. {
        layout.origin[1] -= PAN_SPEED;
    }

    // KEY PAN
    if input_state.is_key_down(KeyCode::ArrowRight) {
        layout.origin[0] -= PAN_SPEED;
    }
    if input_state.is_key_down(KeyCode::ArrowLeft) {
        layout.origin[0] += PAN_SPEED;
    }
    if input_state.is_key_down(KeyCode::ArrowUp) {
        layout.origin[1] += PAN_SPEED;
    }
    if input_state.is_key_down(KeyCode::ArrowDown) {
        layout.origin[1] -= PAN_SPEED;
    }
}

pub fn poll_map_editor_inputs(editor: &mut Editor, layout: &mut Layout<f32>, input_state: &InputState) -> bool {
    if input_state.is_mouse_button_down(MouseButton::Left) {
        let pos = input_state.mouse_position().into();
        let cube = cubic::pixel_to_cube(layout, pos).round::<i32>();
        editor.click(&cube);
    }

    if input_state.is_mouse_button_pressed(MouseButton::Right) {
        editor.right_click();
    }

    if input_state.is_key_pressed(KeyCode::Tab) {
        editor.toggle_layer();
    }

    if input_state.is_key_down(KeyCode::F5) {
        std::fs::create_dir_all("assets/scenarios");
        editor.to_json("assets/scenarios/quicksave.json");
    }
    if input_state.is_key_down(KeyCode::F9) {
        std::fs::create_dir_all("assets/scenarios");
        *editor = Editor::from_json("assets/scenarios/quicksave.json");
    }

    if input_state.is_key_pressed(KeyCode::KeyC) {
        *editor = Editor::new(World::new(), vec!())
    }

    let mut exit = false;
    if input_state.is_key_pressed(KeyCode::Escape) {
        exit = true
    }

    // if input_state.is_key_pressed(KeyCode::F1) {
        
    // }

    poll_camera_inputs(layout, input_state);

    exit
}

pub fn poll_inputs(game: &mut Game, client: Option<&Client>, layout: &mut Layout<f32>, input_state: &InputState) -> bool {
    // if input_state.is_key_down() {
    //     let key = last_key_pressed();
    // }

    match game.current_player_index() {
        Some(player_index) => {
            let player = &game.players[player_index];

            if input_state.is_mouse_button_pressed(MouseButton::Left) & matches!(player.controller, Controller::Human) {
                let pos = input_state.mouse_position().into();
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
            if input_state.is_key_pressed(KeyCode::Space) & matches!(player.controller, Controller::Human) {
                match client {
                    Some(ref c) => {write_json_message(&c.stream, &Message::SkipTurn);},
                    None => player.skip_turn(),
                }
            }
        },
        None => {},
    }

    poll_camera_inputs(layout, input_state);

    match client {
        Some(_) => {},
        None => {
            if input_state.is_key_down(KeyCode::F5) {
                //save_map(&game.world.world, "assets/saves/quicksave.json");
                std::fs::create_dir_all("assets/saves");
                game.to_json("assets/saves/quicksave.json");
            }
            if input_state.is_key_down(KeyCode::F9) {
                std::fs::create_dir_all("assets/saves");
                *game = Game::from_json("assets/saves/quicksave.json");
            }
        }
    }

    let mut exit = false;
    if input_state.is_key_pressed(KeyCode::Escape) {
        exit = true
    }
    exit
}
