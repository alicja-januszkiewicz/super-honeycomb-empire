use crate::{cubic::Layout, game::Game, network::Component, river::CubeSide, world::World};

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
    type Assets;

    fn new(init_layout: Layout<f32>) -> Self;

    /// Return `true` to exit the loop.
    fn run_loop<F>(self, f: F)
    where
        F: 'static + FnMut(&mut Self, f32) -> bool;

    //fn poll_events<F: FnMut()> (event: F);
    fn poll_inputs(game: Box<Game>, client: Option<&crate::Client>, layout: &mut Layout<f32>) -> (bool, Option<Box<dyn Component<Self>>>);

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


