use crate::{mquad::*, inputs::{draw_tile_selector, poll_inputs, poll_map_editor_inputs}, cubic::{Layout, self}, world::{TileCategory, Locality}};

use std::{collections::{HashMap, HashSet}, fs::{OpenOptions, File}};
use std::slice::Iter;
use macroquad::prelude::*;

use strum::IntoEnumIterator;
use strum::EnumIter;

use crate::{cubic::Cube, world::{Tile, World, LocalityCategory}};

pub struct Editor {
    pub world: World,
    pub brush_idx: usize,
    pub brush: BrushLayer,
    pub brush_size: usize,
}

#[derive(Debug, PartialEq, EnumIter)]
pub enum BrushLayer {
    Tile,
    Locality,
}

// impl BrushLayer {
//     fn paint<U, T: IntoEnumIterator + Into<U>>(&mut self, &cube: &Cube<i32>, category: T) {
//         let s = T::into(category);

//         match self {
//             Tile => {},
//             Locality => {}
//         }

//         if U == Tile {
//             // let tile = Tile::new(category);
//             self.world.insert(cube, s);
//         } else {
//             if let Some(tile) = self.world.get(&cube) {
//                 tile.locality = s;
//             }
//         }
//     }
// }

// impl BrushLayer {
//     pub fn iter() -> Iter<'static, BrushLayer> {
//         static MODES: [BrushLayer; 2] = [BrushLayer::Tile, BrushLayer::Locality];
//         MODES.iter()
//     }
// }

trait PlaceItem {
    fn place(&self, world: &mut World, cube: &Cube<i32>);
}

trait RemoveItem {
    fn remove(world: &mut World, cube: &Cube<i32>);
}

impl PlaceItem for LocalityCategory {
    fn place(&self, world: &mut World, cube: &Cube<i32>) {
        if let Some(tile) = world.get_mut(&cube) {tile.locality = Some(self.clone().into())}
    }
}

impl RemoveItem for LocalityCategory {
    fn remove(world: &mut World, cube: &Cube<i32>) {
        if let Some(tile) = world.get_mut(&cube) {tile.locality = None}
    }
}

impl PlaceItem for TileCategory {
    fn place(&self, world: &mut World, cube: &Cube<i32>) {
        world.entry(*cube)
             .and_modify(|t| t.category = self.clone())
             .or_insert(self.clone().into());
    }
}

impl RemoveItem for TileCategory {
    fn remove(world: &mut World, cube: &Cube<i32>) {
        world.remove(cube);
    }
}

impl Editor {
    pub fn new(world: World) -> Self {
        Editor{world, brush_idx: 0, brush: BrushLayer::Tile, brush_size: 1}
    }
    fn paint<T: IntoEnumIterator + PlaceItem + RemoveItem>(&mut self, cube: &Cube<i32>) {
        match T::iter().nth(self.brush_idx) {
            Some(category) => T::place(&category, &mut self.world, &cube),
            None => T::remove(&mut self.world, cube)
        }
    }
    pub fn click(&mut self, cube: &Cube<i32>) {
        match self.brush {
            BrushLayer::Tile => self.paint::<TileCategory>(cube),
            BrushLayer::Locality => self.paint::<LocalityCategory>(cube)
        };
    }
    pub fn right_click(&mut self) {
        self.brush_idx += 1;
        let max = match self.brush {
            BrushLayer::Tile => TileCategory::iter().count(),
            BrushLayer::Locality => LocalityCategory::iter().count(),
        };
        self.brush_idx %= max + 1;
    }
    pub fn toggle_layer(&mut self) {
        let mut b = BrushLayer::iter();
        let index = b.position(|x| x == self.brush).unwrap();
        let max = b.count();
        let new_index = (index + 1) % (max + 1);
        self.brush = BrushLayer::iter().nth(new_index).unwrap();
    }
}

pub fn save_map(hashmap: &HashMap<Cube<i32>, Tile>, path: &str) {
    let file = File::create(&path).expect("Failed to open the file.");

    match serde_json::to_writer(file, &hashmap) {
        Ok(()) => println!("Map saved successfully!"),
        Err(e) => eprintln!("Error during serialization: {}", e),
    }
}

impl World {

    pub fn from_json(path: &str) -> Self {
        let f = File::open(path)
            .expect("file should open read only");


        serde_json::from_reader(f).expect("file should be proper JSON")
        // let world: HashMap<Cube<i32>, Tile> = serde_json::from_reader(f).expect("file should be proper JSON");

        // let mut cubes_by_ownership: HashMap<usize, HashSet<Cube<i32>>> = HashMap::new();
        // let mut cubes_with_airport: HashSet<Cube<i32>> = HashSet::new();

        // world.iter().for_each(|(cube, tile)| {
        //     if let Some(index) = tile.owner_index {
        //         let set = cubes_by_ownership.entry(index).or_insert(HashSet::new());
        //         set.insert(*cube);
        //     }
        //     if let Some(locality) = &tile.locality {
        //         if matches!(locality.category, LocalityCategory::Airport) {
        //             cubes_with_airport.insert(*cube);
        //         }
        //     }
        // });

        // World {
        //     world,
        //     cubes_by_ownership,
        //     cubes_with_airport,
        // }
        
    }
}

// pub async fn run_editor(world: &World, layout: &Layout<f32>, assets: &Assets, time: f32) {
pub async fn run_editor(assets: &Assets) {

    let mut editor = Editor::new(World::new());

    let size = [32.,32.];
    // let size = [0.1,0.1]; // use this if in local coords
    let origin = [300., 300.];//[100.,600.];//[100.,300.];
    let mut layout = cubic::Layout{orientation: cubic::OrientationKind::Flat(cubic::FLAT), size, origin};

    let mut time = 0.0;

    loop {
        clear_background(DARKGRAY);

        poll_map_editor_inputs(&mut editor, &mut layout);

        draw_editor(&editor, &layout, &assets, time);

        next_frame().await;
        time += get_frame_time();
    }
    
}