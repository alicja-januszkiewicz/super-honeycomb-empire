//! Game logic module

// import ai
// import worldgen
// import playergen

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::option::Option;

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

use serde::Deserialize;
use serde::Serialize;
use strum::EnumString;
use strum::{EnumIter, Display};

use crate::cubic::*;
use crate::network::AsAny;
use crate::network::IntoAny;
use crate::rules::Ruleset;
use crate::ui::Ui;
use crate::Army;
use crate::Component;
use crate::Controller;
use crate::Editor;
use crate::Player;
use crate::VisibilityMask;
use crate::World;
use crate::mquad::Assets;
use crate::world::MAX_STACK_SIZE;
use crate::world::ACTIONS_PER_TURN;
use crate::world::Command;

use crate::world::TileCategory;
use crate::world::gen::*;

#[derive(Serialize, Deserialize, EnumIter, strum::AsStaticStr, Display, PartialEq, Default)]
pub enum VictoryCondition {
    #[default]
    Elimination,
    Territory(f32),
    CaptureAndHold(HashSet<Cube<i32>>),
}

impl VictoryCondition {
    fn check_territory(world: &World, player_index: usize, target_percentage: f32) -> bool {
        let world_total = world.len();
        let player_total = world.iter().filter(|&(_, tile)| tile.owner_index == Some(player_index)).count();
        player_total as f32 / world_total as f32 >= target_percentage
    }
    pub fn check(&self, world: &World, player_index: usize) -> bool {
        match self {
            Self::Elimination => unimplemented!(),
            Self::Territory(x) => VictoryCondition::check_territory(&world, player_index, *x),
            _ => unimplemented!(),
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct Game {
    pub turn: usize,
    pub players: Vec<Player>,//[&'a Player<'a>],//Vec<&Player>,
    //pub current_player: &'a Player, // change it to a function?
    pub world: World,
    // #[serde(skip_serializing)]
    // pub player_visibilities: HashMap<usize, crate::VisibilityMask>,
    // pub player_observers: HashMap<usize, HashMap<Cube<i32>, Cube<i32>>>,
    // pub player_fogs: HashMap<usize, crate::Fog>,
    pub player_views: HashMap<usize, World>,
    pub rules: Ruleset,
}

impl From<crate::map_editor::Editor> for Game {
    fn from(value: crate::map_editor::Editor) -> Self {
        // let player_idx: std::collections::hash_map::Keys<'_, usize, HashSet<Cube<i32>>> = value.world.cubes_by_ownership.keys();
        // let mut players: Vec<Player> = vec!();
        // player_idx.for_each(|i| {
        //     let player = Player::new("", Some(crate::ai::AI::new()));
        //         players.push(player);
        //     }
        // );
        crate::Game{turn: 1, players: value.players, world: value.world, player_views: value.player_views, rules: value.rules}
    }
}

impl From<Game> for crate::map_editor::Editor {
    fn from(value: Game) -> Self {
        crate::map_editor::Editor::new(value.world, value.players)
    }
}

impl Game {
    pub fn init_views(&mut self) {
        for (player_idx, fog_flag) in self.rules.fog_of_war.clone() {
            if fog_flag {
                // let view = self.world.iter().filter(|c, t| );
                self.player_views.insert(player_idx, World::new());
            }
        }
    }
    pub fn new(players: Vec<Player>, world: World, rules: Ruleset, assets: &mut Assets) -> Self {
        // let world: World = World::new();
        let player_views = HashMap::new();

        let mut game = Self {
            turn: 1,
            players,
            world,
            player_views,
            rules,
        };
    
        // Game::init_world(&mut game, assets);
        game.init_views();
        // Game::init_views(&mut game);
        game
    }
    // pub async fn draw(&self, &layout: &Layout<f32>, assets: &Assets, time: f32) {
    //     crate::draw(&self, &layout, assets, time).await;
    // }
    // pub fn poll(&mut self, layout: &mut Layout<f32>) {
    //     crate::poll_inputs(self, layout);
    // }
    // pub fn swap(self) -> crate::map_editor::Editor {
    //     self.into()
    // }
    // pub fn update(&mut self) {
    //     self._update()
    // }
    pub fn to_json(&self, path: &str) {
        let file = File::create(&path).expect("Failed to open the file.");
    
        match serde_json::to_writer(file, self) {
            Ok(()) => println!("HashMap serialized and saved successfully."),
            Err(e) => eprintln!("Error during serialization: {}", e),
        }
    }

    pub fn from_json(path: &str) -> Self {
        let f = File::open(path)
            .expect("file should open read only");

        serde_json::from_reader(f).expect("file should be proper JSON")
    }

    // pub fn from_ui(mut ui: Ui, assets: &mut Assets) -> Self {
    //     let players = std::mem::take(&mut ui.players);
    //     let rules = Ruleset::from(ui);
    //     Self::new(players, rules, assets)
    // }

    pub fn current_player_index(&self) -> Option<usize> {
        match self.players.len() {
            0 => None,
            _ => Some((self.turn - 1) as usize % self.players.len())
        }
    }
    pub fn current_player_mut(&mut self) -> Option<&mut Player> {
        match self.current_player_index() {
            Some(index ) => Some(&mut self.players[index]),
            None => None,
        }
    }
    pub fn current_player(&self) -> Option<&Player> {
        match self.current_player_index() {
            Some(index ) => Some(&self.players[index]),
            None => None,
        }
    }
    pub fn init_world(game: &mut Self, assets: &mut Assets) {
        let shape_gen = ShapeGen::Custom(assets.shape.clone());
        // let shape_gen = ShapeGen::Hexagonal(8);
        let river_gen = RiverGen::Custom(assets.river.clone());
        // let river_gen = RiverGen::Random(300, 0.3);
        let localities_gen = LocalitiesGen::Random;
        let capitals_gen = CapitalsGen::Random(4);
        game.world.generate(
            shape_gen,
            river_gen,
            localities_gen,
            capitals_gen,
            &mut assets.locality_names.iter().map(|s| &**s).collect(),
            &assets.init_layout,
        );
        // println!("{}", self.world.len());
        // println!("river (debug): {:?}", self.world.rivers);
    }

    pub fn execute_command(&mut self, command: &Command) {
        self.world.execute_army_order(command);
        let current_player = self.current_player_mut().unwrap();
        current_player.actions -= 1;
        current_player.selection = None; // deselect
        // if let Some(mask) = self.player_visibilities.get_mut(&current_player_index) {
        //     mask.update(&self.world, current_player_index);
        // }
    }
    // Clicking on a tile with an army selects it. If the player
    // already has a selection, it will return a command with the
    // clicked tile as the target of the command. If no command can
    // be issued, and the clicked tile has an army, it will be
    // selected. If it does not, the player's selection will be set
    // to None. Clicking on the selected tile deselects it.
    // returns command to be executed
    pub fn click(&mut self, target_cube: &Cube<i32>) -> Option<Command> {
        let target = self.world.get(target_cube).unwrap();
        let current_player_index = self.current_player_index().unwrap();
        let current_player = self.current_player().unwrap();
        println!("current_selection: {:?}", current_player.selection);
        println!("click: {:?}", target_cube);
        // let is_target_selectable = if let Some(army) = &target.army {
        //     && target.owner_index == self.player_index(&world)
        //     && army.can_move
        //     && Some(target_cube) != self.selection
        // };
        let mut is_target_selectable = false;
        if let Some(army) = &target.army {
            if army.owner_index == Some(current_player_index)
            && army.can_move // shouldnt this be origin.army.canmove?
            && Some(*target_cube) != current_player.selection {
            // && self.world.is_cube_targetable(&current_player.selection, target_cube) {
                is_target_selectable = true;
            }
        };
        if let Some(selection) = current_player.selection {
            let legal_moves = self.world.get_all_legal_moves(&selection, &current_player_index); // self.world.get_reachable_cubes(&selection);
            if legal_moves.contains(target_cube) { // && self.world.is_cube_targetable(&selection, target_cube) { // !matches!(target.category, TileCategory::Water) {
                let command = Command{from: selection, to: *target_cube, via: Vec::new()};
                return Some(command);
            }
        }
        let current_player = self.current_player_mut().unwrap();
        if is_target_selectable {
            current_player.selection = Some(*target_cube);
        } else {
            current_player.selection = None;
        }
        None
    }
    pub fn next_turn(&mut self) {
        let current_player_index = self.current_player_index().unwrap();
        let current_player = self.current_player_mut().unwrap();
        current_player.selection = None;
        current_player.actions = ACTIONS_PER_TURN;
        self.world.train_armies(&current_player_index);

        // Reset army movement points
        self.world.apply_idle_morale_penalty(current_player_index);
        for tile in self.world.values_mut() {
            if let Some(army) = &mut tile.army {
                army.can_move = true;
            }
        }

        self.turn += 1;
        println!("Turn {}: {}", self.turn, self.current_player().unwrap());
    }
    pub fn _update(&mut self) {
        let Some(current_player_index) = self.current_player_index() else {return};
        let current_player = self.current_player().unwrap();

        // if self.players.len() <= 1 {
        //     println!("{} wins!", current_player);
        // }

        // Force a player to skip a turn if he has no units to move or no action points left.
        let can_player_issue_a_command = self.world.can_player_issue_a_command(&current_player_index);
        if current_player.actions == 0 || !can_player_issue_a_command {
            self.next_turn();
            return
        }

        // Let AI make a move
        match &current_player.controller {
            Controller::Human => {},
            Controller::AI(ai) => {
                let targets = ai.generate_targets(&current_player_index, &self.world);
                for target in targets {
                    if self.current_player().unwrap().actions > 0 {
                        match self.click(&target.origin) {
                            Some(command) => self.execute_command(&command),
                            None => {},
                        };
                        match self.click(&target.target) {
                            Some(command) => self.execute_command(&command),
                            None => {},
                        };
                    } else {
                        break
                    }
                }
                self.current_player_mut().unwrap().skip_turn();
            },
            Controller::Remote => {},
        }
    }
}

impl crate::Component for Game {
    // type Swap = Editor;
    fn draw(&self, &layout: &Layout<f32>, assets: &Assets, time: f32) {
        crate::draw(&self, &layout, assets, time);
    }
    fn poll(&mut self, layout: &mut Layout<f32>) -> bool {
        crate::poll_inputs(self, None, layout)
    }
    // fn swap(self) -> Self::Swap{//impl Component {
    //     crate::Editor::from(self)
    // }
    fn swap(self: Box<Self>) -> Box<dyn Component> {
        Box::new(crate::Editor::from(*self))
    }
    // fn swap(self) -> impl Component + IntoAny {
    //     crate::Editor::from(self)
    // }
    fn update(&mut self) {
        self._update()
    }
    // fn empty() -> Self {
    //     let players = vec!();
    //     let world: World = World::new();
    //     let player_views = HashMap::new();
    //     let victory_condition = VictoryCondition::default();
    //     let rules = Ruleset::default(victory_condition, &players);

    //     Self {
    //         turn: 1,
    //         players,
    //         world,
    //         player_views,
    //         rules,
    //     }
    // }
}

// class Game:
//     playergen : object
//         an infinite generator provided by itertools.cycle
//     """
//     def __init__(self):
//         self.turn = 0
//         self.players = playergen.classic(self)
//         self.playergen = itertools.cycle(self.players)
//         self.current_player = self.players[0]
//         #self.world = worldgen.generate_world(shape='classic', radius=6, algorithm='random_ots', spawntype='classic', players=self.players)
//         self.world = worldgen.generate_world(shape='hexagon', radius=20, algorithm='random_ots', spawntype='random', players=self.players)
//         self.initial_layout = worldgen.layout
//         playergen.create_player_cameras(self)

//     def defeat_player(self, player):
//         """Removes a player instance from the players list."""
//         player.is_defeated = True
//         print(player, "has been defeated!")

//     def check_victory_condition(self):
//         for player in self.players:
//             starting_tile = self.world.get(player.starting_cube)
//             if starting_tile.owner != player and not player.is_defeated:
//                 self.defeat_player(player)
//                 self.surrender_to_player(player, starting_tile.owner)

//     def print_world_state(self):
//         """A debugging function used in graphicless mode."""
//         for tile in self.world.values():
//             if tile.army:
//                 print(tile, tile.owner, tile.army)

// def main():
//     """For debugging purposes"""
//     game = Game()
//     game.current_player.actions = 0
//     while len(game.players) > 1:
//         game.update_world()