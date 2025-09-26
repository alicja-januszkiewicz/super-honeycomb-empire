use chrono::Local;
use rand::random;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use wgpu::core::resource;

use crate::fog;
use crate::game::GameResources;
use crate::inputs::poll_inputs;
use crate::inputs::InputState;
use fog::Fog;

use crate::cli;
use cli::Cli;

use crate::world;
use world::Command;

use crate::map_editor;
use map_editor::Editor;

use crate::rules::Ruleset;
// use crate::ui;
use crate::backend::Backend;
use crate::Controller;
use crate::Cube;
use crate::Game;
use crate::Layout;
use crate::World;
use crate::Player;

use core::fmt;
use core::panic;
use std::any::Any;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::format;
use std::fmt::write;
use std::fmt::Display;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::marker::PhantomData;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::ToSocketAddrs;

#[derive(Debug, Clone)]
pub enum EndpointType {
    Offline,
    Client,
    Server
}

pub trait Mode {
    type Endpoint;
}

pub trait IsUserActive {
    fn is_user_active(&self, uid: usize) -> bool {true}
}

impl IsUserActive for Editor {}
impl IsUserActive for Game {
    fn is_user_active(&self, uid: usize) -> bool {
        self.current_player_index().and_then(|pid| Some(pid == uid)).unwrap_or(false)
    }
}

pub trait HandleOutboundMessage {
    fn handle_outbound_message(&mut self, message: Message) -> Message;
}

// State markers with their associated data
pub struct Offline;
pub struct ClientMode;
pub struct ServerMode;

impl Mode for Offline {
    type Endpoint = Offline;  // No endpoint data
}

impl Mode for ClientMode {
    type Endpoint = Client;
}

impl Mode for ServerMode {
    type Endpoint = Server;
}

pub trait NotOffline {}

impl NotOffline for ClientMode {}
impl NotOffline for ServerMode {}

pub struct App<M: Mode, B: Backend, T: Component<B>> {
    pub component: T,
    pub endpoint: M::Endpoint,
    _marker: std::marker::PhantomData<B>,
}

// impl<M: Mode, B: Backend> App<M, B, Game> {
//     pub fn from_ui(mut ui: crate::ui::Ui<B>, assets: &mut Assets) -> Self where <M as Mode>::Endpoint: 'static {
//         let replacement: Box<dyn Chat> = Box::new(Offline);
//         let endpoint_ = std::mem::replace(&mut ui.endpoint, replacement);
//         let endpoint = *endpoint_.into_any().downcast::<M::Endpoint>().unwrap();
//         // let endpoint = Client::new("").unwrap().into();
//         let players = std::mem::take(&mut ui.players);
//         let rules = Ruleset::from(ui);
//         let mut component = Game::new(players, world, rules);
//         println!("init complete!");
//         Self {component, endpoint}
//     }
// }

// impl<M: Mode> App<M, Editor> {
//     pub fn from_ui(mut ui: ui::Ui, assets: &mut Assets) -> Self where <M as Mode>::Endpoint: 'static {
//         let replacement: Box<dyn Chat> = Box::new(Offline);
//         let endpoint_ = std::mem::replace(&mut ui.endpoint, replacement);
//         let endpoint = *endpoint_.into_any().downcast::<M::Endpoint>().unwrap();
//         // let endpoint = Client::new("").unwrap().into();
//         let players = std::mem::take(&mut ui.players);
//         let rules = Ruleset::from(ui);
//         let mut component = Editor::new(World::new(), players);
//         println!("init complete!");
//         Self {component, endpoint}
//     }
// }

impl<M, B, T> App<M, B, T> 
where M: Mode, B: Backend, T: Component<B>
{
    pub fn new(component: T, endpoint: M::Endpoint) -> Self {
        let _marker = PhantomData::default();
        Self {component, endpoint, _marker}
    }
    // pub fn draw(&self, layout: &Layout<f32>, backend: &B, resources: &GameResources, time: f32) {
    //     self.component.draw(layout, backend, resources, time)
    // }
    // pub fn poll(
    //     &mut self,
    //     layout: &mut Layout<f32>,
    //     backend: &B,
    // ) -> Message {
    //     let mut message = self.component.poll(layout, backend);

    //     message = self.endpoint.handle_outbound_message(message);

    //     message
    // }
    // pub fn update() {

    // }
}

// impl<M: Mode<Endpoint = M>, B: Backend, T: Component<B>> App<M, B, T> where M: HandleOutboundMessage{
impl<M, B, T> Component<B> for App<M, B, T> 
where
    M: Endpoint<B>, //HandleOutboundMessage + Endpoint<B>,
    M: Mode<Endpoint = M>, 
    B: Backend,
    T: Component<B> + IsUserActive + 'static, App<M, B, T>: IntoAny
{
    fn draw(&self, layout: &Layout<f32>, backend: &B, resources: &GameResources, time: f32) {
        self.component.draw(layout, backend, resources, time)
    }
    fn poll(
        &mut self,
        layout: &mut Layout<f32>,
        backend: &B,
    ) -> Message {
        let mut message = self.component.poll(layout, backend);
        message
    }
    fn update(mut self: Box<Self>, message: Message) -> Option<Box<dyn Component<B>>> {
        let n = self.endpoint.count_connections();
        let active_users: Vec<usize> = (0..n).filter_map(|uid| self.component.is_user_active(uid).then_some(uid)).collect();

        self.endpoint.tick(Box::new(self.component), message, active_users)
    }
}

// impl<B, T> HandleOutboundMessage for App<Offline, B, T> where B: Backend, T: Component<B> {
//     fn handle_outbound_message(&mut self, message: Message) -> Message {
//         message
//     }
// }

// impl<B, T> HandleOutboundMessage for App<ClientMode, B, T> where B: Backend, T: Component<B> {
//     fn handle_outbound_message(&mut self, message: Message) -> Message {
//         match Message {
//             Message::Command(_)
//             Message::Save
//             Message::Load
//             Message::SkipTurn
//             Me
//         }
//     }
// }


// impl<B, T> App<Offline, B, T>
// where
//     B: Backend,
//     T: Component<B>,
// {
//     pub fn poll(self: Box<Self>, layout: &mut Layout<f32>, backend: &B) -> (Message, Option<Box<dyn Component<B>>>) {
//         let (msg, swap) = self.component.poll(layout, backend);
//         // let mut comp: Box<dyn Component<B>> = self.component.poll(layout, backend);

//         // let message = self.endpoint.poll(&mut *comp, layout, backend);
//         component.execute_message(msg);

//         let mut exit = false;
//         let mut swap = None;
//         match msg {
//             Message::Exit => {exit = true},
//             Message::Swap => {swap = Some(self as Box<dyn Component<B>>)},
//             _ => {},
//         }

//         (exit, swap)
//     }
// }

// impl<B, T> App<ClientMode, B, T>
// where
//     B: Backend,
//     T: Component<B>,
// {
//     pub fn poll(self: Box<Self>, layout: &mut Layout<f32>, backend: &B) -> (Message, Option<Box<dyn Component<B>>>) {
//         let (msg, swap) = self.component.poll(layout, backend);
//         // let mut comp: Box<dyn Component<B>> = self.component.poll(layout, backend);

//         // let message = self.endpoint.poll(&mut *comp, layout, backend);
//         // component.execute_message(msg);

//         // send command to server
//         // it will be sent back by the server if valid, at which point it will be executed
//         write_json_message(&self.endpoint.stream, &msg);

//         let mut exit = false;
//         let mut swap = None;
//         match msg {
//             Message::Exit => {exit = true},
//             Message::Swap => {swap = Some(self as Box<dyn Component<B>>)},
//             _ => {},
//         }

//         (exit, swap)
//     }
// }


// impl<M, B, T> App<M, B, T>
// where
//     M: Mode<Endpoint = M>,
//     B: Backend,
//     T: Component<B>,
// {
//     pub fn poll(
//         mut self,
//         layout: &mut Layout<f32>,
//         backend: &B,
//     ) -> (bool, Option<App<M, B, Box<dyn Component<B>>>>) {
//         B::poll_camera_inputs(layout);

//         // let component handle its own inputs
//         let mut comp: Box<dyn Component<B>> =
//             self.component.poll(layout, backend);

//         // let the endpoint deal with side effects
//         self.endpoint.poll(&mut *comp, layout, backend);

//         let mut exit = false;
//         let mut swap = None;

//         if is_key_pressed(KeyCode::Escape) {
//             exit = true;
//         }
//         if is_key_pressed(KeyCode::F1) {
//             swap = Some(App {
//                 component: comp.swap(),
//                 endpoint: self.endpoint,
//                 _marker: std::marker::PhantomData,
//             });
//         }

//         (exit, swap)
//     }
// }

// impl<T: Component<Swap = U>, U: Component> App<Offline, T> {
//     pub fn swap_component(self) -> Box<App<Offline, U>> {
//         let component = self.component.swap();
//         let endpoint = self.endpoint;
//         Box::new(App::<Offline, U> {component, endpoint})
//     }
// }

// impl App<Offline, Game> {
//     pub fn poll(&mut self, layout: &mut Layout<f32>) -> bool {
//         self.component.poll(layout)
//     }
//     pub fn update(mut self) -> Self {
//         self.component.update();
//         self
//     }
// }
// impl<T: Component<Swap = U>, U: Component> Component for App<Offline, T> {
// impl Component for App<Offline, Game> {
//     fn poll(&mut self, layout: &mut Layout<f32>) -> bool {
//         self.component.poll(layout)
//     }
//     fn draw(&self, layout: &Layout<f32>, assets: &Assets, time: f32) {
//         self.component.draw(layout, assets, time)
//     }
//     fn update(&mut self) {
//         self.component.update();
//     }
//     fn swap(self: Box<Self>) -> Box<dyn Component> {
//         let component = Box::new(self.component)
//             .swap()
//             .into_any()
//             .downcast::<Editor>()
//             .unwrap();
        
//         let endpoint = self.endpoint;
    
//         Box::new(App::<Offline, Editor> { component: *component, endpoint })
//     }
// }

// impl Component for App<Offline, Editor> {
//     fn poll(&mut self, layout: &mut Layout<f32>) -> bool {
//         self.component.poll(layout)
//     }
//     fn draw(&self, layout: &Layout<f32>, assets: &Assets, time: f32) {
//         self.component.draw(layout, assets, time)
//     }
//     fn update(&mut self) {
//         self.component.update();
//     }
//     fn swap(self: Box<Self>) -> Box<dyn Component> {
//         let component = Box::new(self.component)
//             .swap()
//             .into_any()
//             .downcast::<Game>()
//             .unwrap();
        
//         let endpoint = self.endpoint;
    
//         Box::new(App::<Offline, Game> { component: *component, endpoint })
//     }
// }

// impl Component for App<Offline, Editor> {
//     type Swap = App<Offline, Editor>;
//     fn poll(&mut self, layout: &mut Layout<f32>) -> bool {
//         self.component.poll(layout)
//     }
//     fn draw(&self, layout: &Layout<f32>, assets: &Assets, time: f32) {
//         self.component.draw(layout, assets, time)
//     }
//     fn update(&mut self) {
//         self.component.update();
//     }
//     fn swap(self) -> App<Offline, Editor> {
//         let component = self.component.swap();
//         let endpoint = self.endpoint;
//         App::<Offline, Editor> {component, endpoint}
//     }
// }

impl<B: Backend> App<ClientMode, B, Game> {
    // pub fn poll(&mut self, layout: &mut Layout<f32>, input_state: &InputState) -> bool {
    //     poll_inputs(&mut self.component, Some(&self.endpoint), layout, input_state)
    // }
    pub fn update(mut self) -> Self {
        // Force a player to skip a turn if he has no units to move or no action points left.
        let Some(current_player_index) = self.component.current_player_index() else {return self};
        let current_player = self.component.current_player().unwrap();

        let can_player_issue_a_command = self.component.world.can_player_issue_a_command(&current_player_index);
        if current_player.actions == 0 || !can_player_issue_a_command {
            self.component.next_turn();
        }

        match read_json_message_async(&self.endpoint.stream) {
            Some(result) => {
                match result {
                    Ok(message) => {
                        self.handle_message(message);
                    },
                    Err(e) => {
                        panic!("{}", e);
                    }
                }
            }
            None => {},
        }
        self
    }
    fn handle_message(&mut self, message: Message) {
        match message {
            Message::NewPlayer { starting_position, player } => {
                self.component.world.gen_capital_at_cube(self.component.players.len(), starting_position);
                self.component.players.push(player);
            },
            Message::Initialise {..} => {},
            Message::Command(command) => {
                println!("executing command {:?}", command);
                self.component.execute_command(&command);
                // println!("clicking");
                // self.app.click(&command.from);
                // self.app.click(&command.to);
            },
            Message::RevealFog(result) => {
                match result {
                    Ok(mut world) => self.component.world.extend(world.drain()),
                    Err(e) => panic!("{}", e),
                }
            },
            Message::SkipTurn => {
                self.component.current_player_mut().unwrap().skip_turn();
            },
            Message::Chat(msg) => {
                self.endpoint.chatlog.push(msg);
            },
            _ => {},
        }
    }
}

impl<B: Backend> App<ServerMode, B, Game> where
    Game: Component<B>,
{
    pub fn poll(&mut self, layout: &mut Layout<f32>, backend: &B) -> bool {
        // self.component.poll(layout, backend)
        true
    }
    // pub fn update(mut self) -> Self {
    //     self = self.handle_client().expect("err");
    //     self.component.update();
    //     self.poll_current_stream();
    //     self
    //     // self.endpoint = self.poll_current_stream();
    // }
    fn handle_client(mut self) -> std::io::Result<Self> {
        for stream in self.endpoint.listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    // add Player to Game.players
                    let Message::NewPlayer { player: received_player, ..} = read_json_message(&stream).unwrap() else{panic!()};
                    let player_idx = self.component.players.len();
                    let mut player = Player::new(&received_player.name, Controller::Remote);
                    self.component.players.push(player);
                    // generate a starting position for the player
                    let cubes_with_cities = &self.component.world.get_cubes_with_cities();
                    let starting_position = *&self.component.world.gen_random_capital(player_idx, &cubes_with_cities);

                    // send the visible starting area to incoming client
                    // compute the area
                    // let fog = self.app.player_fogs.get(&player_idx).unwrap();
                    // let observations = self.app.world.get_visible_subset(fog);
                    // write_json_message(&stream, &observations);

                    let message = Message::Initialise{turn: self.component.turn, players: self.component.players, world: self.component.world};
                    write_json_message(&stream, &message);
                    let Message::Initialise{players: mut p, world: w, ..} = message else {panic!()};
                    // self.app.players = p;
                    self.component.world = w;

                    player = p.pop().unwrap();
                    // broadcast to other players
                    let message = Message::NewPlayer{starting_position, player};
                    for s in &self.endpoint.streams {
                        write_json_message(&s, &message);
                    }
                    let Message::NewPlayer{starting_position: _, player: pl} = message else {panic!()};
                    p.push(pl);
                    self.component.players = p;

                    stream.set_nonblocking(true)?;
                    //self.streams.push(stream);
                    self.endpoint.streams.insert(player_idx, stream);
                    println!("{} joined", received_player.name);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No more connections to accept right now
                    // break to prevent from blocking
                    break;
                }
                Err(e) => {
                    eprintln!("Failed to accept a connection: {:?}", e);
                }
            }
        }
        Ok(self)
    }

    fn poll_current_stream(mut self: Box<Self>) -> Option<Box<dyn Component<B>>> { // -> <ServerMode as Mode>::Endpoint {
        let mut swap = None;
        if self.endpoint.streams.is_empty() {
            return swap;
        }
        let idx = self.component.current_player_index().unwrap();
        // println!("listening for player index {}", idx);
        let Some(stream) = self.endpoint.streams.get(idx) else {return swap};//&self.streams[idx];
        // println!("got stream...");
        // let Ok(command): Result<Command, Box<dyn std::error::Error>> = read_json_message(stream) else {println!("bad command?"); return Ok(())};
        let Some(Ok(message)): Option<Result<Message, std::io::Error>> = read_json_message_async(stream) else {return swap};
        
        println!("received message from pid {:}: {:}", idx, message);
        let mut comp = std::mem::replace(&mut self.component, panic!("update called after swap"));
        let swap = Box::new(comp).update(message);
        // let swap = self.component.update(message);
        for s in self.endpoint.streams.iter() {
            write_json_message(s, &message);
        }
        swap
    }
    fn handle_command(&mut self, command: Command) {//-> <ServerMode as Mode>::Endpoint {
        let idx = self.component.current_player_index().unwrap();
        let Some(stream) = self.endpoint.streams.get(idx) else {return};

        // play out the command step-by-step
        // at each step, check which players can observe the command (before executing the step)
        // for each player that observes the command, truncate the command up to where they stop observing
        // after the command has finished playing out, relay the potentially truncated command to all players that have observed some part of it
        // rn there are no steps, so the process is simplified. unit's position is leaked if a unit moves out of or into view, but this might eventually be represented (drawn), or the commands will be reworked to involve steps.

        // do not broadcast the same move to its sender
        // let mut n: Vec<usize> = (0..self.app.players.len()).collect();
        // n.remove(idx);
        let n = 0..self.component.players.len(); //.into_iter()

        let views = std::mem::take(&mut self.component.player_views);
        let observations = n.map(|i| views.get(&i)).map(|maybe_view| command.get_observed_sections(maybe_view));
        // execute the move
        self.component.execute_command(&command);

        // tell client move was ok
        // let message = Message::RevealFog(Ok::<World, ServerResponseError>(World::new()));
        // write_json_message(stream, &message);

        // send the observations to clients
        observations.enumerate().for_each(|(idx, obs)| {
            obs.into_iter().for_each(|command| {
                println!("sending {:?} to {}", command, idx);
                write_json_message(self.endpoint.streams.get(idx).unwrap(), &Message::Command(command));
            });
        });
        self.component.player_views = views;
        // self.endpoint
    }
}

impl<M: Mode, B: Backend> App<M, B, Editor> {
    pub fn update(mut self) {}
}

#[derive(Debug)]
pub struct Client {
    pub stream: TcpStream,
    players: Vec<NetworkPlayer>,
    pub chatlog: Vec<ChatMsg>,
}

pub trait GetChat {
    fn get_chatlog(&self) -> &Vec<ChatMsg>;
}

impl GetChat for Offline {
    fn get_chatlog(&self) -> &Vec<ChatMsg> {
        panic!("not implemented")
    }
}

impl GetChat for Client {
    fn get_chatlog(&self) -> &Vec<ChatMsg> {
        &self.chatlog
    }
}

impl GetChat for Server {
    fn get_chatlog(&self) -> &Vec<ChatMsg> {
        &self.chatlog
    }
}

pub trait SendChat {
    fn send_chat_message(&mut self, msg: String) -> Result<(), Box<dyn std::error::Error>>;
}

impl SendChat for Offline {
    fn send_chat_message(&mut self, msg: String) -> Result<(), Box<dyn std::error::Error>> {
        panic!("not implemented")
    }
}

impl SendChat for Client {
    fn send_chat_message(&mut self, msg: String) -> Result<(), Box<dyn std::error::Error>> {
        let message: Message = Message::Chat(ChatMsg::from_str(&msg));
        write_json_message(&self.stream, &message)
    }
}

pub trait GetPlayers {
    fn get_players(&self) -> &Vec<NetworkPlayer>;
}

impl GetPlayers for Client {
    fn get_players(&self) -> &Vec<NetworkPlayer> {
        &self.players
    }
}

impl GetPlayers for Server {
    fn get_players(&self) -> &Vec<NetworkPlayer> {
        &self.players
    }
}


pub trait Endpoint<B: Backend> {
    fn tick_without_component(&mut self);
    fn tick(&mut self, component: Box<dyn Component<B>>, message: Message, active_users: Vec<usize>) -> Option<Box<dyn Component<B>>>;
    fn count_connections(&self) -> usize;
    // fn streams(&self);
    fn handle_input(&self);
    fn close(self: Box<Self>);
}

impl<B: Backend> Endpoint<B> for Server {
    fn tick_without_component(&mut self) {
        self.handle_client().expect("err");
        self.poll_all_streams();
    }
    fn tick(&mut self, mut component: Box<dyn Component<B>>, message: Message, active_users: Vec<usize>) -> Option<Box<dyn Component<B>>> {
        self.handle_client().expect("err");
        self.poll_all_streams();
        // component.update();

        // let comp = *component;
        // let idx = component.current_player_index().unwrap();
        // let idx = (&*component).get_current_player_index().unwrap();
        for uid in active_users {
            let stream = &self.streams[uid];
            let message = read_json_message_async(stream)?.ok().unwrap_or(Message::Tick);
            // println!("received message from uid {:}: {:}", uid, message);
            for s in self.streams.iter() {
                write_json_message(s, &message);
            }
            //TODO: Possibly will lead to bugs when two users try to swap on same frame
            component = component.update(message).unwrap();
        }
        Some(component)
    }
    fn count_connections(&self) -> usize {
        self.streams.len()
    }
    fn handle_input(&self) {
        unimplemented!()
    }
    fn close(self: Box<Self>) {
        
    }
}

impl<B: Backend> Endpoint<B> for Client {
    fn tick_without_component(&mut self) {
        self.update();
    }

    fn tick(&mut self, component: Box<dyn Component<B>>, message: Message, active_users: Vec<usize>) -> Option<Box<dyn Component<B>>> {
        write_json_message(&self.stream, &message);
        self.update();
        Some(component)
    }
    fn count_connections(&self) -> usize {
        1
    }
    fn handle_input(&self) {
        unimplemented!()
    }
    fn close(self: Box<Self>) {
        
    }
}

impl<B: Backend> Endpoint<B> for Offline {
    fn tick_without_component(&mut self) {
        
    }
    fn tick(&mut self, component: Box<dyn Component<B>>, message: Message, active_users: Vec<usize>) -> Option<Box<dyn Component<B>>> {
        component.update(message)
    }
    fn count_connections(&self) -> usize {
        0
    }
    fn handle_input(&self) {
        unimplemented!()
    }
    fn close(self: Box<Self>) {
        
    }
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}
pub trait IntoAny {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl AsAny for Game {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl IntoAny for Game {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

impl AsAny for Editor {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl IntoAny for Editor {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

// impl<M: Mode, T: Component> AsAny for App<M, T> {
impl<B: Backend + 'static> AsAny for App<Offline, B, Game> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<B: Backend + 'static> IntoAny for App<Offline, B, Game> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

impl<B: Backend + 'static> AsAny for App<Offline, B, Editor> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<B: Backend + 'static> IntoAny for App<Offline, B, Editor> {
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub trait Chat<B: Backend>: SendChat + GetChat + Endpoint<B> {
    fn as_any(&self) -> &dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<B: Backend> Chat<B> for Offline {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}
impl<B: Backend> Chat<B> for Client {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}
impl<B: Backend> Chat<B> for Server {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

impl Client {
    pub fn new<A: std::net::ToSocketAddrs + core::fmt::Display>(addr: A) -> Result<Self, std::io::Error> {
        println!("Connecting to {}...", addr);
        let stream = TcpStream::connect(addr)?;
        println!("TCP connection established...");

        // let n = random::<usize>() % 100;
        // let player = Player::new(&format!("Player {}", n), Controller::Human);
        // let message = Message::NewPlayer{starting_position: Cube::new(0, 0), player};
        // write_json_message(&stream, &message).unwrap();
        // println!("Player sent...");

        stream.set_nonblocking(true)?;
        let players = vec!();
        let chatlog = vec!();
        Ok(Self{stream, players, chatlog})

        //Ok(Self{player: Player::new("default", None), app, stream})
        // if let Ok(stream) = TcpStream::connect(addr) {
        //     println!("Connected to the server!");
        // } else {
        //     println!("Couldn't connect to server...");
        // }
    }
    fn update(&mut self) {
        match read_json_message_async(&self.stream) {
            Some(result) => {
                match result {
                    Ok(message) => {
                        match message {
                            Message::Chat(msg) => self.chatlog.push(msg),
                            _ => {},
                        }
                    },
                    Err(e) => {
                        panic!("{}", e);
                    }
                }
            }
            None => {},
        }
    }

}

impl Command {
    /// given a set of observers, work out which command segments are detected
    /// and turn those into separate subcommands.
    // pub fn get_observed_sections(&self, observers: &Fog) -> Vec<Command> {
    //     todo!()
    //                 // if (command.from in fog.keys() || command.to in fog.keys()) {

    //         // }
    //     // let command_path: 
    //     // let observed_route_positions = 
    // }
    pub fn get_observed_sections(&self, maybe_view: Option<&World>) -> Vec<Command> {
        match maybe_view {
            Some(view) => {
                if (view.contains_key(&self.from) || view.contains_key(&self.to)) {
                    vec![self.clone()]
                } else {
                    vec![]
                }
            },
            None => vec![self.clone()]
        }
    }
}

// #[derive(Serialize, Deserialize)]
// enum ServerResponse {
//     ValidMove(World),
//     InvalidMove,
// }

// impl<T: Component> Client<T> {
//     pub fn new<A: std::net::ToSocketAddrs>(app: T, addr: A) -> Result<Self, std::io::Error> {
//         let mut stream = TcpStream::connect(addr)?;

//         let mut buffer = Vec::new();
//         stream.read_to_end(&mut buffer)?;
//         let world: World = serde_json::from_slice(&buffer).expect("Failed to deserialize response");

//         let self_ = Self{player: Player::new("default", None), app, stream};
//         self_.set_world(world);
//         Ok(self_)

//         //Ok(Self{player: Player::new("default", None), app, stream})
//         // if let Ok(stream) = TcpStream::connect(addr) {
//         //     println!("Connected to the server!");
//         // } else {
//         //     println!("Couldn't connect to server...");
//         // }
//     }
// }

// impl<B: Backend> App<ClientMode, B, Game> {
//     pub fn new<Addr: std::net::ToSocketAddrs + core::fmt::Display>(mut game: Game, addr: Addr) -> Result<Self, std::io::Error> {
//         let client = Client::new(addr)?;

//         let n = random::<usize>() % 100;
//         let player = Player::new(&format!("Player {}", n), Controller::Human);
//         let message = Message::NewPlayer{starting_position: Cube::new(0, 0), player};
//         write_json_message(&client.stream, &message).unwrap();
//         println!("Player sent...");
//         // TODO: This will leak player.selection - perhaps censor the field before sending it here?
//         let Message::Initialise { turn, players, world } = read_json_message(&client.stream).unwrap() else {panic!()};
//         game.players = players;
//         let my_idx = game.players.len() - 1;
//         game.players[my_idx].controller = Controller::Human;
//         game.world = world;
//         game.turn = turn;
//         println!("Game state received...");

//         Ok(Self{component: game, endpoint: client, _marker: PhantomData::default()})

//         //Ok(Self{player: Player::new("default", None), component, stream})
//         // if let Ok(stream) = TcpStream::connect(addr) {
//         //     println!("Connected to the server!");
//         // } else {
//         //     println!("Couldn't connect to server...");
//         // }
//     }
// }

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerResponseError;

impl core::fmt::Display for ServerResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for ServerResponseError {}


impl<B: Backend> App<ClientMode, B, Game> {
    pub fn send_command(&mut self, command: Command) -> Result<Command, std::io::Error>{
        let message = Message::Command(command);
        write_json_message(&self.endpoint.stream, &message);
        let Message::Command(command) = message else {panic!()};
        let Message::RevealFog(result) = read_json_message(&self.endpoint.stream).unwrap() else {panic!()};
        match result {
            Ok(mut world) => self.component.world.extend(world.drain()),
            Err(e) => panic!("{}", e),
        }
        Ok(command)
    }
    pub fn listen_for_player_joins(&mut self) {
        let Some(Ok(Message::NewPlayer { starting_position, player })) = read_json_message_async(&self.endpoint.stream) else {return};
        self.component.world.gen_capital_at_cube(self.component.players.len(), starting_position);
        self.component.players.push(player);
    }
}

// pub trait Component {
//     type Swap;
//     fn poll(&mut self, layout: &mut Layout<f32>) -> bool;
//     fn draw(&self, layout: &Layout<f32>, assets: &Assets, time: f32);
//     fn update(&mut self);
//     fn swap(self) -> Self::Swap; //impl Component;
// }
pub trait Component<B: Backend>: IntoAny {
    // type Swap: Component;
    // fn poll(self: Box<Self>, layout: &mut Layout<f32>, backend: &B) -> (bool, Option<Box<dyn Component<B>>>);
    fn draw(&self, layout: &Layout<f32>, backend: &B, resources: &GameResources, time: f32);
    fn poll(&mut self, layout: &mut Layout<f32>, backend: &B) -> Message;
    fn update(self: Box<Self>, message: Message) -> Option<Box<dyn Component<B>>>;
    // fn swap(self) -> Self::Swap; //Box<dyn Component>;//impl Component;
    // fn swap(self: Box<Self>) -> Box<dyn Component<B>>;//Box<dyn Component>;//impl Component;
    // fn empty() -> Self;
}

#[derive(Debug)]
pub struct NetworkPlayer {
    pub name: String,
    pub is_ready: bool,
    pub faction_selection: Option<usize>,
}

impl Display for NetworkPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct Server {
    listener: TcpListener,
    streams: Vec<TcpStream>,
    players: Vec<NetworkPlayer>,
    pub chatlog: Vec<ChatMsg>,
}

impl Server {
    fn handle_client(&mut self) -> std::io::Result<()> {
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let player_idx = self.streams.len();
                    stream.set_nonblocking(true)?;
                    self.streams.insert(player_idx, stream);
                    println!("player {} joined", player_idx);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No more connections to accept right now
                    // break to prevent from blocking
                    break;
                }
                Err(e) => {
                    eprintln!("Failed to accept a connection: {:?}", e);
                }
            }
        }
        Ok(())
    }

    fn poll_all_streams(&mut self) {
        for (player, stream) in self.players.iter().zip(&self.streams) {
            // println!("listening for player index {}", idx);
            let Some(Ok(message)): Option<Result<Message, std::io::Error>> = read_json_message_async(stream) else {return};
        
            println!("received message from pid {:}: {:}", player, message);
    
            match message {
                Message::Chat(mut msg) => {
                    msg.author = format!("{:}", player);
                    msg.timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    self.chatlog.push(msg.clone());
                    write_json_message(stream, &Message::Chat(msg));
                },
                _ => {}
            }
        }
    }
}

impl SendChat for Server {
    fn send_chat_message(&mut self, msg: String) -> Result<(), Box<dyn std::error::Error>> {
        let author = "Server".to_string();
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let message = ChatMsg {body: msg, author, timestamp};
        for s in self.streams.iter() {
            write_json_message(s, &Message::Chat(message.clone()));
        };
        self.chatlog.push(message);
        Ok(()) // todo: handle one or more errors
    }
}

impl Server {
    pub fn new<A: std::net::ToSocketAddrs>(addr: A) -> Result<Self, std::io::Error>{
        let listener = std::net::TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let streams = vec!();
        let players = vec!();
        let chatlog = vec!();
        Ok(Self{listener, streams, players, chatlog})
        // let listeners: Result<Vec<_>, _> = addrs.iter().map(|a| {std::net::TcpListener::bind(a)}).collect();
        // Ok(Self{game, listeners: listeners?})
    }
}

/// Define a generic function to read and deserialize JSON messages
fn read_json_message(stream: &TcpStream) -> Result<Message, Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();

    // Read one line from the stream
    reader.read_line(&mut buffer)?;

    // Deserialize the JSON message
    let response = serde_json::from_str(&buffer.trim_end())?;
    println!("received {}", response);

    Ok(response)
}

// fn read_json_message_async<T>(stream: &TcpStream) -> Option<std::io::Result<Message>>
// where
//     T: serde::de::DeserializeOwned,
// {
fn read_json_message_async(stream: &TcpStream) -> Option<std::io::Result<Message>> {
    // Create a BufReader on the stream
    let mut reader = BufReader::new(stream);
    let mut buffer = String::new();

    // Attempt to read from the stream
    match reader.read_line(&mut buffer) {
        Ok(0) => {
            println!("Connection closed");
            None
        },
        Ok(_) => {
            // Deserialize the JSON message
            match serde_json::from_str(&buffer.trim_end()) {
                Ok(response) => {
                    println!("received {}", response);
                    Some(Ok(response))
                },
                Err(e) => Some(Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e))),
            }
        },
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            // Would block: No data available right now
            None
        },
        Err(e) => {
            // Other errors
            println!("{}", e);
            Some(Err(e))
        }
    }
}

// fn read_json_message_async<T>(stream: &TcpStream) -> Option<std::io::Result<String>>
// where
//     T: serde::de::DeserializeOwned,
// {
//     // Create a BufReader on the stream
//     let mut reader = BufReader::new(stream);
//     let mut buffer = String::new();

//     // Attempt to read from the stream
//     match reader.read_line(&mut buffer) {
//         Ok(0) => {
//             // Connection closed
//             None
//         },
//         Ok(_) => {
//             // Return raw message
//             Some(Ok(buffer))
//         },
//         Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
//             // Would block: No data available right now
//             None
//         },
//         Err(e) => {
//             // Other errors
//             println!("{}", e);
//             Some(Err(e))
//         }
//     }
// }

// fn write_json_message<T: Serialize>(stream: &TcpStream, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
pub fn write_json_message(stream: &TcpStream, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
    println!("sending {}", message);
    let mut writer = BufWriter::new(stream);

    let serialized_message = serde_json::to_string(&message)?;

    writer.write_all(serialized_message.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()?;

    println!("sent {}", message);
    Ok(())
}

// fn write_json_message(stream: &TcpStream, message: Message) -> Result<&'a T, Box<dyn std::error::Error>> {
//     let mut writer = BufWriter::new(stream);

//     let serialized_message = serde_json::to_string(&message)?;

//     writer.write_all(serialized_message.as_bytes())?;
//     writer.write_all(b"\n")?;
//     writer.flush()?;

//     Ok(message)
// }

// performing an action on the client side sends over a Command or a SkipTurn
// the server replies with the same message which the client parses
// and only then executes it.
#[derive(Serialize, Deserialize)]
pub enum Message {
    Tick,
    SetPlayer(Player),
    NewPlayer {starting_position: Cube<i32>, player: Player}, // todo: remove
    Initialise {turn: usize, players: Vec<Player>, world: World},
    Command(Command),
    RevealFog(Result<World, ServerResponseError>),
    SkipTurn,
    Chat(ChatMsg),
    Clear,
    ToggleLayer,
    Swap,
    Save,
    Load,
    Exit,
}

impl core::fmt::Display for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Message::Tick => write!(f, "Tick"),
            Message::SetPlayer{..} => write!(f, "SetPlayer"),
            Message::NewPlayer{..} => write!(f, "NewPlayer"),
            Message::Initialise{..} => write!(f, "Initialise"),
            Message::Command(_) => write!(f, "Command"),
            Message::RevealFog(_) => write!(f, "RevealFog"),
            Message::SkipTurn => write!(f, "SkipTurn"),
            Message::Chat{..} => write!(f, "Chat"),
            Message::Clear => write!(f, "Clear"),
            Message::ToggleLayer => write!(f, "ToggleLayer"),
            Message::Swap => write!(f, "Swap"),
            Message::Save => write!(f, "Save"),
            Message::Load => write!(f, "Load"),
            Message::Exit => write!(f, "Exit"),
        }
    }
}

// fn process_message(message: String) {
//     let s = message.trim_end();

//     let types: vec![Box::new(World), Player];

//     // Attempt to deserialize the message into each type and process it
//     for &ty in &types {
//         match serde_json::from_str::<&dyn JsonDeserializable>(message) {
//             Ok(obj) => {
//                 obj.process_message();
//                 return;
//             },
//             Err(_) => continue,
//         }
//     }

//     // Try to deserialize into type T
//     if let Ok(t_msg) = serde_json::from_str::<T>(s) {
//         println!("Received T message: {:?}", t_msg);
//         return;
//     }

//     // Try to deserialize into type U
//     if let Ok(u_msg) = serde_json::from_str::<U>(s) {
//         println!("Received U message: {:?}", u_msg);
//         return;
//     }

//     // Handle case where the message could not be deserialized into any known type
//     println!("Received unknown message type: {}", s);
// }

// impl Game {
//     pub fn new_for_client(self, player_idx: usize) -> &Self {
//         let mut player_fogs = HashMap::new();
//         player_fogs.insert(player_idx, *self.player_fogs.get(&player_idx).unwrap());
//         &Game {
//             turn: self.turn,
//             players: vec![*self.players.get(player_idx).unwrap()],
//             world: self.world,
//             player_fogs: player_fogs,
//             rules: self.rules,
//         }
//     }
// }

// A chatlog is just a vector of these structs
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMsg {
    body: String,
    author: String,
    timestamp: String,
}

impl fmt::Display for ChatMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let time = &self.timestamp[11..]; // skip date part
        write!(f, "[{}] {}: {}", time, self.author, self.body)
    }
}

impl ChatMsg {
    pub fn from_str(msg: &str) -> Self {
        Self {body: msg.to_string(), author: String::new(), timestamp: String::new()}
    }
}