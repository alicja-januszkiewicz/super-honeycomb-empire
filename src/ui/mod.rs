use core::panic;
use std::collections::HashSet;
use dsl::pda;
use hashbrown::HashMap;
// use iced_macroquad::iced::raw::Element;
// use pda::*;
use pda::PushdownAutomaton;
use strum::IntoEnumIterator;

use crate::game::{Game, VictoryCondition};
use crate::mquad::Assets;
use crate::network::{ChatMsg, Client, Component, EndpointType, Chat, Mode, Offline, SendChat, Server};
use crate::rules::Ruleset;
use crate::world::Player;
use crate::{next_frame, vec2, Vec2, FONT};

use iced_macroquad::{Interface};
use iced_macroquad::iced::{Element, Length, Theme};
use iced_macroquad::iced::{font, font::Font};
use iced_macroquad::iced::widget::{Button, Checkbox, Column, Container, Renderer, Row, Text};
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
            Single => Map,

    Main => Multi => Lobby => Game,
                     Lobby => Map,

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
    VictoryConditionNext,
    VictoryConditionPrev,
    FogOfWarToggled,
    Exit,
}

// #[derive(Default)]
pub struct Ui {
    menu: PushdownAutomaton<State, Input, State>,
    ip_address: String,
    pub endpoint: Box<dyn Chat>,
    chat_message: String,
    pub players: Vec<Player>,
    victory_condition: VictoryCondition,
    fog_of_war: bool,
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
            chat_message: "".into(),
            players: vec![],
            victory_condition: VictoryCondition::Elimination,
            fog_of_war: false,
            exit: false,
        }
    }
}
impl Ui {
    fn update(&mut self, message: Message) {
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
                // if self.endpoint.is_some() | endpoint.is_none() {return}
                // self.endpoint = endpoint;
                self.endpoint = match endpoint_type {
                    EndpointType::Client => Box::new(Client::new(&self.ip_address).unwrap()),
                    EndpointType::Server => Box::new(Server::new(&self.ip_address).unwrap()),
                    EndpointType::Offline => Box::new(Offline),
                };
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
            Message::Exit => {
                self.exit = true;
            }
        }
    }
}

fn get_main_button(display_text: &str, message: Message) -> Button<Message> {
    let scaling = screen_width() / 2560.0; // Reference width for 2K (2560 pixels)

    Button::new(Text::new(display_text).size(64. * scaling).center().font(FONT_HANDLE))
        .on_press(message)
        .width(Length::Fixed(400. * scaling))
}

fn get_sp_menu<'a>(state: &'a Ui) -> Container<'a, Message> {
    let scaling = screen_width() / 2560.0; // Reference width for 2K (2560 pixels)

    let m = center(column!()
    .push(button(text("Choose Map").size(64. * scaling).font(FONT_HANDLE).center()).on_press(Message::Transition(Input::ToMap)).width(900. * scaling).height(900. * scaling))
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
            state.update(message);
        }

        clear_background(LIGHTGRAY);

        let scaling = screen_width() / 2560.0; // Reference width for 2K (2560 pixels)

        let ui = match state.menu.get_state() {
            State::Main => center(column!()
                .push(get_main_button("Play", Message::Transition(Input::ToSingle)))
                .push(get_main_button("Multiplayer", Message::Transition(Input::ToMulti)))
                .push(get_main_button("Settings", Message::Transition(Input::ToSettings)))
                .push(get_main_button("Exit", Message::Exit))
                .spacing(20)
            ).into(),

            State::Single => get_sp_menu(&state).into(), //.center(Length::Fill)

            State::Multi => center(column!()
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
                ).into(),

            State::Lobby => {
                let chatlog: &Vec<ChatMsg> = state.endpoint.get_chatlog();//state.endpoint.get_chatlog();
                let chatlog_: Vec<String> = chatlog.into_iter().map(|msg| msg.to_string()).collect();
                let slog: Vec<Text> = chatlog_.into_iter().map(|msg| Text::new(msg.clone()).into()).collect();
                let elog = slog.into_iter().map(|t| <Text<'_, Theme, Renderer> as Into<Element<Message, Theme>>>::into(t));
                // let elog = slog.iter().map(|t| t.into());

                let chat_column = Column::with_children(
                    // chatlog.iter().map(|msg| Element::from(Text::from(msg.to_string().as_str()))).collect::<Vec<Element<_, _>>>()
                    elog
                    // todo!()
                );
            center(row![
                column!()
                    .push(scrollable(chat_column))
                    .push(text_input("press ENTER to send", &state.chat_message)
                            .on_input(Message::ChatMessageChanged)
                            .on_submit(Message::SendChatMessage)),
                get_sp_menu(&state),
            ]).into()},

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
