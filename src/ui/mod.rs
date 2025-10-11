use core::panic;
// use std::alloc::Layout;
use std::collections::HashSet;
use std::os::unix::net;
use dsl::pda;
use hashbrown::HashMap;
use wgpu::core::resource;
use wgpu::naga::back;

use crate::Backend;

// use pda::*;
use pda::PushdownAutomaton;
use strum::{AsStaticRef, EnumCount, IntoEnumIterator};

use crate::game::{Game, GameResources, VictoryCondition};
use crate::map_editor::Editor;
use crate::network::{self, App, Chat, ChatMsg, Client, Component, EndpointType, ErasedComponent, IntoAny, Mode, Offline, SendChat, Server};
use crate::rules::Ruleset;
use crate::world::r#gen::{CapitalsGen, LocalitiesGen, RiverGen, ShapeGen};
use crate::world::Player;
use crate::FONT;
use crate::cubic::Layout;

// #[cfg(feature = "wgpu")]
// pub use iced::*;

// #[cfg_attr(any(feature="mquad", rust_analyzer), path = "mquad/mod.rs")]
// #[cfg(feature = "mquad")]
// pub use iced_macroquad::iced;
// pub use iced_macroquad::iced::*;

// #[cfg(feature = "wgpu")]
// pub use iced as iced;

// #[cfg(feature = "mquad")]
// pub use iced_macroquad::iced as iced;

pub use iced_macroquad::iced as iced;


use iced::Alignment::Center;
use iced::{Color, Element, Length};
use iced::{font, font::Font};
use iced::theme::{Theme, Palette};
use iced::widget::{container, Button, Checkbox, Column, Container, Renderer, Row, Text};
use iced::widget::{button, row, column, text, center, checkbox, text_input, scrollable, pick_list};
use iced::widget::scrollable::{Scrollable, Direction};
use iced::widget::image;
use iced::widget::image::Handle;

// use crate::{next_frame, vec2, Vec2};

// use iced_macroquad::{Interface};
// use macroquad::miniquad::conf::Platform;

// use macroquad::prelude::*;

use std::cell::RefCell;

struct FrameContext {
    width: f32,
    height: f32,
}

impl Default for FrameContext {
    fn default() -> Self {
        Self {width: 800., height: 600.}
    }
}

impl FrameContext {
    pub fn with<R>(f: impl FnOnce(&FrameContext) -> R) -> R {
        FRAME.with(|ctx| f(ctx.borrow().as_ref().unwrap()))
    }
}

thread_local! {
    static FRAME: RefCell<Option<FrameContext>> = RefCell::new(Some(FrameContext::default()));
}

fn set_frame_context(width: f32, height: f32) {
    FRAME.with(|ctx| *ctx.borrow_mut() = Some(FrameContext { width, height }));
}


trait UiScale {
    fn x(self) -> f32;
    fn y(self) -> f32;
    fn xy(self) -> f32;
}

impl UiScale for f32 {
    fn x(self) -> f32 {
        FrameContext::with(|ctx| self * ctx.width / 2560.0)
    }
    fn y(self) -> f32 {
        FrameContext::with(|ctx| self * ctx.height / 1440.0)
    }
    fn xy(self) -> f32 {
        FrameContext::with(|ctx| self * 2. * ctx.width / 2560.0 * ctx.height / 1440.0)
    }}

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
pub enum Message {
    Tick,
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
    messages: std::sync::Mutex<Option<Vec<Message>>>,
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
    resources: GameResources,
    exit: bool,
}

pub type View<'a> = Element<'a, Message, Theme>;

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

pub fn get_ui_theme() -> Theme {
    let dark_palette = Palette {
        // background: Color::from_rgb8(0x18, 0x1B, 0x1F), // deep slate
        background: Color::from_rgb8(0x00, 0x00, 0x00), // deep slate
        text:       Color::WHITE,                      // high contrast
        primary:    Color::from_rgb8(0x2A, 0x2F, 0x36), // soft gunmetal
        success:    Color::from_rgb8(0xA0, 0xFF, 0xB0), // minty green
        danger:     Color::from_rgb8(0xFF, 0x5C, 0x5C), // coral red
    };
    let dark_theme = Theme::custom("Nightshade".to_string(), dark_palette);
    dark_theme
}

impl Ui {
    pub fn new() -> Self {
        font::load(vec![FONT.into()]);

        let transitions = generate_transitions();
        let start_state = &State::Main;
        let final_states = HashSet::new();
        let mut pda = PushdownAutomaton::new(start_state, final_states, transitions);

        let resources = crate::load_resources();
        let messages = std::sync::Mutex::new(None);

        Self {
            messages: messages,
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
            resources: resources,
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
    fn _update(&mut self, message: Message) -> () {
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
                // self.endpoint.close();
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
                let scaling = 1.0.x();
                let world = crate::World::from_json(&self.scenario);
                self.thumb = None;
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
                match self.mapgen_map.take() {
                    Some(world) => {world.to_json("assets/maps/mapgen.json")},
                    None => {},
                }
            }
            Message::GenerateMap => {
                let mut world: crate::World = crate::World::new();

                let mut locality_names: Vec<&str> = self.resources.locality_names.iter().map(|s| s.as_str()).collect();

                world.generate_from_template(&self.mapgen_ui, &mut locality_names, &self.resources.init_layout);

                self.thumb = None;
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
            let img: iced::widget::Image<Handle> = image(state.thumb.as_ref().unwrap()).into();
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

fn build_ui_for_state(state: &Ui) -> Element<Message, Theme> {
    let scaling = 1.0.x();
    match state.menu.get_state() {
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
            column!().into()
            // handled elsewhere

            // if matches!(state.endpoint_type, EndpointType::Server) {
            //     state.endpoint.send
            // }
        }
        _ => {panic!("menu panic");}
    }
}

impl IntoAny for Ui {
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

impl<B> Component<B> for Ui where B: Backend + 'static {
    type Message = Message;
    fn draw(&self, layout: &Layout<f32>, backend: &mut B, resources: &GameResources, time: f32) {
        if self.thumb.is_none() && self.mapgen_map.is_some() {
            let (width, height) = (900.0.x(), 900.0.y());
            let thumb_bytes = pollster::block_on(async {
                return backend.get_map_thumbnail(&self.mapgen_map.as_ref().unwrap(), width, height, resources).await;
            });
            backend.set_buffer(thumb_bytes);
        }

        let mut view = build_ui_for_state(&self);

        let dark_rounded_style = move |_theme: &Theme| crate::ui::container::Style {
            // solid black background
            background: Some(iced::Background::Color(Color::BLACK)),
            // white text so labels are readable
            text_color:  Some(Color::WHITE),
            ..crate::ui::container::Style::default()
        };

        view = Container::new(view)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(dark_rounded_style)
            .into();

        let mut messages = {
            let mut guard = self.messages.lock().unwrap();
            // Take the Vec out of the Option, or use an empty Vec if None
            std::mem::take(guard.as_mut().unwrap_or(&mut vec![]))
        };

        // Pass the Vec to backend
        backend.render_ui(&mut messages, view);

        // Put it back into the Mutex<Option<Vec<T>>>
        {
            let mut guard = self.messages.lock().unwrap();
            *guard = Some(messages);
        }

    }
    fn poll(&mut self, layout: &mut Layout<f32>, backend: &B) -> Self::Message {
        match backend.take_buffer() {
            Some(pixels) => self.thumb = Some(Handle::from_rgba(900.0.x() as u32, 900.0.y() as u32, pixels)),
            None => {}
        }

        match backend.poll_inputs(layout) {
            network::Message::Exit => Message::Transition(Input::Back),
            _ => Message::Tick,
        }

    }
    fn update(mut self: Box<Self>, message: Self::Message) -> Box<dyn ErasedComponent<B>> {
        self.endpoint.tick_without_component();

        self.messages.lock().unwrap().as_mut().map(|v| v.push(message));

        let drained_messages = {
            let mut guard = self.messages.lock().unwrap();
            std::mem::take(guard.as_mut().unwrap_or(&mut vec![]))
        };

        for message in drained_messages {
            self._update(message);
        }

        let state = self.menu.get_state();
        if matches!(state, State::Game) {
            return App::from_ui(*self)
        }
        
        self
    }
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


impl<M: Mode + 'static, B: Backend + 'static> App<M, B, Game>
where 
    M: crate::network::NotOffline, 
    Game: Component<B, Message = network::Message>, 
    <M as Mode>::Endpoint: 'static,
    App<M, B, Game>: Component<B, Message = network::Message>,
{
    pub fn from_ui(mut ui: Ui) -> Box<dyn ErasedComponent<B>> {
        let replacement: Box<dyn Chat> = Box::new(Offline);
        // let endpoint = Client::new("").unwrap().into();
        let endpoint_ = std::mem::replace(&mut ui.endpoint, replacement);
        let endpoint = *endpoint_.into_any().downcast::<M::Endpoint>().unwrap();

        let players_map = std::mem::take(&mut ui.players);
        let players: Vec<Player> = players_map.into_values().collect();
        let mut world = std::mem::take(&mut ui.mapgen_map).unwrap();
        let rules = Ruleset::from(ui);
        let mut component = Game::new(players, world, rules);
        // component.init_world();

        let app = Box::new( Self::new(component, endpoint) );
        
        println!("init complete!");
        app
    }
}

impl<B: Backend + 'static> App<Offline, B, Game>
where 
    Game: Component<B, Message = network::Message>,
{
    pub fn from_ui(mut ui: Ui) -> Box<dyn ErasedComponent<B>> {
        let players_map = std::mem::take(&mut ui.players);
        let players: Vec<Player> = players_map.into_values().collect();
        //let rules = Ruleset::default(ui.victory_condition, &players); // or empty vector?
        let mut world = std::mem::take(&mut ui.mapgen_map).unwrap();
        let rules = Ruleset::from(ui);
        // let mut world = ui.mapgen_map.unwrap();
        let mut component = Game::new(players, world, rules);

        let app= Box::new(component);

        println!("init complete!");
        app
    }
}

