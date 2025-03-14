use core::panic;
use std::alloc::Layout;
use std::collections::HashSet;
use dsl::pda;
use hashbrown::HashMap;
use iced_macroquad::iced::widget::image;
use iced_macroquad::iced::widget::image::Handle;
// use iced_macroquad::iced::raw::Element;
// use iced_macroquad::iced::raw::Element;
// use pda::*;
use pda::PushdownAutomaton;
use strum::IntoEnumIterator;

use crate::game::{Game, VictoryCondition};
use crate::map_editor::Editor;
use crate::mquad::Assets;
use crate::network::{Chat, ChatMsg, Client, Component, EndpointType, Mode, Offline, SendChat, Server};
use crate::rules::Ruleset;
use crate::world::Player;
use crate::{next_frame, vec2, Vec2, FONT};

use iced_macroquad::{Interface};
use iced_macroquad::iced::{Element, Length, Theme};
use iced_macroquad::iced::{font, font::Font};
use iced_macroquad::iced::widget::{container, Button, Checkbox, Column, Container, Renderer, Row, Text};
use iced_macroquad::iced::widget::{button, row, column, text, center, checkbox, text_input, scrollable};

use macroquad::prelude::*;

// use macroquad::ui::{hash, root_ui};
// use macroquad::ui::widgets;
// use macroquad::ui::Skin;
// use macroquad::ui::widgets::Group;
// use macroquad::prelude::Color;
// use macroquad::prelude::RectOffset;
// use macroquad::prelude::load_ttf_font;

// use iced::{
//     Element, Application, Settings as IcedSettings, Length
// };

// use macroquad::prelude::*;

// use iced_macroquad::iced;
// extern crate glow;
// use iced_glow::{Renderer, Application, Settings};

// struct UI;

// impl Application for UI {
//     type Executor = iced_native::executor::Null;
//     type Message = (); // No messages for now
//     type Theme = iced_native::theme::Theme;
//     type Flags = ();

//     fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
//         (Self, Command::none())
//     }

//     fn title(&self) -> String {
//         "Iced in Macroquad".to_string()
//     }

//     fn update(&mut self, _message: Self::Message, _clipboard: &mut dyn Clipboard) -> Command<Self::Message> {
//         Command::none()
//     }

//     fn view(&self) -> iced_native::Element<'_, Self::Message, Renderer> {
//         use iced_native::widget::{column, text};
//         column![text("Hello, Iced in Macroquad!")].into()
//     }
// }

const FONT_HANDLE: Font = Font::with_name("Iceberg");

pda! {
    Main => Single => Game,
    Main => Single => Map => Mapgen,

    Main => Multi => Lobby => Game,
    Main => Multi => Lobby => Map => Mapgen,

    Main => Settings,
}

// #[derive(Debug, Clone)]
// enum EndpointType {
//     // Null,
//     Client,
//     Server,
// }

#[derive(Debug, Clone)]
enum Message {
    Transition(Input),
    IpAddressChanged(String),
    ChatMessageChanged(String),
    SendChatMessage,
    TransitionAndSetEndpoint(Input, EndpointType),
    CloseEndpoint,
    VictoryConditionNext,
    VictoryConditionPrev,
    FogOfWarToggled,
    SetScenario(String),
    ToggleConfirmScenario,
    Exit,
}

// #[derive(Default)]
pub struct Ui {
    menu: PushdownAutomaton<State, Input, State>,
    ip_address: String,
    pub endpoint: Box<dyn Chat>,
    pub endpoint_type: EndpointType,
    chat_message: String,
    pub players: Vec<Player>,
    pub victory_condition: VictoryCondition,
    fog_of_war: bool,
    scenario: String,
    scenario_selected: bool,
    thumb: Option<Handle>,
    exit: bool,
}

// pub enum Endpoint {
//     Client(Client),
//     Server(Server),
//     Offline(),
// }

// impl Endpoint {
//     fn send_chat_message(&mut self, msg: String) {
//         match self {
//             Endpoint::Client(c) => c.send_chat_message(msg),
//             Endpoint::Server(s) => s.send_chat_message(msg),
//         };
//     }
// }

impl From<Ui> for Ruleset {
    fn from(ui: Ui) -> Self {
        Self::default(ui.victory_condition, &ui.players)
    }
}

impl Ui {
    fn new() -> Self {
        let transitions = generate_transitions();
        let start_state = &State::Main;
        let final_states = HashSet::new();
        let mut pda = PushdownAutomaton::new(start_state, final_states, transitions);
        Self {
            menu: pda,
            ip_address: "127.0.0.1:8000".into(),
            endpoint: Box::new(Offline),
            endpoint_type: EndpointType::Offline,
            chat_message: "".into(),
            players: vec![],
            victory_condition: VictoryCondition::Elimination,
            fog_of_war: false,
            scenario: String::new(),
            scenario_selected: false,
            thumb: None,
            exit: false,
        }
    }
}
impl Ui {
    async fn update(mut self, message: Message, assets: &Assets) -> Self {
        match message {
            Message::Transition(input) => {
                self.menu.transition(input);
            }
            Message::IpAddressChanged(addr) => {
                self.ip_address = addr;
            }
            Message::ChatMessageChanged(msg) => {
                // println!("chat message changed!!!: `{}`", msg);
                self.chat_message = msg
            }
            Message::SendChatMessage => {
                let msg = std::mem::take(&mut self.chat_message);
                println!("sending chat message: {:}", msg);
                self.endpoint.send_chat_message(msg);
            }
            Message::TransitionAndSetEndpoint(input, endpoint_type) => {
                self.menu.transition(input);
                self.endpoint = match endpoint_type {
                    EndpointType::Client => Box::new(Client::new(&self.ip_address).unwrap()),
                    EndpointType::Server => Box::new(Server::new(&self.ip_address).unwrap()),
                    EndpointType::Offline => Box::new(Offline),
                };
                self.endpoint_type = endpoint_type;
            }
            Message::CloseEndpoint => {
                self.endpoint.close();
                self.endpoint = Box::new(Offline);
                self.endpoint_type = EndpointType::Offline;
            }
            Message::VictoryConditionNext => {
                let i = VictoryCondition::iter().position(|vc| vc == self.victory_condition).unwrap();
                self.victory_condition = VictoryCondition::iter().cycle().nth(i+1).unwrap();
            }
            Message::VictoryConditionPrev => {
                let i = VictoryCondition::iter().rev().position(|vc| vc == self.victory_condition).unwrap();
                self.victory_condition = VictoryCondition::iter().rev().cycle().nth(i+1).unwrap();
                //self.victory_condition = VictoryCondition::iter().rev().cycle().skip_while(|vc| *vc == self.victory_condition).next().unwrap();
                // let n = VictoryCondition::iter().skip_while(|vc| *vc == self.victory_condition).count();
                // let n = if n == 0 {VictoryCondition::iter().len()} else {n - 1};
                // self.victory_condition = VictoryCondition::iter().skip(n).next().unwrap();
            }
            Message::FogOfWarToggled => {
                self.fog_of_war = !self.fog_of_war;
            }
            Message::SetScenario(scenario) => {
                self.scenario = scenario;
                let scaling = screen_width() / 2560.0; // Reference width for 2K (2560 pixels)
                self.thumb = Some(get_scenario_thumb(&self.scenario, assets, scaling).await);
            }
            Message::ToggleConfirmScenario => {
                self.scenario_selected = !self.scenario_selected;
            }
            Message::Exit => {
                self.exit = true;
            }
        }
        self
    }
}

fn get_main_button(display_text: &str, message: Message) -> Button<Message> {
    let scaling = screen_width() / 2560.0; // Reference width for 2K (2560 pixels)

    Button::new(Text::new(display_text).size(64. * scaling).center().font(FONT_HANDLE))
        .on_press(message)
        .width(Length::Fixed(400. * scaling))
}

fn get_sp_menu_old<'a>(state: &'a Ui) -> Container<'a, Message> {
    let scaling = screen_width() / 2560.0; // Reference width for 2K (2560 pixels)

    let mut map_text = state.scenario.as_str();
    if map_text.is_empty() {map_text = "Choose Map";}

    let btn: Button<'_, Message> = match &state.thumb {
        Some(handle) => {
            let img: iced_macroquad::iced::widget::Image<Handle> = image(state.thumb.as_ref().unwrap()).into();
            let btn = button(img);
            btn
        }
        None => {
            let btn = button(text(map_text).size(64. * scaling).font(FONT_HANDLE).center());
            btn
        }
    };

    let m = center(column!()
    .push(btn.on_press(Message::Transition(Input::ToMap)).width(900. * scaling).height(900. * scaling))
    .push(checkbox("Fog of War", state.fog_of_war).on_toggle(|_| Message::FogOfWarToggled).font(FONT_HANDLE).text_size(64. * scaling).size(64. * scaling))
    .push(row!(
        text("Victory Condition:").size(64. * scaling).font(FONT_HANDLE),
        button(text("<").size(64. * scaling).font(FONT_HANDLE)).on_press(Message::VictoryConditionPrev),
        text(state.victory_condition.to_string()).size(64. * scaling).font(FONT_HANDLE),
        button(text(">").size(64. * scaling).font(FONT_HANDLE)).on_press(Message::VictoryConditionNext),
    ).spacing(20. * scaling))
    .push(get_main_button("Play", Message::TransitionAndSetEndpoint(Input::ToGame, EndpointType::Offline)))
    .spacing(20. * scaling));
    m
}

fn get_sp_menu<'a>(state: &'a Ui) -> Row<'a, Message> {
    let scaling = screen_width() / 2560.0; // Reference width for 2K (2560 pixels)

    let mut map_text = state.scenario.as_str();
    if map_text.is_empty() {map_text = "Choose Map";}

    let btn: Button<'_, Message> = match &state.thumb {
        Some(handle) => {
            let img: iced_macroquad::iced::widget::Image<Handle> = image(state.thumb.as_ref().unwrap()).into();
            let btn = button(img);
            btn
        }
        None => {
            let btn = button(text(map_text).size(64. * scaling).font(FONT_HANDLE).center());
            btn
        }
    };

    // let m = center(column!()
    // .push(btn.on_press(Message::Transition(Input::ToMap)).width(900. * scaling).height(900. * scaling))
    // .push(checkbox("Fog of War", state.fog_of_war).on_toggle(|_| Message::FogOfWarToggled).font(FONT_HANDLE).text_size(64. * scaling).size(64. * scaling))
    // .push(row!(
    //     text("Victory Condition:").size(64. * scaling).font(FONT_HANDLE),
    //     button(text("<").size(64. * scaling).font(FONT_HANDLE)).on_press(Message::VictoryConditionPrev),
    //     text(state.victory_condition.to_string()).size(64. * scaling).font(FONT_HANDLE),
    //     button(text(">").size(64. * scaling).font(FONT_HANDLE)).on_press(Message::VictoryConditionNext),
    // ).spacing(20. * scaling))
    // .push(get_main_button("Play", Message::TransitionAndSetEndpoint(Input::ToGame, EndpointType::Offline)))
    // .spacing(20. * scaling));

    let lhs_buttons = match state.scenario_selected {
        false => row!(button("Select").on_press(Message::ToggleConfirmScenario)),
        true => row!(button("Back").on_press(Message::ToggleConfirmScenario), get_main_button("Play", Message::TransitionAndSetEndpoint(Input::ToGame, EndpointType::Offline))),
    };

    let scenario_panel= container(scrollable(Column::with_children(
        get_scenario_list().into_iter()
            .map(|r|
                button(text(r.file_name().into_string().unwrap()))
                .on_press(Message::SetScenario(r.path().display().to_string()))
                .into()
            )
        ))).height(1500. * scaling);
    let player_panel = container(scrollable(Column::with_children(state.players.iter().map(|p| row!(text(&p.name)).into()))));

    let lhs_panel = match state.scenario_selected {
        false => scenario_panel,
        true => player_panel,
    };

    let m = row!()
        .push(
            column!(lhs_panel, lhs_buttons)
        )
        .push(
            column!()
                .push(btn.on_press(Message::Transition(Input::ToMap)).width(900. * scaling).height(900. * scaling))
        );
    m
}

fn get_mp_offline_menu<'a>(state: &Ui, scaling: f32) -> Column<'a, Message> {
    column!()
                .push(text_input(&format!("Address: {}", "127.0.0.1:8000"), &state.ip_address)
                        .size(64)
                        .font(FONT_HANDLE)
                        .width(Length::Fixed(820.* scaling))
                        .on_input(&Message::IpAddressChanged))
                .push(row!()
                    .push(get_main_button("Join", Message::TransitionAndSetEndpoint(Input::ToLobby, EndpointType::Client)))
                    .push(get_main_button("Host", Message::TransitionAndSetEndpoint(Input::ToLobby, EndpointType::Server)))
                    .spacing(20. * scaling)
                )
                .spacing(20. * scaling)
}

fn get_mp_online_menu<'a>(state: &Ui, scaling: f32) -> Column<'a, Message> {
    let status_text = match state.endpoint_type {
        EndpointType::Client => "Connected to",
        EndpointType::Server => "Hosting at",
        EndpointType::Offline => unreachable!(),
    };
    column!()
                .push(text!("{} {}", status_text, &state.ip_address)
                        .size(64)
                        .font(FONT_HANDLE)
                        .width(Length::Fixed(820.* scaling)))
                .push(row!()
                    .push(get_main_button("Rejoin Lobby", Message::Transition(Input::ToLobby)))
                    .push(get_main_button("Disconnect", Message::CloseEndpoint))
                    .spacing(20. * scaling)
                )
                .spacing(20. * scaling)
}

// fn get_player_element<'a>(scaling: f32, player: &Player) -> Container<'a, Message, Theme, Renderer> {
//     container(text(player.to_string()))
// }

// fn get_players_list<'a>(scaling: f32, player: &Player) -> Container<'a, Message, Theme, Renderer> {
//     let x =     container(text(player.to_string()))
//     scrollable(x)
// }

fn get_scenario_list() -> Vec<std::fs::DirEntry> {
    std::fs::read_dir("./assets/scenarios")
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect()
}

/// Function to render a scene to an image
async fn get_scenario_thumb(path: &str, assets: &Assets, scaling: f32) -> Handle {
    println!("getting thumb");
    let scenario = Game::from_json(path);
    // let game = Game::from_json(path);
    // let scenario = game.swap();

    let width = 900. * scaling; // 2560;
    let height = 900. * scaling; // 1440;
    // let width = 2560.;
    // let height = 1440.;
    // let width = 2304.;//2560.;
    // let height = 1296.;//440.;

    // Create a render target (offscreen texture)
    let render_target = render_target(width as u32, height as u32);
    // let render_target = render_target(2560, 1440);

    let mut render_target_cam = Camera2D::from_display_rect(Rect::new(0., 0., width, height));
    render_target_cam.render_target = Some(render_target.clone());
    set_camera(&render_target_cam);

    // Draw something
    let mut layout = assets.init_layout.clone();
    layout.size = [4.,4.];
    scenario.draw(&layout, assets, 1.);

    set_default_camera();

    // Retrieve the pixel data
    let image_data = render_target.texture.get_texture_data();

    // (Optional) Save the image
    image_data.export_png("output.png");

    //image_data
    let mut raw_bytes = image_data.bytes;
    flip_image_vertically(&mut raw_bytes, width as usize, height as usize);
    Handle::from_rgba(width as u32, height as u32, raw_bytes) // Convert to Iced Image Handle
}

fn flip_image_vertically(raw_bytes: &mut [u8], width: usize, height: usize) {
    let row_size = width * 4; // 4 bytes per pixel (RGBA)
    for y in 0..(height / 2) {
        let top_index = y * row_size;
        let bottom_index = (height - 1 - y) * row_size;
        for i in 0..row_size {
            raw_bytes.swap(top_index + i, bottom_index + i);
        }
    }
}
pub async fn main_menu(assets: &mut Assets) -> (bool, Ui) {
    let mut state = Ui::new();
    let mut interface = Interface::<Message>::new();
    // interface.set_theme(iced::Theme::Oxocarbon);
    let mut messages = Vec::new();

    font::load(vec![FONT.into()]);

    while !state.exit {
        // poll
        if is_key_pressed(KeyCode::Escape) {
            messages.push(Message::Transition(Input::Back));
        }

        for message in messages.drain(..) {
            state = state.update(message, assets).await;
        }

        state.endpoint.update();

        clear_background(LIGHTGRAY);
        let scaling = screen_width() / 2560.0; // Reference width for 2K (2560 pixels)

        let ui = match state.menu.get_state() {
            State::Main => center(column!()
                .push(get_main_button("Play", Message::Transition(Input::ToSingle)))
                .push(get_main_button("Multiplayer", Message::Transition(Input::ToMulti)))
                .push(get_main_button("Settings", Message::Transition(Input::ToSettings)))
                .push(get_main_button("Exit", Message::Exit))
                .spacing(20. * scaling)
            ).into(),

            State::Single => get_sp_menu(&state).into(), //.center(Length::Fill)

            State::Multi => match state.endpoint_type {
                EndpointType::Offline => center(get_mp_offline_menu(&state, scaling)).into(),
                _ => center(get_mp_online_menu(&state, scaling)).into(),
            } 

            State::Lobby => {
                let chatlog: &Vec<ChatMsg> = state.endpoint.get_chatlog();//state.endpoint.get_chatlog();
                let chatlog_: Vec<String> = chatlog.into_iter().map(|msg| msg.to_string()).collect();
                let slog: Vec<Text> = chatlog_.into_iter().map(|msg| Text::new(msg.clone()).size(64. * scaling).into()).collect();
                let elog = slog.into_iter().map(|t| <Text<'_, Theme, Renderer> as Into<Element<Message, Theme>>>::into(t));
                // let elog = slog.iter().map(|t| t.into());

                let chat_column = Column::with_children(
                    // chatlog.iter().map(|msg| Element::from(Text::from(msg.to_string().as_str()))).collect::<Vec<Element<_, _>>>()
                    elog
                    // todo!()
                );
            center(row![
                column!()
                    .push(container(scrollable(chat_column)).height(1700. * scaling))
                    .push(text_input("press ENTER to send", &state.chat_message)
                            .on_input(Message::ChatMessageChanged)
                            .on_submit(Message::SendChatMessage)
                            ),
                column!().push(scrollable(Column::with_children(state.players.iter().map(|p| container(text(p.to_string())).into())))),
                get_sp_menu(&state),
            ]).into()},

            State::Map => {column!()
                .push(get_main_button("Generate New", Message::Transition(Input::ToMapgen)))
                .push(container(scrollable(Column::with_children(
                    get_scenario_list().into_iter()
                        .map(|r|
                            button(text(r.file_name().into_string().unwrap()))
                            .on_press(Message::SetScenario(r.path().display().to_string()))
                            .into()
                        )
                    ))).height(1500. * scaling))
                .into()
            }

            State::Game => {
                break
            }
            _ => {panic!("menu panic");}
        };

        interface.view(&mut messages, ui);

        next_frame().await
    }

    (state.exit, state)
}

use rust_fsm::*;


pub fn test_ui() {
    pda! {
        Main => Single => Game,
        Main => Multi => Host => Game,
        Main => Multi => Join => Lobby,
        Main => Settings,
    }

    let transitions = generate_transitions();
    println!("{:?}", transitions);

    let start_state = &State::Main;
    let final_states = HashSet::new();
    let mut pda = PushdownAutomaton::new(start_state, final_states, transitions);


    // Test transitions
    println!("0 {:?}", pda.get_state());
    assert_eq!(pda.transition(Input::ToSingle), Ok(()));
    println!("1 {:?}", pda.get_state());
    assert_eq!(pda.transition(Input::ToGame), Ok(()));
    println!("2 {:?}", pda.get_state());
    assert_eq!(pda.transition(Input::Back), Ok(()));
    println!("3 {:?}", pda.get_state());
    assert_eq!(pda.transition(Input::Back), Ok(()));
    println!("4 {:?}", pda.get_state());
    assert_eq!(pda.transition(Input::ToMulti), Ok(()));
    println!("5 {:?}", pda.get_state());
    assert_eq!(pda.transition(Input::ToHost), Ok(()));
    println!("5 {:?}", pda.get_state());
    assert_eq!(pda.transition(Input::ToGame), Ok(()));
    println!("7 {:?}", pda.get_state());
    assert_eq!(pda.transition(Input::Back), Ok(()));
    println!("8 {:?}", pda.get_state());
    assert_eq!(pda.transition(Input::Back), Ok(()));
    println!("9 {:?}", pda.get_state());
    // assert_eq!(pda.transition(Input::ToSettings), Ok(()));
    // println!("10 {:?}", pda.get_state());
}
