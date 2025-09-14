/// The worldgen module contains random world generation functions.
/// It exports the generate_world() function for the Game class __init__()
/// method. In the future it will be used by some other module that will
/// allow the user to tweak the worldgen settings from within the game.

// import data
// from playergen import Player
use core::cmp::max;
use core::cmp::min;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::Player;
use crate::World;
use crate::Tile;
use crate::TileCategory;
use crate::Cube;
use crate::Locality;
use crate::LocalityCategory;
use crate::cubic::DIRECTIONS;
use crate::cubic::Layout;
use crate::cubic::OrientationKind;
use crate::cubic::Pixel;
use crate::cubic::pixel_to_cube;
use crate::river::CubeSide;
use super::extend_borders;
// use crate::cubic::Layout;
// use crate::cubic::POINTY;
// use crate::cubic::FLAT;

extern crate rand;
use num::iter;
use rand::seq::IteratorRandom;
use rand::Rng;
use rand::random;
use rand::seq::index::sample;
use strum::AsStaticStr;
use strum::Display;
use strum::EnumCount;
use strum::EnumIter;

// #layout = cubic.Layout(cubic.orientation_pointy, cubic.Point(50, 50), cubic.Point(800, 550))
// let layout = Layout(POINTY, (.02, .02), (.2, 0));
// #layout = cubic.Layout(cubic.orientation_pointy, cubic.Point(1, 1), cubic.Point(0, 0))

#[derive(AsStaticStr, Clone, Debug, EnumIter, EnumCount, PartialEq)]
pub enum CapitalsGen {
    Classic,
    Random(usize), 
    MaxDist(usize),
}

impl Default for CapitalsGen {
    fn default() -> Self {
        CapitalsGen::Random(4)
    }
}

#[derive(AsStaticStr, Clone, Debug, Default, EnumIter, EnumCount, PartialEq)]
pub enum LocalitiesGen {
    Random, #[default]
    RandomOts, // One Tile of Space
}

#[derive(AsStaticStr, Clone, Debug, EnumIter, EnumCount, PartialEq)]
pub enum ShapeGen {
    Classic,
    Hexagonal(i32), 
    Custom(Vec<(f32, f32)>),
}

impl Default for ShapeGen {
    fn default() -> Self {
        Self::Hexagonal(12)
    }
}

#[derive(AsStaticStr, Clone, Debug, EnumIter, EnumCount, PartialEq)]
pub enum RiverGen {
    Random(usize, f32),
    Custom(Vec<(usize, f32, f32)>),
}

impl Default for RiverGen {
    fn default() -> Self {
        Self::Random(20, 5.)
    }
}

#[derive(Debug)]
pub struct Template {
    pub name: &'static str,
    pub description: &'static str,
    pub shape: Option<ShapeGen>,
    pub capitals: Option<CapitalsGen>,
    pub localities: Option<LocalitiesGen>,
    pub rivers: Option<RiverGen>,
}

const CLASSIC: Template = {
    Template {
        name: "Classic",
        description: "Hex Empire 1 inspired map template",
        shape: Some(ShapeGen::Classic),
        capitals: Some(CapitalsGen::Classic),
        localities: Some(LocalitiesGen::Random),
        rivers: None,
    }
};

impl Default for Template {
    fn default() -> Self {
        CLASSIC
    }
}

const HEX: Template = {
    Template {
        name: "Hexagon",
        description: "Hexagonal map template.",
        shape: Some(ShapeGen::Hexagonal(12)),
        capitals: Some(CapitalsGen::Random(6)),
        localities: Some(LocalitiesGen::Random),
        rivers: Some(RiverGen::Random(20, 5.)),
    }
};

pub const CUSTOM: Template = {
    Template {
        name: "Custom",
        description: "",
        shape: None,
        capitals: None,
        localities: None,
        rivers: None,
    }
};

pub enum TemplateFields {
    Name,
    Capitals,
    Localities,
    Rivers,
    Shape,
}

#[derive(Clone, Debug, Default, EnumIter, EnumCount, PartialEq)]
pub enum TEMPLATES {
    #[default] CLASSIC,
    HEX,
}

impl TEMPLATES {
    pub fn get(&self) -> Template {
        match self {
            Self::CLASSIC => CLASSIC,
            Self::HEX => HEX,
        }
    }
}

impl World {
    /// Populates and returns new World instance with cubes, resulting in a
    /// map identical in shape and size to the original hex empire 1 world map.
    fn gen_water(&mut self) {
        self.iter_mut().for_each(|(_, tile)| {
            if random::<f32>() > 0.7 {
                tile.category = TileCategory::Water
            }
        })
    }

    fn gen_classic_shape(&mut self) {
        // layout.orientation = FLAT
        // layout.origin = (-10.0, 10.0)
        let width = 20;
        let height = 11;
        for q in 0..width {
            let q_offset = q >> 1;
            for r in (-1 * q_offset)..(height - q_offset) {
                self.insert(Cube::new(q,r), Tile{owner_index: None, category: TileCategory::Farmland, locality: None, army: None});
            }
        }
    }

    /// Populates and returns a new World instance with cubes forming a hexagonal shape.
    fn gen_hexagonal_shape(&mut self, radius: i32) {
        // align_hex_top_left(map_radius)
        for q in (-1 * radius)..(radius + 1) {
            let r1 = max(-radius, -q - radius);
            let r2 = min(radius, -q + radius);
            for r in r1..(r2 + 1) {
                self.insert(Cube::new(q,r), Tile{owner_index: None, category: TileCategory::Farmland, locality: None, army: None});
            }
        }
    }

    fn choose_shape_gen(&mut self, shape: &ShapeGen, init_layout: &crate::cubic::Layout<f32>) {
        match shape {
            ShapeGen::Classic => self.gen_classic_shape(),
            ShapeGen::Hexagonal(radius) => self.gen_hexagonal_shape(*radius),
            ShapeGen::Custom(shape) => {
                let my_shape_map = World::from_shape(shape, init_layout);
                self.gen_custom_shape(my_shape_map);
            }
        }
    }

    fn choose_river_gen(&mut self, river: &RiverGen, init_layout: &Layout<f32>) {
        let land_tiles: HashSet<&Cube<i32>> = self.world.iter().filter_map(|(c, t)| {
            if matches!(t.category, TileCategory::Farmland) {Some(c)} else {None}
        }).collect();
        match river {
            RiverGen::Random(ln, th) => {
                self.rivers = crate::river::generate_river(land_tiles, *ln, *th);
            }
            RiverGen::Custom(river) => {
                // TODO new algo:
                // iter line by line
                // round to find all cubes
                // draw some line along the cube path

                // let x_min = river.iter().fold(f32::NAN, |a, &b| a.min(b.1));
                // let y_min = river.iter().fold(f32::NAN, |a, &b| a.min(b.2));
                // river = river.iter().map(|(idx, x, y)| (*idx, x - x_min, y - y_min)).collect();

                // 1. construct cube paths
                let mut rivers_sets: HashMap<usize, HashMap<Cube<i32>, usize>> = HashMap::new();
                // let mut river_line: HashMap<Cube<i32>, usize> = HashMap::new();
                for (cube_idx, (id, x, y)) in river.iter().enumerate() {
                    let cube = pixel_to_cube(&init_layout, [*x, *y]);
                    let origin: Cube<i32> = cube.round();
                    rivers_sets.entry(*id).or_insert(HashMap::new()).insert(origin, cube_idx);
                    // river_line.insert(origin, cube_idx);
                }

                // 2. sort cube paths by insertion order
                let mut rivers: Vec<Vec<Cube<i32>>> = Vec::new();
                for river_line in rivers_sets.values_mut() {
                    let mut line: Vec<(&Cube<i32>, &usize)> = river_line.iter().collect();
                    line.sort_by(|(c1, i1), (c2, i2)| i1.cmp(i2));
                    let line: Vec<Cube<i32>> = line.iter().map(|(c, i)| **c).collect();
                    rivers.push(line);
                }

                // 3. construct segment lines
                for river_line in rivers {
                    println!("river_line: {:?}", river_line);
                    // let mut prev_cube = river_line[0];
                    let mut rivers_segments: Vec<CubeSide> = Vec::new();
                    let mut origin_cube = Cube::new(0,0); // initialised to satisfy borrowck
                    for idx in 0..(river_line.len() - 1) {
                        // let idx_plus_one = if idx < river_line.len() {idx + 1} else {idx};
                        let cube = river_line[idx];
                        let next_cube = river_line[idx + 1];
                        let dir = next_cube - cube;
                        println!("cube: {:}, next_cube: {:}, dir: {:}", cube, next_cube, dir);
                        let dir_idx = DIRECTIONS.iter().position(|c| c == &dir).unwrap_or_else(|| 0);
                        let mut dir_iter = DIRECTIONS.iter().cycle().skip(dir_idx);

                        if idx == 0 {
                            origin_cube = cube;
                            let start_dir = dir_iter.next().unwrap();
                            rivers_segments.push(CubeSide::from(Cube::<f32>::from(cube) + (*start_dir/2)));
                            // let start_dir = DIRECTIONS[(dir_idx + 1) % 6];
                            // rivers_segments.push(CubeSide::from(Cube::<f32>::from(cube) + (start_dir/2)));
                        }

                        // let start_seg_dir = Cube::<i32>::from((Cube::<f32>::from(rivers_segments.last().unwrap()) - Cube::<f32>::from(cube)) * 2.);
                        // let seg_dir = start_seg_dir
                        // while this_seg_dir != start_seg_dir {
                        //     let value = CubeSide::from(Cube::<f32>::from(cube) + (*this_seg_dir / 2));
                        //     rivers_segments.push(value);
                        // }

                        let mut seg_dir_idx = (dir_idx + 1) % 6;
                        // let mut prev_cube = river_line[max(0, (idx as i32 - 1)) as usize];
                        while seg_dir_idx != dir_idx {
                            let segment = rivers_segments.last().unwrap();
                            // if idx > 0 {prev_cube = river_line[idx - 1]}

                            // println!("segment: {:}, cube: {:}, origin_cube: {:}", segment, cube, origin_cube);
                            // println!("segment - cube: {:}", *segment - cube);
                            // println!("(segment - cube) * 2: {:}", (*segment - origin_cube) * 2);
                            // let seg_dir = ((*segment - origin_cube) * 2).int();
                            let seg_dir: Cube<i32> = ((Cube::<f32>::from(*segment) - Cube::<f32>::from(origin_cube)) * 2.).round();


                            // let seg_dir: Cube<i32> = (Cube::<f32>::from(*segment - prev_cube) * 2.).round();
                            println!("seg_dir: {:}", seg_dir);
                            seg_dir_idx = DIRECTIONS.iter().position(|c| c == &seg_dir).unwrap();
                            let next_dir = DIRECTIONS.iter().cycle().skip((seg_dir_idx + 1)).next().unwrap();
                            println!("next_dir: {:}",next_dir);
                            let value = CubeSide::from(Cube::<f32>::from(cube) + (*next_dir / 2));
                            println!("value: {:}", value);
                            rivers_segments.push(value);
                            origin_cube = cube;
                            // print!("rs: {:?}", rivers_segments);
                            // self.rivers.insert()
                        }


                        // let mut seg_dir_idx = (dir_idx + 1) % 6;//(dir_idx + 1) % 6;
                        // let mut prev_cube = river_line[max(0, (idx as i32 - 1)) as usize];
                        // while seg_dir_idx != dir_idx {
                        //     let segment = rivers_segments.last().unwrap();
                        //     if idx > 0 {prev_cube = river_line[idx - 1]}
                        //     let seg_dir = ((*segment - prev_cube) * 2).int();
                        //     // let seg_dir: Cube<i32> = (Cube::<f32>::from(*segment - prev_cube) * 2.).round();
                        //     println!("seg_dir: {:}",seg_dir);
                        //     seg_dir_idx = DIRECTIONS.iter().position(|c| c == &seg_dir).unwrap();
                        //     let next_dir = DIRECTIONS.iter().cycle().skip(seg_dir_idx + 1).next().unwrap();
                        //     println!("next_dir: {:}",next_dir);
                        //     let value = CubeSide::from(Cube::<f32>::from(cube) + (*next_dir / 2));
                        //     rivers_segments.push(value);
                        //     // self.rivers.insert()
                        // }

                    }
                    self.rivers.extend(rivers_segments);
                }

                // let mut counts: HashMap<CubeSide, usize> = HashMap::new();
                // self.rivers = HashSet::new();

                // let x_min = river.iter().fold(f32::NAN, |a, &b| a.min(b.0));
                // let y_min = river.iter().fold(f32::NAN, |a, &b| a.min(b.1));
                // let river_2: Vec<_> = river.iter().map(|(x, y)| (x - x_min, y - y_min)).collect();

                // let (mut x_factor, mut y_factor) = (1.5, f32::sqrt(3.));
                // if matches!(init_layout.orientation, OrientationKind::Pointy(_)) {
                //     (x_factor, y_factor) = (y_factor, x_factor);
                // }
                // for (mut x, mut y) in river_2 {
                //     // x = x / (init_layout.size[0] * x_factor);
                //     // y = y / (init_layout.size[1] * y_factor);
                //     let cube = pixel_to_cube(&init_layout, [x, y]);
                //     let origin: Cube<i32> = cube.round();

                //     let corners = Cube::<f32>::from(origin).corners(&init_layout);
                //     let mut corners_2 = corners.clone();
                //     corners_2.rotate_left(1);
                //     let segments: Vec<_> = corners.iter().zip(corners_2.iter()).collect();
                //     let midpoints: Vec<_> = segments.iter().map(|(p1 , p2)| (**p1 + **p2) / 2.).collect();
                //     let diffs: Vec<_> = midpoints.iter().map(|p| (x - p.0).powf(2.) + (y - p.1).powf(2.)).collect();

                //     let idx = diffs.iter()
                //         .enumerate()
                //         .max_by(|(_, a), (_, b)| a.total_cmp(b))
                //         .map(|(index, _)| index)
                //         .unwrap();

                //     let dir = DIRECTIONS[idx];
                //     let segment = CubeSide::from(Cube::<f32>::from(origin) + dir/2);
                //     let count = counts.entry(segment).or_insert(0);
                //     *count += 1;
                //     // self.rivers.insert(segment);

                // }
                // self.rivers = counts.iter().filter_map(|(k, v)| if *v > 0 {Some(*k)} else {None}).collect();
                println!("rivers: {:?}", self.rivers);
            }
        }
    }

    fn gen_random_localities(&mut self, locality_names: &mut Vec<&str>) {
        let mut rng = rand::thread_rng();
        let amount = self.len() / 10;
        let random_positions = sample(&mut rng, self.len(), amount);
        // let mut categories: Vec<LocalityCategory> = Vec::new();

        for (i, world_index) in random_positions.into_iter().enumerate() {
            let cube = self.keys().skip(world_index).next().unwrap();
            let neighbours = cube.disc(1);
            let mut category = LocalityCategory::City;
            for n in neighbours {
                // let mut category: LocalityCategory = LocalityCategory::City;
                if self.get(&n).is_some_and(|t| matches!(t.category, TileCategory::Water)) {
                    // 50% chance to turn city into portcity
                    let roll = rng.gen::<f32>();
                    if roll > 0.5 {
                        category = LocalityCategory::PortCity;
                    }
                    
                    // categories.push(category);
                    break;
                }
                // categories.push(category);
            }

            // 10% chance to turn into airport
            let roll = rng.gen::<f32>();
            if roll > 0.9 {
                category = LocalityCategory::Airport;
            }

            let tile = self.values_mut().skip(world_index).next().unwrap();
            if !matches!(tile.category, TileCategory::Water) {
                tile.locality = Some(Locality::new(locality_names.pop().unwrap_or(&"city"), category))//categories.remove(i)))
            }
        }

        // for (world_index, locality_name) in random_positions.into_iter().zip(locality_names.into_iter().cycle()) {
        //     let tile = self.values_mut().skip(world_index).next().unwrap();
        //     tile.category = categories[i]
        // }

        // for ((cube, tile), locality_name) in self.iter().zip(locality_names.into_iter().cycle()) {
        //     if !matches!(tile.category, TileCategory::Water) && rand::thread_rng().gen::<f32>() > 0.9 {
        //         // check for an adjacent water tile
        //         let mut category = LocalityCategory::City;
        //         for neighbour in cube.disc(1) {
        //             if self.get(&neighbour).is_some_and(|tile| matches!(tile.category, TileCategory::Water)) {
        //                 category = LocalityCategory::PortCity;
        //                 break;
        //             }
        //         }
        //         tile.locality = Some(Locality::new(locality_name, category))
        //     }
        // }
    }

    fn gen_random_localities_with_ots(&mut self, locality_names: &mut Vec<&str>) {
        unimplemented!();
    }
    /// Same as localgen_random(), but ensures there is one tile of space between every locality.
    // get bounds, calc legal cubes, 
    // fn gen_random_localities_with_ots(&mut self, locality_names: Vec<&str>) {
    //     for (cube, locality_name) in self.keys().zip(locality_names) {
    //         let mut tile = self.remove(cube).unwrap();
    //         let mut flag = true;
    //         if tile.locality.is_some() {
    //             flag = false;
    //         }
    //         for neighbour in cube.disc(1) {
    //             if let Some(tile) = self.get(&neighbour) {
    //                 if tile.locality.is_some() {
    //                     flag = false;
    //                     break;
    //                 }
    //             }
    //         }
    //         if flag && rand::thread_rng().gen::<f32>() > 0.9 {
    //             tile.locality = Some(Locality::new(locality_name, LocalityCategory::City));
    //         }
    //     }
    // }

    fn choose_localities_gen(&mut self, gen: &LocalitiesGen, locality_names: &mut Vec<&str>) {
        match gen {
            LocalitiesGen::Random => self.gen_random_localities(locality_names),
            LocalitiesGen::RandomOts => self.gen_random_localities_with_ots(locality_names),
        }
    }

    /// Spawn positions hardcoded to correspond to the original hex empire 1 spawn positions.
    fn gen_classic_capitals(&mut self, locality_names: &mut Vec<&str>) {
        let starting_positions = [
            Cube::new(1, 1),
            Cube::new(1, 9),
            Cube::new(18, -8),
            Cube::new(18, 0),
        ];

        for (index, ((pos), locality_name)) in starting_positions.iter().zip(locality_names.iter()).enumerate() {
            let mut tile = self.get_mut(&pos).unwrap();
            tile.category = TileCategory::Farmland;
            tile.owner_index = Some(index);
            tile.locality = Some(Locality::new(locality_name, LocalityCategory::Capital(index)));

            let set = self.cubes_by_ownership.entry(index).or_insert(HashSet::new());
            set.insert(*pos);

            // extend_borders(self, &pos);
            for neighbour in pos.disc(1) {
                if let Some(tile) = self.get_mut(&neighbour) {
                    tile.category = TileCategory::Farmland;
                    tile.owner_index = Some(index);
                }
            }
        }
    }
    // def gen_random_capitals(filled_world, players):
    //     for player in players:
    //         starting_cube = random.choice(list(filled_world.keys()))
    //         starting_tile = filled_world.get(starting_cube)
    //         starting_tile.owner = player
    //         starting_tile.locality = Locality(data.choose_random_city_name(), "Capital")
    //         player.starting_cube = starting_cube
    // fn gen_random_capitals(&mut self, locality_names: &mut Vec<&str>, mut players: &mut Vec<Player>) {
    //     let start_pos = sample(&mut rand::thread_rng(), self.len(), players.len());
    //     players.iter_mut().enumerate().for_each(|(player_index, player)| {
    //         let index = start_pos.index(player_index);
    //         let cube = self.keys().skip(index).next().unwrap().clone();
    //         let tile = self.get_mut(&cube).unwrap();
    //         tile.category = TileCategory::Farmland;
    //         tile.owner_index = Some(player_index);
    //         tile.locality = Some(Locality::new("", LocalityCategory::Capital));
    //         player.capital_pos = Some(cube);
    //         let set = self.cubes_by_ownership.entry(player_index).or_insert(HashSet::new());
    //         set.insert(cube);
    //     });
    // }
    /// pick a random city for each player and turn it into their capital
    fn gen_random_capitals(&mut self, locality_names: &mut Vec<&str>, player_n: usize) {
        let cubes_with_cities = self.get_cubes_with_cities();
        for n in 0..player_n {
            self.gen_random_capital(n, &cubes_with_cities);
        }
    }
    /// pick a random city for a player and turn it into their capital
    pub fn gen_random_capital(&mut self, player_idx: usize, cubes_with_cities: &HashSet<Cube<i32>>) -> Cube<i32> {
        let mut rng = rand::thread_rng();
        let cube = cubes_with_cities.iter().choose(&mut rng).unwrap().clone();
        self.gen_capital_at_cube(player_idx, cube)
    }
    pub fn gen_capital_at_cube(&mut self, player_idx: usize, cube: Cube<i32>) -> Cube<i32> {
        let tile = self.get_mut(&cube).unwrap();
        tile.owner_index = Some(player_idx);
        tile.locality.as_mut().unwrap().category = LocalityCategory::Capital(player_idx);
        let set = self.cubes_by_ownership.entry(player_idx).or_insert(HashSet::new());
        set.insert(cube);
        cube
    }
    fn gen_maxdist_capitals(&mut self, locality_names: &mut Vec<&str>, player_n: usize) {
        unimplemented!()
    }

    fn choose_capitals_gen(&mut self, gen: &CapitalsGen, locality_names: &mut Vec<&str>) {
        match gen {
            CapitalsGen::Classic => self.gen_classic_capitals(locality_names),
            CapitalsGen::Random(n) => self.gen_random_capitals(locality_names, *n),
            CapitalsGen::MaxDist(n) => self.gen_maxdist_capitals(locality_names, *n),
        }
    }

    pub fn generate(
        &mut self,
        shape_gen: ShapeGen,
        river_gen: RiverGen,
        localities_gen: LocalitiesGen,
        capitals_gen: CapitalsGen,
        locality_names: &mut Vec<&str>,
        init_layout: &crate::cubic::Layout<f32>,
    ) {
        self.choose_shape_gen(&shape_gen, init_layout);
        // self.gen_water();
        self.choose_river_gen(&river_gen, init_layout);
        self.choose_localities_gen(&localities_gen, locality_names);
        self.choose_capitals_gen(&capitals_gen, locality_names);
    }

    pub fn generate_from_template(
        &mut self,
        template: &Template,
        locality_names: &mut Vec<&str>,
        init_layout: &crate::cubic::Layout<f32>,
    ) {
        match &template.shape {
            Some(shape) => self.choose_shape_gen(shape, init_layout),
            None => {}
        }

        match &template.rivers {
            Some(rivers) => self.choose_river_gen(rivers, init_layout),
            None => {}
        }

        match &template.localities {
            Some(localities) => self.choose_localities_gen(localities, locality_names),
            None => {}
        }

        match &template.capitals {
            Some(capitals) => self.choose_capitals_gen(capitals, locality_names),
            None => {}
        }
    }
}

impl World {
    //' A regular hexagonal grid is drawn over the shape.
    //' If the center of a cell falls inside the shape, it's included in the map.
    //' The grid is then moved around so as to minimise certain metrics, with the aim of
    //' maximising the resultant shape's resemblance to the input shape.
    fn from_shape(mut shape: &Vec<(f32, f32)>, layout: &crate::cubic::Layout<f32>) -> HashSet<Cube<i32>> {
        // use fold if working w floats: https://stackoverflow.com/questions/28446632/how-do-i-get-the-minimum-or-maximum-value-of-an-iterator-containing-floating-poi
        // let x_max = shape.iter().map(|p| p.0).max().unwrap();
        // let x_min = shape.iter().map(|p| p.0).min().unwrap();
        // let y_max = shape.iter().map(|p| p.1).max().unwrap();
        // let y_min = shape.iter().map(|p| p.1).min().unwrap();
        // let bounding_box = [(x_min, y_min), (x_max, y_max)];
        // shape = shape.iter().map(|(x, y)| (x - layout.size[0], y - layout.size[1])).collect();

        let x_min = shape.iter().fold(f32::NAN, |a, &b| a.min(b.0));
        let y_min = shape.iter().fold(f32::NAN, |a, &b| a.min(b.1));
        // shape = shape.iter().map(|(x, y)| (x - x_min, y - y_min)).collect();
        let x_max = shape.iter().fold(f32::NAN, |a, &b| a.max(b.0));
        let y_max = shape.iter().fold(f32::NAN, |a, &b| a.max(b.1));
        let bounding_box = [(x_min, y_min), (x_max, y_max)];
        println!("bounding box: {:?}", bounding_box);

        let (mut x_factor, mut y_factor) = (1.5, f32::sqrt(3.));
        if matches!(layout.orientation, OrientationKind::Pointy(_)) {
            (x_factor, y_factor) = (y_factor, x_factor);
        }

        let width = ((x_max - x_min) / (layout.size[0] * x_factor)).ceil() as i32;
        let height = ((y_max - y_min) / (layout.size[1] * y_factor)).ceil() as i32;
        println!("layout size: {:?}", layout.size);
        println!("width: {}, height: {}", width, height);
        // let bounding_box_c: Vec<Cube<i32>> = bounding_box.iter().map(|&(x, y)| crate::pixel_to_cube(&layout, [x, y]).round()).collect();
        // let start = -bounding_box_c[0].q();

        // convert to Cube coords at end?
        //let mut map : HashSet<(i32, i32)> = HashSet::new();
        // for dx in {bounding_box[0].0..bounding_box[1].0}.step_by(layout.size[0] as usize) {
        //     for dy in {bounding_box[0].1..bounding_box[1].1}.step_by(layout.size[1] as usize) {

        //     }
        // }
        // generate a map over the entire bounding box
        // let shape_cubic: HashSet<Cube<i32>> = shape.iter().map(|(x,y)| {
        //     pixel_to_cube(&layout, [*x, *y]).round()
        // }).collect();
        // let q_max = shape_cubic.iter().map(|p| p.q()).max().unwrap();
        // let q_min = shape_cubic.iter().map(|p| p.q()).min().unwrap();
        // let r_max = shape_cubic.iter().map(|p| p.r()).max().unwrap();
        // let r_min = shape_cubic.iter().map(|p| p.r()).min().unwrap();
        // let bounding_box = [(q_min, r_min), (q_max, r_max)];
        let mut map : HashSet<Cube<i32>> = HashSet::new();
        // layout.orientation = FLAT
        // layout.origin = (-10.0, 10.0)
        // let width = i32::abs_diff(q_max, q_min) as i32;//20;//(q_max - q_min).abs();
        // let height = r_max;//i32::abs_diff(r_max, r_min) as i32;//20;//(r_max - r_min).abs();
        // println!("{:?}", bounding_box);

        // // Define the vertices of your irregular polygon
        // let shape = vec![(0.0, 0.0), (4.0, 0.0), (3.0, 3.0), (1.0, 2.0)];

        // // Define the point to check
        // let x = 2.0;
        // let y = 1.0;

        // // Check if the point is inside the polygon
        // let is_inside = is_point_inside_polygon(&shape, x, y);

        // if is_inside {
        //     println!("Point ({}, {}) is inside the polygon.", x, y);
        // } else {
        //     println!("Point ({}, {}) is outside the polygon.", x, y);
        // }

        // let point = [324.0, -924.0];
        // let result = is_inside_polygon(&shape, point);
        // println!("Is the point inside the polygon? {}", result);

        match layout.orientation {
            OrientationKind::Flat(_) => {
                for q in 0..width {
                    let q_offset = q >> 1;
                    for r in (-1 * q_offset)..(height - q_offset) {
                        // map.insert(Cube::new(q,r));
                        let cube = Cube::new(q as f32,r as f32);
                        let mut pos = cube.to_pixel(layout);
                        pos.0 += x_min;
                        pos.1 += y_min;
                        if is_inside_polygon(&shape, pos) {
                            map.insert(Cube::new(q, r));
                        } else {
                            // println!{"not inside: {:?}, {:?}", cube, pos}
                        }
                    }
                }
            }
            OrientationKind::Pointy(_) => {
                for r in 0..height {
                    let r_offset = r >> 1;
                    for q in (-1 * r_offset)..(width - r_offset) {
                        map.insert(Cube::new(q,r));
                        let cube = Cube::new(q as f32,r as f32);
                        let mut pos = cube.to_pixel(layout);
                        // pos[0] += x_min;
                        // pos[1] += y_min;
                        // if is_inside_polygon(&shape, pos) {
                        //     map.insert(Cube::new(q, r));
                        // }
                    }
                }
            }
        }

        println!("map: {:?}", map);
        println!("map len: {}", map.len());
        map
        

        //overlapping_subset = 

        //let starting_point = bounding_box[0];
        //let 
    }

    fn gen_custom_shape(&mut self, shape: HashSet<Cube<i32>>) {
        shape.iter().for_each(|cube| self.insert(*cube, Tile::new(TileCategory::Farmland)));
    }
}

// fn is_inside_polygon(polygon: &Vec<(f32, f32)>, point: [f32; 2]) -> bool {
//     // let mut polygon = polygon.clone();
//     // polygon.reverse();
//     let mut count = 0;
//     let (px, py) = (point[0], point[1]);
//     let mut j = polygon.len() - 1; // Initialize j as the last vertex

//     // Loop through each edge of the polygon
//     for i in 0..polygon.len() {
//         let (x1, y1) = polygon[i];
//         let (x2, y2) = polygon[j];
        
//         // Check if the ray from point crosses this edge
//         if ((y1 > py) != (y2 > py)) && (px < (x2 - x1) * (py - y1) / (y2 - y1) + x1) {
//             count += 1;
//         }
        
//         j = i; // Update j
//     }

//     // Odd number of crossings means the point is inside the polygon
//     count % 2 == 1
// }

// fn is_inside_polygon(polygon: &Vec<(f32, f32)>, point: [f32; 2]) -> bool {
//     let (px, py) = (point[0], point[1]);
//     let mut inside = false;
//     let mut j = polygon.len() - 1;

//     for i in 0..polygon.len() {
//         let (xi, yi) = polygon[i];
//         let (xj, yj) = polygon[j];

//         if (yi < py && yj >= py || yj < py && yi >= py) && (xi <= px || xj <= px) {
//             if xi + (py - yi) / (yj - yi) * (xj - xi) < px {
//                 inside = !inside;
//             }
//         }

//         j = i;
//     }

//     inside
// }

fn is_inside_polygon(polygon: &Vec<(f32, f32)>, point: Pixel<f32>) -> bool {
    let (px, py) = (point.0, point.1);
    let mut inside = false;
    let mut j = polygon.len() - 1;

    for i in 0..polygon.len() {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];

        if (yi < py && yj >= py || yj < py && yi >= py) && (xi <= px || xj <= px) {
            if xi + (py - yi) / (yj - yi) * (xj - xi) < px {
                inside = !inside;
            }
        }

        j = i;
    }

    inside
}

// fn is_inside_polygon(polygon: &Vec<(f32, f32)>, point: [f32; 2]) -> bool {
//     let mut polygon = polygon.clone();
//     polygon.reverse();
//     let num_vertices = polygon.len();
//     if num_vertices < 3 {
//         return false; // A polygon with less than 3 vertices is not valid.
//     }

//     let mut inside = false;
//     let (x, y) = (point[0], point[1]);

//     // Iterate through each edge of the polygon.
//     for i in 0..num_vertices {
//         let j = (i + 1) % num_vertices; // Next vertex index.

//         let xi = polygon[i].0;
//         let yi = polygon[i].1;
//         let xj = polygon[j].0;
//         let yj = polygon[j].1;

//         // Check if the point is to the left of the edge.
//         // let intersect = ((yi > y) != (yj > y))
//         //     && (x < (xj - xi) * (y - yi) / (yj - yi) + xi);
//         let intersect = (yi >= y && yj < y) || (yj >= y && yi < y) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi);

//         if intersect {
//             inside = !inside;
//         }
//     }

//     return inside;
// }


fn is_point_inside_polygon(shape: &Vec<(f32, f32)>, point: [f32; 2]) -> bool {
    // let mut shape = shape.clone();
    // shape.reverse();
    let (x, y) = (point[0], point[1]);
    let num_vertices = shape.len();
    let mut inside = false;
    // let mut j = num_vertices - 1;

    for i in 0..num_vertices {
        let (xi, yi) = shape[i];
        let (xj, yj) = shape[(i + 1) % num_vertices];

        if f32::min(yi, yj) < y && y <= f32::max(yi, yj) && x <= f32::max(xi, xj) {
            let mut x_intersection = xi;
            if yi != yj {
                x_intersection = (y - yi) * (xj - xi) / (yj - yi) + xi;
            }
            if xi == xj || x <= x_intersection {
                inside = ! inside
            }
        }
        // if ((yi > y) != (yj > y))
        //     && (x < (xj - xi) * (y - yi) / (yj - yi) + xi)
        // {
        //     inside = !inside;
        // }

        // j = i;
    }

    inside
}

// def is_point_inside_polygon(x, y, polygon):
//     n = len(polygon)
//     inside = False

//     for i in range(n):
//         x1, y1 = polygon[i]
//         x2, y2 = polygon[(i + 1) % n]

//         if min(y1, y2) < y <= max(y1, y2) and x <= max(x1, x2):
//             if y1 != y2:
//                 x_intersection = (y - y1) * (x2 - x1) / (y2 - y1) + x1
//             if x1 == x2 or x <= x_intersection:
//                 inside = not inside

//     return inside