/// The AI module

use std::cmp::Reverse;
use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::Cube;
use crate::Tile;
use crate::World;
use crate::TileCategory;
use crate::LocalityCategory;

// enum SCORES {
//     Farmland(i32),
//     City(i32),
//     CapitalSatellite(i32),
//     Capital(i32),
//     Manpower(i32),
// }

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Scores {
    manpower: i32,
    water: i32,
    farmland: i32,
    city: i32,
    port_city: i32,
    airport: i32,
    capital: i32,
    satellite_capital: i32,
}

#[derive(Debug)]
pub struct ScoredMove {
    score: i32,
    pub origin: Cube<i32>,
    pub target: Cube<i32>,
}

pub const DEFAULT_SCORES: Scores = Scores {
    water: 0,
    manpower: 1,
    farmland: 1,
    port_city: 8,
    airport: 9,
    city: 10,
    satellite_capital: 15,
    capital: 100,
};

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct AI {
    pub scores: Scores,
}

impl AI {
    pub fn new() -> Self {
        AI{scores: DEFAULT_SCORES}
    }
    fn match_tile_category_score(&self, category: &TileCategory) -> i32 {
        match category {
            TileCategory::Farmland => self.scores.farmland,
            TileCategory::Water => self.scores.water,
        }
    }
    fn match_locality_category_score(&self, category: &LocalityCategory, owner_idx: Option<usize>) -> i32 {
        match category {
            LocalityCategory::City => self.scores.city,
            LocalityCategory::PortCity => self.scores.port_city,
            LocalityCategory::Airport => self.scores.airport,
            LocalityCategory::Capital(i) => {
                if *i == owner_idx.unwrap() {
                    self.scores.capital
                } else {
                    self.scores.satellite_capital
                }
            },
            // LocalityCategory::SatelliteCapital => self.scores.satellite_capital,
        }
    }
    /// Calculates the base score for capturing a tile.
    fn match_tile_score(&self, tile: &Tile) -> i32 {
        match &tile.locality {
            Some(locality) => self.match_locality_category_score(&locality.category, tile.owner_index),
            None => self.match_tile_category_score(&tile.category),
        }
    }

    /// Calculates the bonus score for capturing neighbouring tiles of a cube.
    fn calculate_extended_border_score(&self, own_player_index: &usize, world: &World, cube: &Cube<i32>) -> i32 {
        let mut score = 0;
        let neighbours_cube = cube.disc(1);
        for neighbour in neighbours_cube {
            if let Some(tile) = &world.get(&neighbour) {
                if world.is_cube_extendable(&cube, &neighbour) && tile.owner_index != Some(*own_player_index) {
                    score += self.match_tile_category_score(&tile.category);
                }
            }
        }
        score
    }

    /// Calculates the combat score component.
    fn calculate_combat_score(&self, world: &World, origin_cube: &Cube<i32>, target_cube: &Cube<i32>) -> i32 {
        let origin = world.get(origin_cube).unwrap();
        let target = world.get(target_cube).unwrap();
        let origin_army = origin.army.as_ref().unwrap();
        let target_army = target.army.as_ref().unwrap();
        let diff = origin_army.combat_strength() - target_army.combat_strength();
        let mut score = diff / 10;
        if diff > 0 {
            score += target_army.manpower * self.scores.manpower / 10;
        } else {
            score -= origin_army.manpower * self.scores.manpower / 10;
        }
        score
    }

    /// Calculates and returns a score value for a given move.
    fn calculate_score(&self, own_player_index: &usize, world: &World, origin_cube: &Cube<i32>, target_cube: &Cube<i32>) -> i32 {
        let origin = world.get(origin_cube).unwrap();
        let target = world.get(target_cube).unwrap();
        let mut score = 0;
        if target.owner_index != Some(*own_player_index) {
            match &target.army {
                Some(target_army) => {
                    let origin_army = origin.army.as_ref().unwrap();
                    score += self.calculate_combat_score(&world, &origin_cube, target_cube);
                    let diff = origin_army.combat_strength() - target_army.combat_strength();
                    if diff > 0 {
                        score += self.match_tile_score(&target);
                        score += self.calculate_extended_border_score(&own_player_index, &world, &target_cube);
                    }
                }
                None => {
                    score += self.match_tile_score(&target);
                    score += self.calculate_extended_border_score(&own_player_index, &world, &target_cube);
                }
            }
        } else {
            score += self.calculate_extended_border_score(&own_player_index, &world, &target_cube);
        }
        score
    }

    /// Creates a subset of the game world containing only entries with own armies.
    fn create_owned_armies_world_subset(&self, own_player_index: &usize, world: &World) -> HashSet<Cube<i32>> {
        // TODO: Decouple subset of tiles containing armies from useful moves subset.
        let mut result = HashSet::new();
        for cube in world.cubes_by_ownership.get(own_player_index).unwrap().iter() {
            let tile = world.get(cube).unwrap();
            if let Some(army) = &tile.army {
                if army.can_move && world.is_there_capturable_tile_within_range(cube) { //can_move redundant if all created once a turn?
                    result.insert(*cube);
                }
            }
        }
        result
    }

    /// Explores the scores a tile containing an army can achieve for all valid targets.
    fn explore_targets(&self, own_player_index: &usize, world: &World, origin: &Cube<i32>) -> Option<ScoredMove> {
        //let mut results = Vec::new();
        let valid_targets = world.get_reachable_cubes(&origin);
        let prev_score = 0;
        let mut result = None;
        for target in valid_targets {
            let score = self.calculate_score(&own_player_index, world, &origin, &target);
            if score > prev_score {result = Some(ScoredMove{score, origin: *origin, target});}
            // let element = ScoredMove{score, origin: *origin, target};
            // results.push(element);
            //return result; // can only move each army once, how to handle?
        }
        result
    }

    /// Score every likely useful player move.
    fn create_target_list(&self, own_player_index: &usize, world: &World) -> Vec<ScoredMove> {
        let subset = self.create_owned_armies_world_subset(&own_player_index, &world); // this only returns 'useful' armies
        let mut target_list = vec!();
        for origin in subset {
            target_list.push(self.explore_targets(&own_player_index, &world, &origin))
            //target_list.append(&mut self.explore_targets(&own_player_index, &world, &origin))
        }
        target_list.into_iter().flatten().collect::<Vec<ScoredMove>>()
    }

    /// Based on the target list, pick generate the most optimal targets.
    pub fn generate_targets(&self, own_player_index: &usize, world: &World) -> Vec<ScoredMove> {
        // TODO: Return a lazy generator instead. -> std::slice::Iter<'_, ScoredMove>
        let mut target_list = self.create_target_list(&own_player_index, &world);
        target_list.sort_by_key(|scored_move| Reverse(scored_move.score));
        if target_list.is_empty() {println!("empty target list")};
        target_list
    }

    // def controller(game, target):
    //     """Attempts to calculate the most optimal move and performs the necessary
    //     click_on_tile() Player method calls to execute them.
    //     """
    //     origin_tile = game.world.get(target.origin)
    //     target_tile = game.world.get(target.target)
    //     tilepair_origin = (target.origin, origin_tile)
    //     tilepair_target = (target.target, target_tile)
    //     game.current_player.click_on_tile(tilepair_origin)
    //     game.current_player.click_on_tile(tilepair_target)

    // # def create_threat_list():
    // #     """For every hostile army in the game world, calculate the threat level."""
    // #     pass

    // # def pick_important_threats():
    // #     """Based on the threat list, pick 5 most dangerous threats."""
    // #     pass

    // # def pick_actions():
    // #     """Pick """
}