pub mod gen;
use crate::cubic;
use crate::ai;
use crate::cubic::Layout;
use crate::cubic::Pixel;

use cubic::Cube;
use cubic::DIRECTIONS;
use ai::AI;

use std::char::MAX;
use std::collections::HashSet;
use std::collections::HashMap;

use std::collections::VecDeque;
use std::ops::Deref;
use std::ops::DerefMut;

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

use std::cmp::max;
use std::cmp::min;

use serde::{Serialize, Deserialize};
use strum::EnumIter;

pub const ACTIONS_PER_TURN: i32 = 5;
const MAX_TRAVEL_DISTANCE: i32 = 2;
const EXTEND_BORDERS_DISTANCE: usize = 1;
pub const MAX_STACK_SIZE: i32 = 99;
const MORALE_BONUS_ANNEX_RURAL: i32 = 1;
const MORALE_BONUS_ANNEX_CITY_ORIGIN: i32 = 20;
const MORALE_BONUS_ANNEX_CITY_ALL: i32 = 10;
const MORALE_BONUS_ANNEX_SOVEREIGN_CAPITAL_ORIGIN: i32 = 80;
const MORALE_BONUS_ANNEX_SOVEREIGN_CAPITAL_ALL: i32 = 50;
const MORALE_PENALTY_LOSING_CITY: i32 = 10;
const MORALE_PENALTY_PER_MANPOWER_LOSING_BATTLE: f32 = 0.1;
const MORALE_PENALTY_IDLE_ARMY: i32 = 1;
const BASE_GROWTH_CITY: i32 = 5;
const BASE_GROWTH_CAPITAL: i32 = 10;
const BONUS_GROWTH_PER_TILE: i32 = 1;

// My own made up constants
const BASE_GROWTH_SATELLITE_CAPITAL: i32 = 7;
const MORALE_BONUS_ANNEX_SATELLITE_CAPITAL_ORIGIN: i32 = 40;
const MORALE_BONUS_ANNEX_SATELLITE_CAPITAL_ALL: i32 = 25;
// const UNIT_VIEW_DISTANCE: usize = 2; // for now the view distance will be the same as travel distance

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub from: Cube<i32>,
    pub to: Cube<i32>,
    pub via: Vec<Cube<i32>>,
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum Controller {
    Human,
    AI(AI),
    Remote,
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub name: String,
    pub actions: i32,
    pub controller: Controller,
    pub selection: Option<Cube<i32>>,

    // self.camera = None
    // self.color = color
    // self.is_defeated = False
}

impl Player {
    pub fn new(name: &str, controller: Controller) -> Self {
        Player {
            name: name.to_string(),
            actions: ACTIONS_PER_TURN,
            controller,
            selection: None,
        }
    }
    pub fn skip_turn(&mut self) {
        self.actions = 0;
    }
    // fn player_index(&self, world: &World) -> Option<usize> {
    //     for cube in self.cubes_owned.iter() {
    //         let first_tile = world.get(cube).unwrap();
    //         return first_tile.owner_index;
    //     }
    //     None
    // }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.name)
    }
}

#[derive(EnumIter, Serialize, Deserialize, Clone)]
pub enum LocalityCategory {
    City,
    PortCity,
    Capital(usize), // contains idx of original owner
    // SatelliteCapital,
    Airport,
}

// impl From<Locality> for LocalityCategory {
//     fn from(locality: Locality) -> Self {
//         locality.category
//     }
// }

impl From<LocalityCategory> for Locality {
    fn from(category: LocalityCategory) -> Self {
        Self::new("", category)
    }
}

impl Display for LocalityCategory {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            LocalityCategory::City => write!(f, "City"),
            LocalityCategory::PortCity => write!(f, "Port City"),
            LocalityCategory::Capital(i) => write!(f, "P{} Capital", i),
            //LocalityCategory::SatelliteCapital => write!(f, "Satellite Capital"),
            LocalityCategory::Airport => write!(f, "Airport"),
        }
        // write!(f, "({})", self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Locality {
    pub name: String,
    pub category: LocalityCategory, 
    starting_owner_index: Option<usize>,
}

impl Display for Locality {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({} of {})", self.category, self.name)
    }
}

impl Locality {
    pub fn new(name: &str, category: LocalityCategory) -> Self {
        Locality {
            name: name.to_string(),
            category: category,
            starting_owner_index: None,
        }
    }
}
#[derive(EnumIter, Serialize, Deserialize, Clone)]
pub enum TileCategory {
    Farmland,
    Water,
}

impl Display for TileCategory {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            TileCategory::Farmland => write!(f, "Farmland"),
            TileCategory::Water => write!(f, "Water"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Tile {
    pub owner_index: Option<usize>,
    pub category: TileCategory,
    pub locality: Option<Locality>,
    pub army: Option<Army>,
}

// impl From<Tile> for TileCategory {
//     fn from(tile: Tile) -> Self {
//         tile.category
//     }
// }

impl From<TileCategory> for Tile {
    fn from(category: TileCategory) -> Self {
        Tile::new(category)
    }
}


impl Tile {
    pub fn new(category: TileCategory) -> Self {
        Tile {
            owner_index: None,
            category: category,
            locality: None,
            army: None,
        }
    }

    pub fn is_capturable(&self) -> bool {
        match self.category {
            TileCategory::Water => false,
            TileCategory::Farmland => true,
        }
    }
}

impl Tile {
    pub fn owner<'a>(&self, players: &'a mut Vec<Player>) -> Option<&'a mut Player> {
        match self.owner_index {
            Some(index) => Some(&mut players[index]),
            None => None,
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter) -> Result {
        //     if self.locality:
        //         string = str(self.locality.name)
        //     else:
        //         string = str(self.category)
        //     return string
        // write!(f, "({})", self.category)
        if self.locality.is_some() {
            write!(f, "{}", self.locality.as_ref().unwrap())
        }
        else {
            write!(f, "{}", self.category)
        }
    }
}

// impl<'a> FromIterator<&'a Tile> for [&'a Tile; 6] {
//     fn from_iter<I: IntoIterator<Item=&'a Tile>>(iter: I) -> Self {
//         let mut arr = [&Tile{owner_index: 0, category: "".to_string(), army: None}; 6];

//         let mut i = 0;
//         for elem in iter {
//             arr[i] = elem;
//             i += 1;
//         }
//         // for i in iter {
//         //     c.add(i);
//         // }

//         arr
//     }
// }

// Players interact with the game world by issuing commands to tiles containing an army,
// effectively moving armies across tiles.
#[derive(Debug, Serialize, Deserialize)]
pub struct Army {
    pub manpower: i32,
    pub morale: i32,
    pub owner_index: Option<usize>,
    pub can_move: bool,
}

impl Army {
    pub fn new(manpower: i32, owner_index: Option<usize>) -> Self {
        Army {
            manpower,
            morale: (manpower as f32 / 2.).round() as i32,
            owner_index,
            can_move: true,
        }
    }
    // Grow the army up to MAX_STACK_SIZE, and return any growth overflow.
    pub fn grow(&mut self, manpower: i32) -> i32 {
        let growth_capacity = MAX_STACK_SIZE - self.manpower;
        let actual_growth = min(growth_capacity, manpower);
        self.manpower += actual_growth;
        self.morale = min(MAX_STACK_SIZE, self.morale + (actual_growth as f32 /2.).round() as i32);
        manpower - actual_growth
        // min(self.manpower + manpower - MAX_STACK_SIZE, 0) == manpower - min(MAX_STACK_SIZE - self.manpower, manpower)
    }
    // fn update_morale() ?
    fn apply_morale_bonus(&mut self, bonus: i32) {
        assert!(bonus > 0);
        self.morale = min(self.manpower, self.morale + bonus)
    }
    fn apply_morale_penalty(&mut self, penalty: i32, total_manpower: i32) {
        assert!(penalty > 0);
        assert!(total_manpower >= 0); // sanity error catching, remove later
        let minimum_morale = min(self.manpower, total_manpower / 50);
        self.morale = max(minimum_morale, self.morale - penalty);
    }
    pub fn combat_strength(&self) -> i32 {
        self.manpower + self.morale
    }
}

impl Display for Army {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({}/{})", self.manpower, self.morale)
    }
}

// pub struct World(pub HashMap<Cube<i32>, Tile>);

// #[derive(Serialize)]
pub struct World {
    pub world: HashMap<Cube<i32>, Tile>,
    // #[serde(skip)]
    pub cubes_by_ownership: HashMap<usize, HashSet<Cube<i32>>>,
    // #[serde(skip)]
    pub cubes_with_airport: HashSet<Cube<i32>>,
    // #[serde(skip)]
    pub rivers: HashSet<crate::river::CubeSide>,
}

impl Serialize for World {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.world.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for World {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let world = HashMap::<Cube<i32>, Tile>::deserialize(deserializer)?;

        let mut cubes_by_ownership: HashMap<usize, HashSet<Cube<i32>>> = HashMap::new();
        let mut cubes_with_airport: HashSet<Cube<i32>> = HashSet::new();

        world.iter().for_each(|(cube, tile)| {
            if let Some(index) = tile.owner_index {
                let set = cubes_by_ownership.entry(index).or_insert(HashSet::new());
                set.insert(*cube);
            }
            if let Some(locality) = &tile.locality {
                if matches!(locality.category, LocalityCategory::Airport) {
                    cubes_with_airport.insert(*cube);
                }
            }
        });

        let rivers = HashSet::new();

        Ok(World {
            world,
            cubes_by_ownership,
            cubes_with_airport,
            rivers,
        })
    }
}

// impl Serialize for World {
//     fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         serializer.serialize(self.world)
//     }
// }

impl Display for World {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({} tile-sized world with {} players)", self.world.len(), self.cubes_by_ownership.len())
    }
}

impl Deref for World {
    type Target = HashMap<Cube<i32>, Tile>;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world
    }
}

impl World {
    pub fn new() -> Self {
        World {
            world: HashMap::new(),
            cubes_by_ownership: HashMap::new(),
            cubes_with_airport: HashSet::new(),
            rivers: HashSet::new(),
        }
    }
    fn handle_internal(&mut self, key: Cube<i32>, value: &Tile) {
        // let Some((key, value)) = self.world.get_key_value(&key);
        if let Some(index) = value.owner_index {
            let set = self.cubes_by_ownership.entry(index).or_insert(HashSet::new());
            set.insert(key);
        }
        if let Some(locality) = &value.locality {
            if matches!(locality.category, LocalityCategory::Airport) {
                self.cubes_with_airport.insert(key);
            }
        }
    }
    pub fn insert(&mut self, key: Cube<i32>, value: Tile) {
        self.handle_internal(key, &value);
        self.world.insert(key, value);
    }
    pub fn extend<T>(&mut self, mut iter: T)
    where
        T: Iterator<Item = (Cube<i32>, Tile)>,
    {
        iter.by_ref().for_each(|(k, v)| {
            self.handle_internal(k, &v);
        });
        self.world.extend(iter);
    }
    // pub fn extend(&mut self, iter: HashMap<Cube<i32>, Tile>,) {
    //     iter.iter().for_each(|(cube, tile)| {self.handle_internal(*cube, tile)});
    //     self.world.extend(iter)
    // }
    pub fn remove(&mut self, k: &Cube<i32>) -> Option<Tile> {
        let value = self.world.remove(k);
        if let Some(value) = &value {
            if let Some(index) = value.owner_index {
                let set = self.cubes_by_ownership.get_mut(&index).unwrap();
                set.remove(k);
            }
            if let Some(locality) = &value.locality {
                if matches!(locality.category, LocalityCategory::Airport) {
                    self.cubes_with_airport.remove(k);
                }
            }
        }
        value
    }
    pub fn get_cubes_with_cities(&self) -> HashSet<Cube<i32>> {
        let cubes_with_cities: HashSet<Cube<i32>> = self.iter().filter(|(c, t)| {
            t.locality.as_ref().is_some_and(|l| {
                matches!(l.category, LocalityCategory::City)
            })
        }).map(|(c, t)| *c).collect();
        cubes_with_cities
    }
    pub fn set_tile_owner(&mut self, cube: &Cube<i32>, new_index: usize) {
        let mut tile = self.remove(cube).unwrap();
        tile.owner_index = Some(new_index);
        self.insert(*cube, tile);
        // let tile = self.world.get(cube).unwrap();
        // let old_owner = tile.owner_index;
        // tile.owner_index = Some(new_index);
    }
    /// Issues an appropriate order to the origin tile,
    /// with the target tile as the order target.
    /// Returns a set of captured coordinates.
    /// This function is called from within the Player.click_on_tile() method,
    /// and the order to be issued is determined based on the following conditions:
    /// move_to() - the target tile has no army and belongs to the origin tile owner.
    /// capture_tile() - the target tile has no army.
    /// regroup() - the target tile has an allied army.
    /// attack() - the target tile has a hostile army.
    // pub fn execute_army_order(&mut self, origin_cube: &Cube<i32>, target_cube: &Cube<i32>) {
     pub fn execute_army_order(&mut self, command: &Command) {
        let origin_cube = &command.from;
        let target_cube = &command.to;
        let target = self.get(target_cube).unwrap();
        let origin_owner = self.get(origin_cube).unwrap().army.as_ref().unwrap().owner_index;
        let target_owner = self.get(target_cube).unwrap().owner_index;
        // let target_army = self.get(target_cube).unwrap().army.as_ref();
    
        let mut extend = true;
    
        // one can view move_to() as a special case of regroup(), as with attack() and capture()...
        match &target.army {
            Some(army) if target_owner == origin_owner => regroup(self, origin_cube, target_cube),
            Some(army) => { // attack
                let losing_player = attack(self, origin_cube, target_cube);
                if losing_player == origin_owner {
                    extend = false;
                }
            },
            None if origin_owner == target_owner => move_to(self, origin_cube, target_cube), // own empty
            None if origin_owner != target_owner => { // else's empty
                capture_tile(self, origin_cube, target_cube);
                move_to(self, origin_cube, target_cube);
            },
            _ => unreachable!(),
        }
    
        if extend {
            extend_borders(self, target_cube);
        }
    }

    pub fn breadth_first_search(&self, start: &Cube<i32>) {
        let mut frontier = VecDeque::from([*start]);
        let mut reached = HashSet::new();
        reached.insert(*start);

        while !frontier.is_empty() {
            let current = frontier.pop_front().unwrap();
            for next in current.disc(1) {
                if !reached.contains(&next) {
                    frontier.push_back(next);
                    reached.insert(next);
                }
            }
        }
    }

    pub fn breadth_first_search_path(&self, start: &Cube<i32>, goal: &Cube<i32>) {
        let mut frontier = VecDeque::from([*start]);
        let mut came_from = HashMap::new();
        came_from.insert(*start, None);

        while !frontier.is_empty() {
            let current = frontier.pop_front();
            for next in current.unwrap().disc(1) {
                if !came_from.contains_key(&next) {
                    frontier.push_back(next);
                    came_from.insert(next, current);
                }
            }
        }

        let mut current = *goal;
        let mut path = vec![];
        while current != *start {
            path.push(current);
            current = came_from[&current].unwrap();
        }
        path.push(*start); // optional
        path.reverse(); // optional
    }

    pub fn get_all_legal_moves(&self, origin: &Cube<i32>, owner_index: &usize) -> HashSet<Cube<i32>> {
        let mut legal_moves = HashSet::new();
        self.get_reachable_cubes(origin).iter().for_each(|target| {
            if self.is_cube_targetable(origin, &target) {
                legal_moves.insert(*target);
            }
        });

        if self[origin].locality.as_ref().is_some_and(|l| matches!(l.category, LocalityCategory::Airport)) {
            let airport_cubes = &self.cubes_with_airport;
            let empty_set = HashSet::new();
            let player_cubes = self.cubes_by_ownership.get(owner_index).unwrap_or(&empty_set);
            let owned_airport_cubes: HashSet<&Cube<i32>> = airport_cubes.intersection(player_cubes).collect();
            owned_airport_cubes.iter().for_each(|target| {
                legal_moves.insert(**target); // careful, this will insert origin if it contains an airport
            });
        }
        legal_moves.remove(origin);
        legal_moves
    }

    // can the cube be captured by border extension mechanic
    pub fn is_cube_extendable(&self, origin: &Cube<i32>, target: &Cube<i32>) -> bool {
        let origin_tile = self.get(origin).unwrap();
        match self.get(target) {
            Some(tile) => tile.army.is_none() 
                              && tile.locality.is_none()
                              && !matches!(tile.category, TileCategory::Water)
                              && !matches!(origin_tile.category, TileCategory::Water),
            None => false,
        }
    }

    pub fn is_cube_targetable(&self, origin: &Cube<i32>, target: &Cube<i32>) -> bool {
        let origin_t = self.get(origin).unwrap();
        match self.get(target) {
            Some(tile) => {
                match tile.category {
                    TileCategory::Water => {
                        matches!(origin_t.category, TileCategory::Water)
                        || {
                            origin_t.locality.as_ref().is_some_and(|l| matches!(l.category, LocalityCategory::PortCity))
                            // // is cube connected to other water tiles
                        }
                    },
                    _ => true,
                }
            },
            None => false
        }
    }

    pub fn is_cube_passable(&self, origin: &Cube<i32>, target: &Cube<i32>) -> bool {
        let origin_t = self.get(origin).unwrap();
        // let target_t = self.get(target);
        match self.get(target) {
            Some(tile) => {
                let cat_cond = match tile.category {
                    TileCategory::Water => {
                        origin_t.locality.as_ref().is_some_and(|l| matches!(l.category, LocalityCategory::PortCity)) ||
                        matches!(origin_t.category, TileCategory::Water)
                    },
                    _ => true,
                };
                let arm_cond = match &tile.army {
                    Some(army) => {
                        army.owner_index == origin_t.army.as_ref().unwrap().owner_index
                    },
                    None => true,
                };
                let loc_cond = match &tile.locality {
                    Some(_) => {
                        tile.owner_index == origin_t.owner_index
                    },
                    None => true,
                };

                cat_cond && arm_cond && loc_cond
            },
            None => false
        }
    }

    pub fn get_reachable_cubes(&self, start_cube: &Cube<i32>) -> HashSet<Cube<i32>> {
        let mut visited = HashSet::new();
        visited.insert(*start_cube);
        let mut fringes = vec!(); // vec of vecs of cubes
        //fringes.push(&vec!(start_cube));
        fringes.push(*start_cube);
        
        for _ in 0..(MAX_TRAVEL_DISTANCE as usize) {
            //fringes.push(&vec!());
            let mut next_fringes = vec!();
            for cube in fringes {
                for direction in DIRECTIONS {
                    let neighbour = cube + direction;
                    if !visited.contains(&neighbour) {
                        // This way we also add obstacles themselves, if they exist
                        if self.get(&neighbour).is_some() {
                            visited.insert(neighbour);
                        }
                        if self.is_cube_passable(start_cube, &neighbour) {
                            next_fringes.push(neighbour);
                        }
                    }
                }
            }
            fringes = next_fringes;
        }
        visited.remove(start_cube);
        visited
    }

    pub fn is_there_capturable_tile_within_range(&self, cube: &Cube<i32>) -> bool {
        let valid_targets = self.get_reachable_cubes(&cube);
        for target in valid_targets {
            let origin = self.get(cube).unwrap();
            let target = self.get(&target).unwrap();
            if origin.owner_index != target.owner_index {
                return true
            }
        }
        false
    }

    pub fn train_armies(&mut self, &player_index: &usize) {
        let (world, cubes_by_ownership) = self.split_fields();
        if !cubes_by_ownership.get(&player_index).is_some() {return} // TODO: make this redundant in the future
        let player_cubes = cubes_by_ownership.get(&player_index).unwrap();//.into_iter().flatten().collect::<HashSet<&Cube<i32>>>();
        let mut bonus_growth = player_cubes.len() as i32 * BONUS_GROWTH_PER_TILE;
        let mut locality_cubes: HashSet<Cube<i32>> = world.iter().filter(|(c, t)| t.locality.is_some()).map(|(c, t)| *c).collect();
        let player_cubes_w_locality: HashSet<Cube<i32>> = player_cubes.iter().filter_map(|v| locality_cubes.take(v)).collect(); // inplace intersetction

        // First apply base growth
        for cube in player_cubes_w_locality.iter() {
            let tile = world.get_mut(cube).unwrap();
            let growth = match &tile.locality {
                Some(locality) => match &locality.category {
                    LocalityCategory::City => BASE_GROWTH_CITY,
                    LocalityCategory::PortCity => continue,
                    LocalityCategory::Airport => continue,
                    LocalityCategory::Capital(i) => {
                        if *i == player_index {
                            BASE_GROWTH_CAPITAL
                        } else {
                            BASE_GROWTH_SATELLITE_CAPITAL
                        }
                    },
                    //LocalityCategory::SatelliteCapital => BASE_GROWTH_SATELLITE_CAPITAL,
                }
                _ => {continue},
            };
            match &mut tile.army {
                Some(army) => {
                    army.grow(growth);
                }
                None => {
                    tile.army = Some(Army::new(growth, tile.owner_index));
                }
            }
        }

        // Then apply bonus growth
        let army_cubes: HashSet<Cube<i32>> = world.iter().filter(|(_, t)| 
            t.army.as_ref().is_some_and(|a| 
                a.manpower < MAX_STACK_SIZE)).map(|(c, _)| *c).collect();
        let mut player_cubes_w_locality_n_army: HashSet<_> = player_cubes_w_locality.intersection(&army_cubes).collect();
        
        while player_cubes_w_locality_n_army.len() > 0 && 
              bonus_growth > player_cubes_w_locality_n_army.len() as i32 {
            let growth = bonus_growth / player_cubes_w_locality_n_army.len() as i32;
            let growth_remainder = bonus_growth % player_cubes_w_locality_n_army.len() as i32;
            let mut cubes_to_remove: Vec<&Cube<i32>> = vec!();
            let overflow = player_cubes_w_locality_n_army.iter().fold(
                0, |acc, c| {
                    let overflow = world.get_mut(c).unwrap().army.as_mut().unwrap().grow(growth);
                    if overflow > 0 {cubes_to_remove.push(&c)};
                    acc + overflow
                });
            bonus_growth = growth_remainder + overflow;
            for c in cubes_to_remove {
                player_cubes_w_locality_n_army.remove(c);
            }
        }
    }

    // Called from within Game at the end of the turn. Applies a morale penalty to idle armies.
    pub fn apply_idle_morale_penalty(&mut self, player_index: usize) {
        let empty_set = HashSet::new();
        let total_manpower = player_total_manpower(&self, player_index);
        let player_cubes = self.cubes_by_ownership.get(&player_index).unwrap_or(&empty_set);
    
        for cube in player_cubes {
            let tile = self.world.get_mut(cube).unwrap();
            if let Some(army) = &mut tile.army {
                if army.can_move {
                    army.apply_morale_penalty(MORALE_PENALTY_IDLE_ARMY, total_manpower);
                }
            }
        }
    }

    pub fn can_player_issue_a_command(&self, player_index: &usize) -> bool {
        self.cubes_by_ownership
            .get(player_index)
            .map_or(false, |cubes| {
                cubes.iter().any(|cube| {
                    self.world.get(cube).map_or(false, |tile| {
                        tile.army.as_ref().map_or(false, |army| army.can_move)
                    })
                })
            })
    }
    // Transfer the ownership of all of defeated_player tiles to player.
    pub fn surrender_to_player(&mut self, surrendering_player_index: &usize, conquering_player_index: &usize) {
        let (world, cubes_by_ownership) = self.split_fields();
        for cube in cubes_by_ownership.get(&surrendering_player_index).unwrap() {
            let tile = world.get_mut(cube).unwrap();
            tile.army = None;
            tile.owner_index = Some(*conquering_player_index);
        }
    }
    
    //Split struct fields into separate variables.
    pub fn split_fields(&mut self) -> (&mut HashMap<Cube<i32>, Tile>, &mut HashMap<usize, HashSet<Cube<i32>>>) {
        (&mut self.world, &mut self.cubes_by_ownership)
    }
}

/// Sets the owner of the nearest neighbours (NN) of the target tile,
/// to the owner of the origin tile, subject to conditions.
/// Conditions: The NN tile does not contain any armies or localities,
/// and does not already belong to origin.owner. Tile category is not water.
fn extend_borders(mut world: &mut World, target_cube: &Cube<i32>) {
    // Origin has now captured the target and so is irrelevant
    let target_owner_index = world.get(&target_cube).unwrap().owner_index;
    let neighbours_cube = target_cube.disc(EXTEND_BORDERS_DISTANCE); // Find the NN of the target cube
    for cube in neighbours_cube {
        if let Some(tile) = world.get(&cube) {
            if world.is_cube_extendable(&target_cube, &cube) && tile.owner_index != target_owner_index {
                capture_tile(&mut world, &target_cube, &cube);
            }
        }
    }
}

// Moves the origin tile army to the target tile.
fn move_to(mut world: &mut World, origin_cube: &Cube<i32>, target_cube: &Cube<i32>) {
    let mut origin = world.remove(origin_cube).unwrap();
    let mut target = world.get_mut(target_cube).unwrap();
    target.army = origin.army;
    target.army.as_mut().unwrap().can_move = false;
    origin.army = None;
    world.insert(*origin_cube, origin);
}

// Combines the origin tile army with an allied target tile army.
fn regroup(mut world: &mut World, origin_cube: &Cube<i32>, target_cube: &Cube<i32>) {
    let mut origin = world.remove(origin_cube).unwrap();
    let mut target = world.get_mut(target_cube).unwrap();
    let mut origin_army = origin.army.as_mut().unwrap();
    let mut target_army = target.army.as_mut().unwrap();

    let total_manpower = origin_army.manpower + target_army.manpower;
    let army_over_max_stack = total_manpower - MAX_STACK_SIZE;
    if army_over_max_stack <= 0 {
        target_army.manpower = total_manpower;
        target_army.morale = ((origin_army.morale + target_army.morale) as f32 / 2.).round() as i32;
        origin.army = None;
    } else {
        let origin_morale_per_manpower = origin_army.morale as f32 / origin_army.manpower as f32;
        target_army.manpower += origin_army.manpower - army_over_max_stack;
        target_army.morale = ((origin_army.morale as f32 - (army_over_max_stack as f32 * origin_morale_per_manpower) + target_army.morale as f32) / 2.).round() as i32;
        origin_army.manpower = army_over_max_stack;
        origin_army.morale = (army_over_max_stack as f32 * origin_morale_per_manpower).round() as i32;
    }
    target_army.can_move = false;

    world.insert(*origin_cube, origin);
}

/// Attacks the target tile from the origin tile.
fn attack(mut world: &mut World, origin_cube: &Cube<i32>, target_cube: &Cube<i32>) -> Option<usize> {
    let mut origin = world.remove(origin_cube).unwrap();
    let mut target = world.remove(target_cube).unwrap();

    println!("{:?} attacks {:?} with {} against {}", origin.owner_index, target.owner_index, origin.army.as_ref().unwrap(), target.army.as_ref().unwrap());

    origin.army.as_mut().unwrap().can_move = false;

    let mut manpower_lost = 0;
    let mut losing_player = None;

    let diff = origin.army.as_ref().unwrap().combat_strength() - target.army.as_ref().unwrap().combat_strength();

    let combat_strength_to_army = (diff as f32/2.).ceil() as i32;

    if diff > 0 {
        losing_player = target.army.as_ref().unwrap().owner_index;
        manpower_lost = target.army.as_ref().unwrap().manpower;
        target.army = None;
        origin.army.as_mut().unwrap().manpower = combat_strength_to_army;
        origin.army.as_mut().unwrap().morale = combat_strength_to_army;
    } else {
        losing_player = origin.army.as_ref().unwrap().owner_index;
        manpower_lost = origin.army.as_ref().unwrap().manpower;
        origin.army = None;
        target.army.as_mut().unwrap().manpower = max(1, -combat_strength_to_army);
        target.army.as_mut().unwrap().morale = max(1, -combat_strength_to_army);
    }

    world.insert(*origin_cube, origin);
    world.insert(*target_cube, target);
    if diff > 0 {
        capture_tile(&mut world, origin_cube, target_cube);
        move_to(&mut world, origin_cube, target_cube);
    }
    apply_morale_penalty_losing_combat(&mut world, losing_player.unwrap(), manpower_lost);
    losing_player
}

/// Calculates the minimum morale value an army can have.
/// If you use this function you have to remember to still account for the fact
/// that an army's morale shouldn't exceed its manpower.
fn player_total_manpower(world: &World, player_index: usize) -> i32 {
    // let mut total_manpower = 0;
    // let player_cubes = world.cubes_by_ownership.get(&player_index).unwrap();
    // for cube in player_cubes.iter() {
    //     let tile = world.get(&cube).unwrap();
    //     if tile.owner_index == Some(player_index) {
    //         if let Some(army) = &tile.army {
    //             total_manpower += army.manpower;
    //         }
    //     }
    // }
    // total_manpower

    world.cubes_by_ownership.get(&player_index).map_or(0, |cubes| {
        cubes.iter().map(|c| 
            world.get(c).unwrap().army.as_ref().map_or(0, |a| a.manpower)
        ).sum()
    })
}

/// Calculates and applies the morale penalty to every army of the losing player.
fn apply_morale_penalty_losing_combat(mut world: &mut World, losing_player_index: usize, manpower_lost: i32) {
    let penalty = (MORALE_PENALTY_PER_MANPOWER_LOSING_BATTLE * manpower_lost as f32) as i32; // implicit floor
    if penalty == 0 { return }
    let total_manpower = player_total_manpower(&world, losing_player_index);
    println!("Player {:?} suffers {} morale penalty", losing_player_index, penalty);
    for tile in world.values_mut() {
        if tile.owner_index == Some(losing_player_index) {
            if let Some(army) = tile.army.as_mut() {
                army.apply_morale_penalty(penalty, total_manpower);
            }
        }
    }
}

/// Change the owner of the target tile to that of the origin tile,
/// and apply appropriate morale modifiers to the owners of those tiles.
fn capture_tile(world: &mut World, origin_cube: &Cube<i32>, target_cube: &Cube<i32>) {
// fn capture_tile(mut game_world_tiles: &mut hash_map::ValuesMut<Cube<i32>, Tile>, mut origin: &mut Tile, mut target: &mut Tile) {

    let mut target = world.remove(target_cube).unwrap();

    if !target.is_capturable() {
        world.insert(*target_cube, target);
        return
    }

    let mut origin = world.remove(origin_cube).unwrap();

    let origin_owner_idx = origin.army.as_ref().unwrap().owner_index.unwrap();
    let origin_owner = format!("Player {}", origin_owner_idx);
    let from_clause = match target.owner_index {
        Some(index) => format!(" from Player {}", index),
        None => "".to_string(),
    };
    println!("{} captures {} {}{}", origin_owner, target, target_cube, from_clause);

    let mut capturing_army_morale_bonus = 0;
    let mut origin_owner_morale_bonus = 0;
    let mut target_owner_morale_penalty = None;
    let mut target_total_manpower = -1; // this needs to get overwritten

    // Calculate morale bonus/penalty
    // let target = world.get_mut(target_cube).unwrap();
    match &target.locality {
        Some(locality) => match &locality.category {
            LocalityCategory::Capital(i) => {
                if *i == origin_owner_idx {
                    capturing_army_morale_bonus = MORALE_BONUS_ANNEX_SOVEREIGN_CAPITAL_ORIGIN;
                    origin_owner_morale_bonus = MORALE_BONUS_ANNEX_SOVEREIGN_CAPITAL_ALL;
                } else {
                    capturing_army_morale_bonus = MORALE_BONUS_ANNEX_SATELLITE_CAPITAL_ORIGIN;
                    origin_owner_morale_bonus = MORALE_BONUS_ANNEX_SATELLITE_CAPITAL_ALL;
                }
            }
            // LocalityCategory::SatelliteCapital => {
            //     capturing_army_morale_bonus = MORALE_BONUS_ANNEX_SATELLITE_CAPITAL_ORIGIN;
            //     origin_owner_morale_bonus = MORALE_BONUS_ANNEX_SATELLITE_CAPITAL_ALL;
            // }
            LocalityCategory::City | LocalityCategory::PortCity | LocalityCategory::Airport => {
                capturing_army_morale_bonus = MORALE_BONUS_ANNEX_CITY_ORIGIN;
                origin_owner_morale_bonus = MORALE_BONUS_ANNEX_CITY_ALL;
                if let Some(index) = target.owner_index {
                    target_total_manpower = player_total_manpower(&world, index);
                    target_owner_morale_penalty = Some(MORALE_PENALTY_LOSING_CITY);
                }
            }
        }
        None if matches!(target.category, TileCategory::Farmland) => {
            capturing_army_morale_bonus = MORALE_BONUS_ANNEX_RURAL;
            origin_owner_morale_bonus = MORALE_BONUS_ANNEX_RURAL;
        },
        _ => unreachable!(),
    };

    // Apply morale bonus/penalty
    // ... to the capturing army:
    origin.army.as_mut().unwrap().apply_morale_bonus(capturing_army_morale_bonus);

    // ... to other armies:
    for tile in world.values_mut() {
        if tile.army.is_some() {
            if tile.owner_index == origin.owner_index {
                tile.army.as_mut().unwrap().apply_morale_bonus(origin_owner_morale_bonus);
            } else if tile.owner_index == target.owner_index {
                if let Some(penalty) = target_owner_morale_penalty {
                    tile.army.as_mut().unwrap().apply_morale_penalty(penalty, target_total_manpower);
                }
            }
        }
    }
    
    // Actually capture the tile
    target.owner_index = origin.army.as_ref().unwrap().owner_index;
    world.insert(*origin_cube, origin);
    world.insert(*target_cube, target);
    // move_to(&mut world, origin_cube, target_cube);
}

/// Returns (min_x, min_y, max_x, max_y) of all tiles in screen space
pub fn get_world_bounds(world: &World, layout: &Layout<f32>) -> (f32, f32, f32, f32) {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for (cube, _) in world.world.iter() {
        let Pixel(x, y) = Cube::<f32>::from(*cube).to_pixel(layout);
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    (min_x, min_y, max_x, max_y)
}
