use core::panic;
use std::alloc::Layout;
use std::collections::HashSet;
use dsl::pda;
use hashbrown::HashMap;
use iced_macroquad::iced::theme::Palette;
// use iced_macroquad::iced::advanced::Renderer;
use iced_macroquad::iced::widget::image;
use iced_macroquad::iced::widget::image::Handle;
use iced_macroquad::iced::Alignment::Center;
use macroquad::miniquad::conf::Platform;
// use iced_macroquad::iced::raw::Element;
// use iced_macroquad::iced::raw::Element;
// use pda::*;
use pda::PushdownAutomaton;
use strum::{AsStaticRef, EnumCount, IntoEnumIterator};

use crate::game::{Game, VictoryCondition};
use crate::map_editor::Editor;
use crate::mquad::Assets;
use crate::network::{Chat, ChatMsg, Client, Component, EndpointType, Mode, Offline, SendChat, Server};
use crate::rules::Ruleset;
use crate::world::r#gen::{CapitalsGen, LocalitiesGen, RiverGen, ShapeGen};
use crate::world::Player;
use crate::{next_frame, vec2, Vec2, FONT};

use iced_macroquad::{Interface};
use iced_macroquad::iced::{Color, Element, Length, Theme};
use iced_macroquad::iced::{font, font::Font};
use iced_macroquad::iced::widget::{container, Button, Checkbox, Column, Container, Renderer, Row, Text};
use iced_macroquad::iced::widget::{button, row, column, text, center, checkbox, text_input, scrollable, pick_list};

use macroquad::prelude::*;

trait UiScale {
    fn x(self) -> f32;
    fn y(self) -> f32;
    fn xy(self) -> f32;
}

impl UiScale for f32 {
    fn x(self) -> f32 {
        let s = screen_width() / 2560.0;
        self * s
    }
    fn y(self) -> f32 {
        let s = screen_height() / 1440.0;
        self * s
    }
    fn xy(self) -> f32 {
        let h = screen_height() / 1440.0;
        let w = screen_width() / 2560.0;
        self * h * w *2.
    }}

use iced_macroquad::iced::widget::scrollable::{Scrollable, Direction};

trait HorizontalScroll<'a, Message: 'a> {
    fn horizontal(self) -> Scrollable<'a, Message>;
}

impl<'a, Message: 'a> HorizontalScroll<'a, Message> for Scrollable<'a, Message> {
    fn horizontal(self) -> Scrollable<'a, Message> {
        self.direction(Direction::Horizontal(scrollable::Scrollbar::new()))
    }
}

fn scrollable_h<'a, Message: 'a>(
    content: impl Into<Element<'a, Message, Theme>>
) -> Scrollable<'a, Message, Theme> {
    use crate::ui::HorizontalScroll;
    scrollable(content).horizontal()
}

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
    MapgenTemplateNext,
    MapgenTemplatePrev,
    MapgenLocalitiesNext,
    MapgenLocalitiesPrev,
    MapgenCapitalsNext,
    MapgenCapitalsPrev,
    MapgenShapeNext,
    MapgenShapePrev,
    MapgenRiverNext,
    MapgenRiverPrev,
    FogOfWarToggled,
    SetScenario(String),
    GenerateMap,
    SaveMap,
    ToggleConfirmScenario,
    ToggleGenerateMap,
    AssignPlayer(usize, Player),
    Exit,
}

// #[derive(Default)]
// struct MapGenUi {
//     template: crate::gen::Template,
//     localities: LocalitiesGen,
//     capitals: CapitalsGen,
//     shape: ShapeGen,
//     river: RiverGen,
// }

// #[derive(Default)]
pub struct Ui {
    menu: PushdownAutomaton<State, Input, State>,
    ip_address: String,
    pub endpoint: Box<dyn Chat>,
    pub endpoint_type: EndpointType,
    chat_message: String,
    pub players: HashMap<usize, Player>,
    pub victory_condition: VictoryCondition,
    fog_of_war: bool,
    scenario: String,
    lhs_panel: Lhs_Panel,
    mapgen_template: Option<crate::gen::TEMPLATES>,
    mapgen_ui: crate::gen::Template,
    pub mapgen_map: Option<crate::World>,
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
        let players: Vec<Player> = ui.players.into_values().collect();
        Self::default(ui.victory_condition, &players)
    }
}

enum Lhs_Panel {
    ScenarioSelection,
    PlayerSelection,
    MapGeneration,
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
            players: HashMap::new(),
            victory_condition: VictoryCondition::Elimination,
            fog_of_war: false,
            scenario: String::new(),
            lhs_panel: Lhs_Panel::ScenarioSelection,
            mapgen_template: Some(crate::gen::TEMPLATES::default()),
            mapgen_ui: crate::gen::Template::default(),
            mapgen_map: None,
            thumb: None,
            exit: false,
        }
    }
}

// fn handle_spinner<T: IntoEnumIterator + Eq>(ui: &mut Ui, options: T, rev: bool) {
//     let i = match ui.mapgen_template {
//         None => 0,
//         Some(x) => T::iter().position(|t| t == x).unwrap() + 1,
//     };
//     ui.mapgen_template = if i == crate::gen::TEMPLATES::COUNT {
//         None
//     } else {Some(crate::gen::TEMPLATES::iter().cycle().nth(i+2).unwrap())};
//     ui.mapgen_ui = match &ui.mapgen_template {
//         None => crate::world::gen::CUSTOM,
//         Some(t) => t.get(),
//     };
// }

fn spinner<'a>(text_left: &'a str, text_middle: &'a str, msg_left: Message, msg_right: Message) -> Row<'a, Message> {
    row!(
        text(text_left).size(64.0.xy()).font(FONT_HANDLE),
        button(text("<").size(64.0.xy()).font(FONT_HANDLE)).on_press(msg_left),
        text(text_middle).size(64.0.xy()).font(FONT_HANDLE),
        button(text(">").size(64.0.xy()).font(FONT_HANDLE)).on_press(msg_right),
    ).spacing(20.0.x())
}

fn handle_spinner_field<T>(field: &mut Option<T>, rev: bool)
where
    T: IntoEnumIterator + PartialEq + Clone + strum::EnumCount + std::fmt::Debug,
    <T as IntoEnumIterator>::Iterator: Clone,
    <T as IntoEnumIterator>::Iterator: DoubleEndedIterator
{
    let i = match field {
        None => 0,
        Some(x) => {
            if rev {
                T::iter().rev().position(|t| std::mem::discriminant(&t) == std::mem::discriminant(x)).unwrap() + 1
            } else {
                println!("{:?}", x);
                T::iter().position(|t| std::mem::discriminant(&t) == std::mem::discriminant(x)).unwrap() + 1
            }
        },
    };

    let is_even = T::COUNT % 2 == 0;
    let offset = if is_even {0} else {1};

    *field = if i == T::COUNT {
        None
    } else {
        if rev {
            Some(T::iter().rev().cycle().nth(i + offset).unwrap())
        } else {
            Some(T::iter().cycle().nth(i + offset).unwrap())
        }
    };
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
            // Message::MapgenTemplateNext => {
            //     let i = match self.mapgen_template {
            //         None => 0,
            //         Some(x) => crate::gen::TEMPLATES::iter().position(|t| t == x).unwrap() + 1,
            //     };
            //     self.mapgen_template = if i == crate::gen::TEMPLATES::COUNT {
            //         None
            //     } else {Some(crate::gen::TEMPLATES::iter().cycle().nth(i+2).unwrap())};
            //     self.mapgen_ui = match &self.mapgen_template {
            //         None => None,
            //         Some(t) => Some(t.get()),
            //     };
            // }
            Message::MapgenTemplateNext => {
                handle_spinner_field(&mut self.mapgen_template, false);
                self.mapgen_ui = match &self.mapgen_template {
                    None => crate::world::gen::CUSTOM,
                    Some(t) => t.get(),
                };
            }
            Message::MapgenTemplatePrev => {
                handle_spinner_field(&mut self.mapgen_template, true);
                self.mapgen_ui = match &self.mapgen_template {
                    None => crate::world::gen::CUSTOM,
                    Some(t) => t.get(),
                };
            }
            Message::MapgenShapeNext => {
                handle_spinner_field(&mut self.mapgen_ui.shape, false);
                self.mapgen_template = None;
                self.mapgen_ui.name = "Custom";
            }
            Message::MapgenShapePrev => {
                handle_spinner_field(&mut self.mapgen_ui.shape, true);
                self.mapgen_template = None;
                self.mapgen_ui.name = "Custom";
            }
            Message::MapgenCapitalsNext => {
                handle_spinner_field(&mut self.mapgen_ui.capitals, false);
                self.mapgen_template = None;
                self.mapgen_ui.name = "Custom";
            }
            Message::MapgenCapitalsPrev => {
                handle_spinner_field(&mut self.mapgen_ui.capitals, true);
                self.mapgen_template = None;
                self.mapgen_ui.name = "Custom";
            }
            Message::MapgenLocalitiesNext => {
                handle_spinner_field(&mut self.mapgen_ui.localities, false);
                self.mapgen_template = None;
                self.mapgen_ui.name = "Custom";
            }
            Message::MapgenLocalitiesPrev => {
                handle_spinner_field(&mut self.mapgen_ui.localities, true);
                self.mapgen_template = None;
                self.mapgen_ui.name = "Custom";
            }
            Message::MapgenRiverNext => {
                handle_spinner_field(&mut self.mapgen_ui.rivers, false);
                self.mapgen_template = None;
                self.mapgen_ui.name = "Custom";
            }
            Message::MapgenRiverPrev => {
                handle_spinner_field(&mut self.mapgen_ui.rivers, true);
                self.mapgen_template = None;
                self.mapgen_ui.name = "Custom";
            }
            Message::FogOfWarToggled => {
                self.fog_of_war = !self.fog_of_war;
            }
            Message::SetScenario(scenario) => {
                self.scenario = scenario;
                let scaling = screen_width() / 2560.0; // Reference width for 2K (2560 pixels)
                let world = crate::World::from_json(&self.scenario);
                self.thumb = Some(get_map_thumb(&world, assets, scaling).await);
                self.mapgen_map = Some(world);
            }
            Message::ToggleConfirmScenario => {
                self.lhs_panel = match self.lhs_panel {
                    Lhs_Panel::PlayerSelection => Lhs_Panel::ScenarioSelection,
                    Lhs_Panel::ScenarioSelection => Lhs_Panel::PlayerSelection,
                    Lhs_Panel::MapGeneration => panic!()
                }
            }
            Message::ToggleGenerateMap => {
                self.lhs_panel = match self.lhs_panel {
                    Lhs_Panel::PlayerSelection => panic!(),
                    Lhs_Panel::ScenarioSelection => Lhs_Panel::MapGeneration,
                    Lhs_Panel::MapGeneration => Lhs_Panel::ScenarioSelection,
                }
            }
            Message::SaveMap => {
                std::fs::create_dir_all("assets/maps");
                match self.mapgen_map {
                    Some(world) => {world.to_json("assets/maps/mapgen.json")},
                    None => {},
                }
                self.mapgen_map = None;
            }
            Message::GenerateMap => {
                let mut world: crate::World = crate::World::new();

                let mut locality_names: Vec<&str> = assets.locality_names.iter().map(|s| s.as_str()).collect();

                world.generate_from_template(&self.mapgen_ui, &mut locality_names, &assets.init_layout);

                let scaling = screen_width() / 2560.0; // Reference width for 2K (2560 pixels)
                self.thumb = Some(get_map_thumb(&world, assets, scaling).await);

                self.mapgen_map = Some(world);
            }
            Message::Exit => {
                self.exit = true;
            }
            Message::AssignPlayer(idx, player) => {
                self.players.insert(idx, player);
            }
            _ => {}
        }
        self
    }
}

fn get_main_button(display_text: &str, message: Message) -> Button<Message> {
    Button::new(Text::new(display_text).size(64.0.xy()).center().font(FONT_HANDLE))
        .on_press(message)
        .width(Length::Fixed(500.0.xy()))
}

// fn get_sp_menu_old<'a>(state: &'a Ui) -> Container<'a, Message> {
//     let mut map_text = state.scenario.as_str();
//     if map_text.is_empty() {map_text = "Choose Map";}

//     let btn: Button<'_, Message> = match &state.thumb {
//         Some(handle) => {
//             let img: iced_macroquad::iced::widget::Image<Handle> = image(state.thumb.as_ref().unwrap()).into();
//             let btn = button(img);
//             btn
//         }
//         None => {
//             let btn = button(text(map_text).size(64.0.xy()).font(FONT_HANDLE).center());
//             btn
//         }
//     };

//     let m = center(column!()
//     .push(btn.on_press(Message::Transition(Input::ToMap)).width(900.0.x()).height(900.0.y()))
//     .push(checkbox("Fog of War", state.fog_of_war).on_toggle(|_| Message::FogOfWarToggled).font(FONT_HANDLE).text_size(64.0.xy()).size(64.0.xy()))
//     .push(row!(
//         text("Victory Condition:").size(64.0.xy()).font(FONT_HANDLE),
//         button(text("<").size(64.0.xy()).font(FONT_HANDLE)).on_press(Message::VictoryConditionPrev),
//         text(state.victory_condition.to_string()).size(64.0.xy()).font(FONT_HANDLE),
//         button(text(">").size(64.0.xy()).font(FONT_HANDLE)).on_press(Message::VictoryConditionNext),
//     ).spacing(20.0.x()))
//     .push(get_main_button("Play", Message::TransitionAndSetEndpoint(Input::ToGame, EndpointType::Offline)))
//     .spacing(20.0.y()));
//     m
// }

// fn get_sp_menu<'a>(state: &'a Ui) -> Container<'a, Message> {
fn get_sp_menu<'a>(state: &'a Ui, width: f32) -> Vec<Column<'a, Message>> {
    let mut map_text = state.scenario.as_str();
    if map_text.is_empty() {map_text = "Choose Map";}

    let btn: Button<'_, Message> = match &state.thumb {
        Some(handle) => {
            let img: iced_macroquad::iced::widget::Image<Handle> = image(state.thumb.as_ref().unwrap()).into();
            let btn = button(img);
            btn
        }
        None => {
            let btn = button(text(map_text).size(64.0.xy()).font(FONT_HANDLE).center());
            btn
        }
    };

    // let m = center(column!()
    // .push(btn.on_press(Message::Transition(Input::ToMap)).width(900.0.ui()).height(900.0.ui()))
    // .push(checkbox("Fog of War", state.fog_of_war).on_toggle(|_| Message::FogOfWarToggled).font(FONT_HANDLE).text_size(64.0.ui()).size(64.0.ui()))
    // .push(row!(
    //     text("Victory Condition:").size(64.0.ui()).font(FONT_HANDLE),
    //     button(text("<").size(64.0.ui()).font(FONT_HANDLE)).on_press(Message::VictoryConditionPrev),
    //     text(state.victory_condition.to_string()).size(64.0.ui()).font(FONT_HANDLE),
    //     button(text(">").size(64.0.ui()).font(FONT_HANDLE)).on_press(Message::VictoryConditionNext),
    // ).spacing(20.0.ui()))
    // .push(get_main_button("Play", Message::TransitionAndSetEndpoint(Input::ToGame, EndpointType::Offline)))
    // .spacing(20.0.ui()));

    fn text_std<'a>(t: impl text::IntoFragment<'a>) -> Text<'a> {
        text(t).size(64.0.xy()).font(FONT_HANDLE)
    }

    let lhs_buttons = match state.lhs_panel {
        Lhs_Panel::ScenarioSelection => row!(button(text_std("Select")).on_press(Message::ToggleConfirmScenario), button(text_std("Generate")).on_press(Message::ToggleGenerateMap)),
        Lhs_Panel::PlayerSelection => row!(button(text_std("Back")).on_press(Message::ToggleConfirmScenario), button(text_std("Play")).on_press(Message::TransitionAndSetEndpoint(Input::ToGame, EndpointType::Offline))),
        Lhs_Panel::MapGeneration => row!(button(text_std("Back")).on_press(Message::ToggleGenerateMap), button(text_std("Save")).on_press(Message::SaveMap)),
    };

    let scenario_panel= container(scrollable(Column::with_children(
        get_scenario_list().into_iter()
            .map(|r|
                button(text(r.path().file_stem().and_then(|s| Some(s.to_string_lossy().into_owned())).unwrap()).width(Length::Fill).center().font(FONT_HANDLE))
                .on_press(Message::SetScenario(r.path().display().to_string()))
                .width(Length::Fill)
                // .width(800.0.x())
                .into()
            )
        ))).style(container::rounded_box);
    
    //fn assign_player(player: Player)

    let mut player_slots: Vec<Element<Message, Theme>> = Vec::new();
    if let Some(world) = state.mapgen_map.as_ref() {
        // println!("1. world is some");
        let player_ids: Vec<usize> = world.cubes_by_ownership.keys().copied().collect();
        let players = vec![Player::new("Alicja", crate::Controller::Human), Player::new("AI", crate::Controller::AI(crate::ai::AI{scores: crate::DEFAULT_SCORES}))];
        // println!("2. player_ids:{:?}", player_ids);
        for pid in player_ids {
            // println!("3. pid={}", pid);
            match state.players.get(&pid) {
                Some(player) => {
                    // let text_str = player.name.as_str();
                    // let text_elem = text(text_str).font(FONT_HANDLE).width(Length::Fill);
                    //let e = button(text_elem);//.on_press(Message::AssignPlayer((), ()));
                    // let players: Vec<Player> = state.players.values().cloned().collect();
                    let e = pick_list(players.clone(), Some(player), {move |p| Message::AssignPlayer(pid, p)}).width(Length::Fill);
                    player_slots.push(e.into());
                    
                },
                None => {
                    // let text_str = format!("Assign Player {}", pid);
                    // let text_elem = text(text_str).font(FONT_HANDLE).width(Length::Fill);
                    // let e = button(text_elem);//.on_press(Message::AssignPlayer((), ()));
                    // player_slots.push(e.into());
                    // let players: Vec<Player> = state.players.values().cloned().collect();
                    let e = pick_list(players.clone(), None::<Player>, {move |p| Message::AssignPlayer(pid, p)}).width(Length::Fill);
                    player_slots.push(e.into());
                },
            }
        }
            // let player_slots = state.players.iter().map(|p| button(text(&p.name)).into());
    }
    // println!("player slot count: {:}", player_slots.iter().count());
    let player_panel = container(scrollable(Column::with_children(
        player_slots).spacing(20.0.y()))
            // state.players.iter().map(|p| row!(text(&p.name)).into())))
        // state.players.iter().map(|p| row!(text(&p.name)).into())))
    ).style(container::rounded_box).width(Length::Fill);

    //let gen_p = center(column!().push(row!()));

    // let template_text = match &state.mapgen_ui {
    //     None => "Custom",
    //     Some(t) => t.name,
    // };

    let template_text = state.mapgen_ui.name;

    fn to_str(opt: &Option<impl AsStaticRef<str>>) -> &'static str {
        opt.as_ref().map(|v| v.as_static()).unwrap_or("None")
    }

    fn get_template_param(ui: &Ui, param: crate::gen::TemplateFields) -> &str {
        match param {
            // Some(mui) => match param {
                crate::world::gen::TemplateFields::Name => ui.mapgen_ui.name,
                crate::world::gen::TemplateFields::Shape => to_str(&ui.mapgen_ui.shape),
                crate::world::gen::TemplateFields::Capitals => to_str(&ui.mapgen_ui.capitals),
                crate::world::gen::TemplateFields::Localities => to_str(&ui.mapgen_ui.localities),
                crate::world::gen::TemplateFields::Rivers => to_str(&ui.mapgen_ui.rivers),
            }
            // None => {"???"}
        // }
    }

    let shape_text = get_template_param(&state, crate::world::gen::TemplateFields::Shape);
    let capitals_text = get_template_param(&state, crate::world::gen::TemplateFields::Capitals);
    let localities_text = get_template_param(&state, crate::world::gen::TemplateFields::Localities);
    let rivers_text = get_template_param(&state, crate::world::gen::TemplateFields::Rivers);

    let gen_panel = container(scrollable(
    column!()
        .push(spinner("Template: ", template_text, Message::MapgenTemplatePrev, Message::MapgenTemplateNext))
        .push("")
        .push(spinner("Shape:        ", shape_text, Message::MapgenShapePrev, Message::MapgenShapeNext))
        .push(spinner("Capitals:    ", capitals_text, Message::MapgenCapitalsPrev, Message::MapgenCapitalsNext))
        .push(spinner("Localities:", localities_text, Message::MapgenLocalitiesPrev, Message::MapgenLocalitiesNext))
        .push(spinner("Rivers:        ", rivers_text, Message::MapgenRiverPrev, Message::MapgenRiverNext))
        .push(button(text("Generate").size(64.0.xy()).font(FONT_HANDLE)).on_press(Message::GenerateMap))
        .spacing(20.0.y()))

    ).style(container::rounded_box);

    let lhs_panel = match state.lhs_panel {
        Lhs_Panel::MapGeneration => gen_panel,
        Lhs_Panel::PlayerSelection => player_panel,
        Lhs_Panel::ScenarioSelection => scenario_panel,
    };

    let rhs_panel = container(scrollable(column!()
    .push(btn.on_press(Message::Transition(Input::ToMap)).width(1200.0.x()).height(1000.0.y()))
    .push(checkbox("Fog of War", state.fog_of_war).on_toggle(|_| Message::FogOfWarToggled).font(FONT_HANDLE).text_size(64.0.xy()).size(64.0.xy()))
    .push(spinner("Victory Condition:",state.victory_condition.as_static(), Message::VictoryConditionPrev, Message::VictoryConditionNext))
    )).style(container::rounded_box).center_x(1240.0.x());

    // let m = center(scrollable_h(row!()
    //     .push(
    //         column!(lhs_panel.height(1200.0.y()).width(1240.0.x()), lhs_buttons.width(1200.0.x()))
    //     )
    //     .push(
    //         column!(rhs_panel.height(1200.0.y()))
    //     )
    //     .spacing(20.0.x())).height(1800.0.y()).width(2500.0.x()));
    let m = vec![
        column!(lhs_panel.height(1200.0.y()).width(width.x()), lhs_buttons.width(width.x())),
        column!(rhs_panel.height(1200.0.y()).width(width.x()))
    ];
    m
}

fn get_mp_offline_menu<'a>(state: &Ui, scaling: f32) -> Column<'a, Message> {
    column!()
                .push(text_input(&format!("Address: {}", "127.0.0.1:8000"), &state.ip_address)
                        .size(64.0.xy())
                        .font(FONT_HANDLE)
                        .width(Length::Fixed(820.0.x()))
                        .on_input(&Message::IpAddressChanged))
                .push(row!()
                    .push(get_main_button("Join", Message::TransitionAndSetEndpoint(Input::ToLobby, EndpointType::Client)))
                    .push(get_main_button("Host", Message::TransitionAndSetEndpoint(Input::ToLobby, EndpointType::Server)))
                    .spacing(20.0.x())
                )
                .spacing(20.0.y())
}

fn get_mp_online_menu<'a>(state: &Ui, scaling: f32) -> Column<'a, Message> {
    let status_text = match state.endpoint_type {
        EndpointType::Client => "Connected to",
        EndpointType::Server => "Hosting at",
        EndpointType::Offline => unreachable!(),
    };
    column!()
                .push(text!("{} {}", status_text, &state.ip_address)
                        .size(64.0.xy())
                        .font(FONT_HANDLE)
                        .width(Length::Fixed(820.0.x())))
                .push(row!()
                    .push(get_main_button("Rejoin Lobby", Message::Transition(Input::ToLobby)))
                    .push(get_main_button("Disconnect", Message::CloseEndpoint))
                    .spacing(20.0.x())
                )
                .spacing(20.0.y())
}

// fn get_player_element<'a>(scaling: f32, player: &Player) -> Container<'a, Message, Theme, Renderer> {
//     container(text(player.to_string()))
// }

// fn get_players_list<'a>(scaling: f32, player: &Player) -> Container<'a, Message, Theme, Renderer> {
//     let x =     container(text(player.to_string()))
//     scrollable(x)
// }

fn get_scenario_list() -> Vec<std::fs::DirEntry> {
    std::fs::read_dir("./assets/maps")
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect()
}

/// Function to render a scene to an image
async fn get_map_thumb(world: &crate::World, assets: &Assets, scaling: f32) -> Handle {
    println!("getting thumb");
    // let scenario = Game::from_json(path);
    // let game = Game::from_json(path);
    // let scenario = game.swap();

    // texture size
    let width = 900.0.x(); // 2560;
    let height = 900.0.y(); // 1440;

    let mut layout = assets.init_layout.clone();

    // 2a. bounds with the original logical hex size (whatever init_layout is)
    let (min_x, min_y, max_x, max_y) = crate::mquad::map_bounds(world, &layout);
    let map_w = max_x - min_x;
    let map_h = max_y - min_y;


    // 2b. uniform scale that preserves aspect ratio
    let scale = (width / map_w).min(height / map_h);

    layout.size = [
        assets.init_layout.size[0] * scale,
        assets.init_layout.size[1] * scale,
    ];

    // 2c. shift so the (scaled) map is centred in the texture
    layout.origin = [
        (width  - map_w * scale) * 0.5 - min_x * scale,
        (height - map_h * scale) * 0.5 - min_y * scale,
    ];

    // Create a render target (offscreen texture)
    let render_target = render_target(width as u32, height as u32);

    let mut render_target_cam = Camera2D::from_display_rect(Rect::new(0., 0., width, height));
    render_target_cam.render_target = Some(render_target.clone());
    set_camera(&render_target_cam);

    // Draw something
    crate::mquad::draw_thumb(world, &layout, assets, 1.);

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

pub async fn main_menu<'a>(assets: &mut Assets) -> (bool, Ui) {
    let mut state = Ui::new();
    let mut interface = Interface::<Message>::new();

    let dark_palette = Palette {
        // background: Color::from_rgb8(0x18, 0x1B, 0x1F), // deep slate
        background: Color::from_rgb8(0x00, 0x00, 0x00), // deep slate
        text:       Color::WHITE,                      // high contrast
        primary:    Color::from_rgb8(0x2A, 0x2F, 0x36), // soft gunmetal
        success:    Color::from_rgb8(0xA0, 0xFF, 0xB0), // minty green
        danger:     Color::from_rgb8(0xFF, 0x5C, 0x5C), // coral red
    };
    let dark_theme = Theme::custom("Nightshade".to_string(), dark_palette);
    interface.set_theme(dark_theme);

    // interface.set_theme(Theme::Moonfly);
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

        // clear_background(LIGHTGRAY);
        let scaling = screen_width() / 2560.0; // Reference width for 2K (2560 pixels)

        let mut ui = match state.menu.get_state() {
            State::Main => center(scrollable(column!()
                .push(get_main_button("Play", Message::Transition(Input::ToSingle)))
                .push(get_main_button("Multiplayer", Message::Transition(Input::ToMulti)))
                .push(get_main_button("Settings", Message::Transition(Input::ToSettings)))
                .push(get_main_button("Exit", Message::Exit))
                .spacing(20.0.y()))
            ).into(),

            State::Single => {
                // let m = center(scrollable_h(row!()
                //     .push(
                //         column!(lhs_panel.height(1200.0.y()).width(1240.0.x()), lhs_buttons.width(1200.0.x()))
                //     )
                //     .push(
                //         column!(rhs_panel.height(1200.0.y()))
                //     )
                //     .spacing(20.0.x())).height(1800.0.y()).width(2500.0.x()));
                let cols = get_sp_menu(&state, 1200.0);
                let cols_e: Vec<Element<'_, Message, Theme>> = cols.into_iter().map(Into::into).collect();
                // center(scrollable_h(
                //         Row::with_children(cols_e).spacing(20.0.x())
                //     ).height(1800.0.y()).width(2500.0.x())
                // ).into()
                container(scrollable_h(
                        Row::with_children(cols_e).spacing(20.0.x())
                    ).height(1800.0.y()).width(2500.0.x())
                ).center_y(Length::Fixed(1600.0.y())).into()
            }

            State::Multi => match state.endpoint_type {
                EndpointType::Offline => center(get_mp_offline_menu(&state, scaling)).into(),
                _ => center(get_mp_online_menu(&state, scaling)).into(),
            } 

            State::Lobby => {
                let chatlog: &Vec<ChatMsg> = state.endpoint.get_chatlog();//state.endpoint.get_chatlog();
                let chatlog_: Vec<String> = chatlog.into_iter().map(|msg| msg.to_string()).collect();
                let slog: Vec<Text> = chatlog_.into_iter().map(|msg| Text::new(msg.clone()).size(64.0.xy()).into()).collect();
                let elog = slog.into_iter().map(|t| <Text<'_, Theme, Renderer> as Into<Element<Message, Theme>>>::into(t));
                // let elog = slog.iter().map(|t| t.into());

                let chat_column = Column::with_children(
                    // chatlog.iter().map(|msg| Element::from(Text::from(msg.to_string().as_str()))).collect::<Vec<Element<_, _>>>()
                    elog
                    // todo!()
                );
                // column!(lhs_panel.height(1200.0.y()).width(1240.0.x()), lhs_buttons.width(1200.0.x())),
                // column!(rhs_panel.height(1200.0.y()))
                let mut sp = get_sp_menu(&state, 800.0);
                // sp[0] = std::mem::replace(&mut sp[0], Column::new()).width(800.0.x());
                // sp[1] = std::mem::replace(&mut sp[0], Column::new()).width(800.0.x());
                let sp_e: Vec<Element<'_, Message, Theme>> = sp.into_iter().map(Into::into).collect();
                center(
                    //scrollable_h(
                    row!()
                        .push(
                            column!()
                                .push(container(scrollable(chat_column)).height(1250.0.y()))
                                .push(
                                    text_input("press ENTER to send", &state.chat_message)
                                        .on_input(Message::ChatMessageChanged)
                                        .on_submit(Message::SendChatMessage)
                                        .size(64.0.xy()),
                                ).width(800.0.x())
                        )
                        .push(
                            column!().push(scrollable(Column::with_children(
                                state
                                    .players
                                    .iter()
                                    .map(|(i, p)| container(text(p.to_string())).into()),
                            ))),
                        )
                        .extend(sp_e)
                        .spacing(20.0.x())
                )
                .into()
            },

            State::Map => {column!()
                .push(get_main_button("Generate New", Message::Transition(Input::ToMapgen)))
                .push(container(scrollable(Column::with_children(
                    get_scenario_list().into_iter()
                        .map(|r|
                            button(text(r.file_name().into_string().unwrap()))
                            .on_press(Message::SetScenario(r.path().display().to_string()))
                            .into()
                        )
                    ))).height(1500.0.x()))
                .into()
            }

            State::Game => {
                // if matches!(state.endpoint_type, EndpointType::Server) {
                //     state.endpoint.send
                // }
                break
            }
            _ => {panic!("menu panic");}
        };

        let dark_rounded_style = move |_theme: &Theme| crate::ui::container::Style {
            // solid black background
            background: Some(iced_macroquad::iced::Background::Color(Color::BLACK)),
            // white text so labels are readable
            text_color:  Some(Color::WHITE),
            ..crate::ui::container::Style::default()
        };

        ui = Container::new(ui)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(dark_rounded_style)
        .into();

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

// mod style {
//     use iced_macroquad::iced::{widget::container, Color};

//     pub struct Container;

//     impl container::StyleSheet for Container {
//         fn style(&self) -> container::Style {
//             container::Style {
//                 border_width: 2.0,
//                 border_color: Color::BLACK,
//                 background: Some(Color::from_rgb(0.9, 0.9, 0.9).into()),
//                 ..Default::default()
//             }
//         }
//     }
// }
