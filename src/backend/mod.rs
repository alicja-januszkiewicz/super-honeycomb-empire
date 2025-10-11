use crate::{cubic::Layout, game::{Game, GameResources}, network::{Component, Message}, river::CubeSide, world::World};
use crate::ui;

#[cfg(feature="wgpu")]
mod wgpu;

// #[cfg(feature="mquad")]
#[cfg_attr(any(feature="mquad", rust_analyzer), path = "mquad/mod.rs")]
pub mod mquad;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }

// impl From<[f32; 4]> for Color {
//     fn from(value: [f32; 4]) -> Self {
//         Self {r: value[0], g: value[1], b: value[2] , a: value[3]}
//     }
// }

pub trait Backend {
    // type Event;
    // type Assets;
    fn render_ui<'a>(&mut self, messages: &mut Vec<ui::Message>, view: ui::View<'a>);

    fn set_buffer(&mut self, data: Vec<u8>);
    fn take_buffer(&self) -> Option<Vec<u8>>;

    fn new(init_layout: Layout<f32>) -> Self;

    /// Return `true` to exit the loop.
    fn run_loop<F>(self, f: F)
    where
        F: 'static + FnMut(&mut Self, f32) -> bool;

    //fn poll_events<F: FnMut()> (event: F);
    fn poll_inputs(&self, layout: &mut Layout<f32>) -> Message;
    fn poll_click_inputs(&self, layout: &mut Layout<f32>) -> Option<crate::cubic::Cube<i32>>;
    fn poll_right_click_inputs(&self, layout: &mut Layout<f32>) -> Option<crate::cubic::Cube<i32>>;

    // fn assets(&self) -> &Self::Assets;
    // fn assets_mut(&mut self) -> &mut Self::Assets;

    // Frame lifecycle
    // fn begin_frame(&mut self, viewport: [u32; 2]);
    // fn end_frame(&mut self);
    fn get_frame_time() -> f32;
    async fn next_frame();

    // Drawing
    // fn draw_hex(&mut self, pos: [f32; 2], size: f32, color: Color);
    // fn draw_texture(&mut self, texture: TextureId, pos: [f32; 2], size: [f32; 2], tint: Color);
    // fn draw_text(&mut self, text: &str, pos: [f32; 2], size: f32, color: Color);
    fn clear(color: Color);
    // fn draw_base_tiles(view: &World, layout: &Layout<f32>, assets: &Self::Assets, time: f32);
    // fn draw_game_tiles(view: &World, layout: &Layout<f32>, assets: &Self::Assets);
    fn draw_base_tiles(&self, view: &World, layout: &Layout<f32>, time: f32);
    fn draw_game_tiles(&self, view: &World, layout: &Layout<f32>);
    fn draw_army_legal_moves(game: &Game, layout: &Layout<f32>);
    fn draw_army_can_move_indicator(game: &Game, layout: &Layout<f32>);
    fn draw_army_info(world: &World, layout: &Layout<f32>);
    fn draw_fps_counter(x: f32, y: f32, font_size: f32, color: Color);
    fn draw_map_control_summary(game: &Game);
    fn draw_river(segment: &CubeSide, layout: &Layout<f32>);
    fn draw_circle(x: f32, y: f32, r: f32, color: Color);

    async fn get_map_thumbnail(&self, world: &crate::World, width: f32, height: f32, resources: &GameResources) -> Vec<u8>;

    // Resource creation
    // fn create_texture(&mut self, data: &[u8], size: [u32; 2]) -> Result<TextureId, BackendError>;
    // fn create_font(&mut self, ttf_bytes: &[u8]) -> Result<FontId, BackendError>;
}

// #[derive(Debug)]
// pub enum BackendError {
//     InvalidData,
//     GpuError(String),
//     // etc.
// }

// #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
// pub struct TextureId(u32);

// #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
// pub struct FontId(u32);

pub fn draw_thumb<B: Backend>(world: &World, &layout: &Layout<f32>, backend: &B, resources: &GameResources, time: f32) {
    let color = Color { r: 0.31, g: 0.31, b: 0.31, a: 1.0 };
    B::clear(color);

    backend.draw_base_tiles(&world, &layout, time);
    backend.draw_game_tiles(&world, &layout);

    for cs in &world.rivers {
        B::draw_river(&cs, &layout);
    }

    let mut shape = resources.river.clone();

    // let COLORS = vec!(BEIGE, BLACK, BLUE, BROWN, GOLD, GREEN, LIME, MAGENTA, MAROON, ORANGE, PINK, PURPLE, RED, VIOLET, WHITE, YELLOW,);
    let COLORS = vec!([122.,122.,122.,122.].into());

    for j in 1..shape.len() {
        let (id, mut x, mut y) = shape[j];
        x *= layout.size[0] / resources.init_layout.size[0];
        y *= layout.size[1] / resources.init_layout.size[1];
        x += layout.origin[0];
        y += layout.origin[1];
        let color = COLORS[j % COLORS.len()];
        B::draw_circle(x, y, 8., color);
    }
}
