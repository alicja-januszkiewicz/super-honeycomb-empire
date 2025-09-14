use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::game;
use game::{VictoryCondition};

use crate::world;
use world::Player;

#[derive(Serialize, Deserialize)]
pub struct Ruleset {
    pub victory_condition: VictoryCondition,
    pub fog_of_war: HashMap<usize, bool>,
}

// impl Default for Ruleset {
//     fn default() -> Self {
//         Self {
//             victory_condition: VictoryCondition::Elimination,
//             fog_of_war: HashMap::new()
//         }
//     }
// }

impl Ruleset {
    pub fn new(victory_condition: VictoryCondition, fog_of_war: HashMap<usize, bool>) -> Self {
        Self{victory_condition, fog_of_war}
    }

    pub fn default(victory_condition: VictoryCondition, players: &Vec<Player>) -> Self {
        let n = players.len();
        let mut fog_of_war = HashMap::new();
        (0..n).into_iter().for_each(|i| {fog_of_war.insert(i, true);});
        fog_of_war.insert(n, true);
        // fog_of_war.insert(n, false);
        Self::new(victory_condition, fog_of_war)
    }
}
