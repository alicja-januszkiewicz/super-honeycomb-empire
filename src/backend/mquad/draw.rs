use std::collections::HashSet;
use std::f32::consts::PI;

// use mquad::*;
use macroquad::prelude::*;
use macroquad::texture::load_image;

// use super::input::{poll_inputs, poll_map_editor_inputs};

use crate::backend::Backend;
use crate::cubic;
use crate::fog::Visibility;
use crate::game::GameResources;
use crate::World;
use crate::cubic::Cube;
use crate::cubic::DIRECTIONS;
use crate::cubic::Layout;
use crate::cubic::OrientationKind;
use crate::cubic::Pixel;
use crate::cubic::pixel_to_cube;
use crate::game::Game;
// use crate::inputs::{draw_tile_selector, draw_all_locality_names}; // Removed - these functions no longer exist
use crate::map_editor::Editor;
use crate::world::LocalityCategory;
use crate::world::Tile;
use crate::world::TileCategory;

fn clone_subset_with_keys<K, V>(original: &std::collections::HashMap<K, V>, key_subset: &std::collections::HashSet<K>) -> std::collections::HashMap<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    key_subset
        .iter()
        .filter_map(|k| original.get(k).map(|v| (k.clone(), v.clone())))
        .collect()
}

fn get_subset_with_references<'a, K, V>(
    original: &'a std::collections::HashMap<K, V>,
    key_subset: &HashSet<&K>,
) -> std::collections::HashMap<&'a K, &'a V>
where
    K: Eq + std::hash::Hash,
{
    key_subset
        .iter()
        .filter_map(|&k| original.get_key_value(k))
        .collect()
}

pub fn owner_to_color(&owner: &Option<usize>) -> macroquad::color::Color {
    match owner {
        Some(0) => Color { r: 1.0, g: 0., b: 0., a: 0.67 },
        Some(1) => Color { r: 0.0, g: 0.47, b: 0.95, a: 0.67 },
        Some(2) => Color { r: 0.0, g: 0.89, b: 0.19, a: 0.67 },
        Some(3) => Color { r: 0.53, g: 0.24, b: 0.75, a: 0.67 },
        Some(4) => Color { r: 0.0, g: 0.68, b: 0.67, a: 0.67 },  // tealasia
        Some(5) => Color { r: 0.86, g: 0.07, b: 0.23, a: 0.67 }, // crimsonia
        Some(6) => Color { r: 0.0, g: 0.5, b: 1.0, a: 0.67 }, //azurea
        Some(7) => Color { r: 0.85, g: 0.58, b: 0.87, a: 0.67 }, // lavendara
        Some(8) => Color { r: 1.0, g: 0.75, b: 0.0, a: 0.67 }, // amberon
        Some(9) => Color { r: 0.0, g: 0.35, b: 0.71, a: 0.67 }, //cobaltia
        _ => WHITE,
    }
}

pub struct Assets {
    // pub locality_names: Vec<String>,
    pub font: Font,
    pub army: Texture2D,
    pub port: Texture2D,
    pub airport: Texture2D,
    pub fields: Texture2D,
    pub water_material: Material,
    // pub init_layout: Layout<f32>,
    // pub shape: Vec<(f32, f32)>,
    // pub river: Vec<(usize, f32, f32)>,
}



// impl FontHandle for macroquad::text::Font {}
// impl TextureHandle for macroquad::texture::Texture2D {}
// impl MaterialHandle for macroquad::material::Material {}

//pub fn draw_base_tiles(world: &std::collections::HashMap<&Cube<i32>, &Tile>, layout: &Layout<f32>, assets: &Assets, time: f32) {
pub fn draw_base_tiles(world: &World, layout: &Layout<f32>, assets: &Assets, time: f32) {
    // let lens_center = get_frame_time();
    assets.water_material.set_uniform("Time", time);
    let size = layout.size[0] as f32;
    for (cube, tile) in world.iter() {
        let pixel = Cube::<f32>::from(*cube).to_pixel(&layout);
        // let color = match tile.category {
        //     TileCategory::Farmland => LIGHTGRAY,
        //     TileCategory::Water => SKYBLUE,
        // };
        let x = pixel.0;
        let y = pixel.1;
        let vertical = match layout.orientation {
            OrientationKind::Pointy(_) => true,
            OrientationKind::Flat(_) => false,
        };
        match tile.category {
            TileCategory::Farmland => {
                // set_texture("texture", &assets.fields);
                // gl_use_material(assets.water_material);
                draw_hexagon(x, y, size, layout.size[0]/20., vertical, BLACK, LIGHTGRAY);
                // gl_use_default_material();
            },
            TileCategory::Water => {
                gl_use_material(&assets.water_material);
                draw_hexagon(x, y, size, 0., vertical, BLACK, SKYBLUE);
                gl_use_default_material();
            }
        }
    }
}

pub fn draw_game_tiles(world: &std::collections::HashMap<&Cube<i32>, &Tile>, layout: &Layout<f32>, assets: &Assets) {
    let size = layout.size[0] as f32;
    let mut army_params = DrawTextureParams::default();
    army_params.dest_size = Some(Vec2{x: layout.size[0] as f32*1.5, y: layout.size[1] as f32*1.5});
    let mut airport_params = DrawTextureParams::default();
    airport_params.dest_size = Some(Vec2{x: layout.size[0] as f32, y: layout.size[1] as f32});
    let mut port_params = DrawTextureParams::default();
    port_params.dest_size = Some(Vec2{x: layout.size[0] as f32 * 0.9, y: layout.size[1] as f32 * 0.9});

    let airport_offset = layout.size[0] * 0.5;
    let port_offset = layout.size[0] * 0.5 * 0.9;
    let x_army_offset = layout.size[0] as f32 * 0.7;
    let y_army_offset = layout.size[1] as f32 * 0.7;
    for (cube, tile) in world.iter() {
        let pixel = Cube::<f32>::from(**cube).to_pixel(&layout);
        let x = pixel.0;
        let y = pixel.1;
        if tile.owner_index.is_some() {
            let color = owner_to_color(&tile.owner_index);
            let vertical = match layout.orientation {
                OrientationKind::Pointy(_) => true,
                OrientationKind::Flat(_) => false,
            };
            draw_hexagon(x, y, size, 0., vertical, BLACK, color);
            // match tile.category {
            //     TileCategory::Farmland => draw_hexagon(x, y, size, layout.size[0]/10., true, BLACK, color),
            //     TileCategory::Water => draw_hexagon(x, y, size, layout.size[0]/10., true, BLACK, SKYBLUE)
            // }
        }

        if tile.locality.is_some() {
            match tile.locality.as_ref().unwrap().category {
                LocalityCategory::Capital(i) => {
                    if i == tile.owner_index.unwrap() {
                        draw_circle(x, y, size/2., RED)
                    } else {
                        draw_circle(x, y, size/2., PINK)
                    }
                }
                // LocalityCategory::Capital => draw_circle(x, y, size/2., RED),
                // LocalityCategory::SatelliteCapital => draw_circle(x, y, size/2., PINK),
                LocalityCategory::City => draw_circle(x, y, size/2., DARKBROWN),
                LocalityCategory::PortCity => {
                    draw_circle(x, y, size/2., BLUE);
                    draw_texture_ex(&assets.port, x - port_offset, y - port_offset, WHITE, port_params.clone());
                },
                LocalityCategory::Airport => {
                    draw_rectangle(x - size/2., y - size/2., size, size, DARKGREEN);
                    draw_texture_ex(&assets.airport, x - airport_offset, y - airport_offset, WHITE, airport_params.clone());
                }
            }
        }
        if tile.army.is_some() {
            let color = owner_to_color(&tile.army.as_ref().unwrap().owner_index);
            // draw_texture(assets.army, x - x_army_offset, y - y_army_offset, color);
            draw_texture_ex(&assets.army, x - x_army_offset, y - y_army_offset, color, army_params.clone());
        }
        // if let Some(tile.locality) = locality {
        //     draw_circle(x, y, size, DARKBROWN)
        // }
        
    }
}

impl World {
    pub fn draw_shape_outline(mut shape: Vec<(f32, f32)>, layout: &crate::cubic::Layout<f32>, init_layout: &crate::cubic::Layout<f32>) {
        shape.push(shape[0]);
        for j in 1..shape.len() {
            let i = j - 1;
            let (mut x1, mut y1) = shape[i];
            let (mut x2, mut y2) = shape[j];
            x1 *= layout.size[0] / init_layout.size[0];
            x2 *= layout.size[0] / init_layout.size[0];
            y1 *= layout.size[1] / init_layout.size[1];
            y2 *= layout.size[1] / init_layout.size[1];
            x1 += layout.origin[0];
            x2 += layout.origin[0];
            y1 += layout.origin[1];
            y2 += layout.origin[1];
            macroquad::shapes::draw_line(x1, y1, x2, y2, 3., macroquad::color::BLACK);
        }
    }
    // pub fn draw_base_tiles(&self, &layout: &Layout<f32>, assets: &Assets, time: f32) {
    //     // let lens_center = get_frame_time();
    //     assets.water_material.set_uniform("Time", time);
    //     let size = layout.size[0] as f32;
    //     for (cube, tile) in self.world.iter() {
    //         let pixel = Cube::<f32>::from(*cube).to_pixel(&layout);
    //         // let color = match tile.category {
    //         //     TileCategory::Farmland => LIGHTGRAY,
    //         //     TileCategory::Water => SKYBLUE,
    //         // };
    //         let x = pixel.0;
    //         let y = pixel.1;
    //         let vertical = match layout.orientation {
    //             OrientationKind::Pointy(_) => true,
    //             OrientationKind::Flat(_) => false,
    //         };
    //         match tile.category {
    //             TileCategory::Farmland => {
    //                 // set_texture("texture", &assets.fields);
    //                 // gl_use_material(assets.water_material);
    //                 draw_hexagon(x, y, size, layout.size[0]/20., vertical, BLACK, LIGHTGRAY);
    //                 // gl_use_default_material();
    //             },
    //             TileCategory::Water => {
    //                 gl_use_material(&assets.water_material);
    //                 draw_hexagon(x, y, size, 0., vertical, BLACK, SKYBLUE);
    //                 gl_use_default_material();
    //             }
    //         }
    //     }
    // }
    pub fn draw_game_tiles(&self, &layout: &Layout<f32>, assets: &Assets) {
        let size = layout.size[0] as f32;
        let mut army_params = DrawTextureParams::default();
        army_params.dest_size = Some(Vec2{x: layout.size[0] as f32*1.5, y: layout.size[1] as f32*1.5});
        let mut airport_params = DrawTextureParams::default();
        airport_params.dest_size = Some(Vec2{x: layout.size[0] as f32, y: layout.size[1] as f32});
        let mut port_params = DrawTextureParams::default();
        port_params.dest_size = Some(Vec2{x: layout.size[0] as f32 * 0.9, y: layout.size[1] as f32 * 0.9});

        let airport_offset = layout.size[0] * 0.5;
        let port_offset = layout.size[0] * 0.5 * 0.9;
        let x_army_offset = layout.size[0] as f32 * 0.7;
        let y_army_offset = layout.size[1] as f32 * 0.7;
        for (cube, tile) in self.world.iter() {
            let pixel = Cube::<f32>::from(*cube).to_pixel(&layout);
            let x = pixel.0;
            let y = pixel.1;
            if tile.owner_index.is_some() {
                let color = owner_to_color(&tile.owner_index);
                let vertical = match layout.orientation {
                    OrientationKind::Pointy(_) => true,
                    OrientationKind::Flat(_) => false,
                };
                draw_hexagon(x, y, size, 0., vertical, BLACK, color);
                // match tile.category {
                //     TileCategory::Farmland => draw_hexagon(x, y, size, layout.size[0]/10., true, BLACK, color),
                //     TileCategory::Water => draw_hexagon(x, y, size, layout.size[0]/10., true, BLACK, SKYBLUE)
                // }
            }

            if tile.locality.is_some() {
                match tile.locality.as_ref().unwrap().category {
                    LocalityCategory::Capital(i) => {
                        if i == tile.owner_index.unwrap() {
                            draw_circle(x, y, size/2., RED)
                        } else {
                            draw_circle(x, y, size/2., PINK)
                        }
                    }
                    //LocalityCategory::SatelliteCapital => draw_circle(x, y, size/2., PINK),
                    LocalityCategory::City => draw_circle(x, y, size/2., DARKBROWN),
                    LocalityCategory::PortCity => {
                        draw_circle(x, y, size/2., BLUE);
                        draw_texture_ex(&assets.port, x - port_offset, y - port_offset, WHITE, port_params.clone());
                    },
                    LocalityCategory::Airport => {
                        draw_rectangle(x - size/2., y - size/2., size, size, DARKGREEN);
                        draw_texture_ex(&assets.airport, x - airport_offset, y - airport_offset, WHITE, airport_params.clone());
                    }
                }
            }
            if tile.army.is_some() {
                let color = owner_to_color(&tile.army.as_ref().unwrap().owner_index);
                // draw_texture(assets.army, x - x_army_offset, y - y_army_offset, color);
                draw_texture_ex(&assets.army, x - x_army_offset, y - y_army_offset, color, army_params.clone());
            }
            // if let Some(tile.locality) = locality {
            //     draw_circle(x, y, size, DARKBROWN)
            // }
            
        }
    }
}


// def game_tile_primitive(context, layout, tilepair):
//     cube, tile = tilepair
//     color = set_color(tile)
//     hexagon_rgba(context, layout, cube, color)


fn draw_map_control_summary(game: &Game) {
    let width = macroquad::window::screen_width();
    let ratio = 0.83; // 1700 / 2048
    let mut dy = 0.;
    for (idx, player) in game.players.iter().enumerate() {
        let color = owner_to_color(&Some(idx));
        let no_owned = game.world.cubes_by_ownership.get(&idx).unwrap().len();
        let percentage = no_owned as f32 / game.world.len() as f32 * 100.;
        let text = format!("{}: {:.2}%", player.name, percentage);
        let (x, mut y) = (ratio * width, 50.);
        y += dy;
        dy += 40.;
        draw_text(&text, x, y, 40., color);
    }
}


// pub fn draw(game: &Game, &layout: &Layout<f32>, assets: &Assets, resources: &GameResources, time: f32) {
//     macroquad::prelude::clear_background(macroquad::prelude::DARKGRAY);
//     // let mut player_index = game.current_player_index();
//     // // do not reveal ai player pov
//     // if game.current_player().ai.is_some() {
//     //     player_index = 0;
//     // }
//     // // let world_keys = game.world.keys().cloned().collect();
//     // let world_keys: HashSet<_> = game.world.keys().collect();
//     // let world = match game.player_visibilities.get(&player_index) {
//     //     Some(mask) => {
//     //         // let set: std::collections::HashSet<Cube<i32>> = mask.keys().cloned().collect();
//     //         let set: HashSet<_> = mask.iter().filter_map(|(k, v)| match v {
//     //             Visibility::Clear => Some(k),
//     //             Visibility::Shroud => Some(k),
//     //             Visibility::Fog => None,
//     //         }).collect();
//     //         let key_subset: HashSet<_> = set.intersection(&world_keys).copied().collect();
//     //         get_subset_with_references(&game.world, &key_subset)
//     //     },
//     //     None => get_subset_with_references(&game.world, &world_keys)
//     // };

//     // draw_base_tiles(&world, &layout, assets, time);

//     // let world = match game.player_visibilities.get(&player_index) {
//     //     Some(mask) => {
//     //         // let set: std::collections::HashSet<Cube<i32>> = mask.keys().cloned().collect();
//     //         let set: HashSet<_> = mask.iter().filter_map(|(k, v)| match v {
//     //             Visibility::Clear => Some(k),
//     //             Visibility::Shroud => None,
//     //             Visibility::Fog => None,
//     //         }).collect();
//     //         let key_subset: HashSet<_> = set.intersection(&world_keys).copied().collect();
//     //         get_subset_with_references(&game.world, &key_subset)
//     //     },
//     //     None => get_subset_with_references(&game.world, &world_keys)
//     // };

//     // draw_game_tiles(&world, &layout, assets);

//     let view = game.current_player_index()
//         .and_then(|pid| game.player_views.get(&pid))
//         .unwrap_or(&game.world);

//     view.draw_base_tiles(&layout, &assets, time);
//     view.draw_game_tiles(&layout, &assets);

//     // draw_tile_selector(&layout); // TODO: Implement in wgpu renderer

//     let has_selection = game.current_player().is_some_and(|p| p.selection.is_some());
//     if has_selection {
//         draw_army_legal_moves(&game, &layout);
//     } else {
//         draw_army_can_move_indicator(&game, &layout);
//     }

//     draw_army_info(&game.world, &layout);
//     // draw_all_locality_names(&game.world, &layout, &assets); // TODO: Implement in wgpu renderer

//     draw_text(&get_fps().to_string(), 50.0, 50.0, 40., BLACK);
//     draw_map_control_summary(game);

//     for cs in &game.world.rivers {
//         draw_river(&cs, &layout);
//     }

//     // let mut shape = assets.shape.clone();
//     let mut shape = resources.river.clone();

//     // let x_min = shape.iter().fold(f32::NAN, |a, &b| a.min(b.1));
//     // let y_min = shape.iter().fold(f32::NAN, |a, &b| a.min(b.2));
//     // shape = shape.iter().map(|(id, x, y)| (*id, x - x_min, y - y_min)).collect();
//     // let ids: std::collections::HashSet<usize> = shape.iter().map(|(id, x, y)| *id).collect();
//     // let ids: Vec<usize> = ids.into_iter().collect();
//     // println!("ids len: {:}", ids.len());
//     // let COLORS = vec!(BEIGE, BLACK, BLUE, BROWN, DARKBLUE, DARKBROWN, DARKGREEN, DARKPURPLE, GOLD, GREEN, LIME, MAGENTA, MAROON, ORANGE, PINK, PURPLE, RED, SKYBLUE, VIOLET, WHITE, YELLOW,);
//     let COLORS = vec!(BEIGE, BLACK, BLUE, BROWN, GOLD, GREEN, LIME, MAGENTA, MAROON, ORANGE, PINK, PURPLE, RED, VIOLET, WHITE, YELLOW,);

//     for j in 1..shape.len() {
//         let (id, mut x, mut y) = shape[j];
//         x *= layout.size[0] / resources.init_layout.size[0];
//         y *= layout.size[1] / resources.init_layout.size[1];
//         x += layout.origin[0];
//         y += layout.origin[1];
//         // let id_idx = ids.iter().position(|id_| id == *id_).unwrap();
//         let color = COLORS[j % COLORS.len()];
//         // let color = RED;
//         draw_circle(x, y, 8., color);
//     }
//     // World::draw_shape_outline(shape, &layout, &assets.init_layout);
// }

// fn draw_editor_brush(editor: &Editor) {
//     match editor.brush {
//         BrushMode::Place => {
//             let mouse = mouse_position();
//             let pixel = Cube::<f32>::from(*cube).to_pixel(&layout);
//             // let color = match tile.category {
//             //     TileCategory::Farmland => LIGHTGRAY,
//             //     TileCategory::Water => SKYBLUE,
//             // };
//             let size = layout.size[0] as f32;
//             let x = pixel[0] as f32;
//             let y: f32 = pixel[1] as f32;
//             let vertical = match layout.orientation {
//                 OrientationKind::Pointy(_) => true,
//                 OrientationKind::Flat(_) => false,
//             };
//             match tile.category {
//                 TileCategory::Farmland => {
//                     // set_texture("texture", &assets.fields);
//                     // gl_use_material(assets.water_material);
//                     draw_hexagon(x, y, size, layout.size[0]/20., vertical, BLACK, LIGHTGRAY);
//                     // gl_use_default_material();
//                 },
//         }
//     }
// }

fn draw_army_can_move_indicator(game: &Game, &layout: &Layout<f32>) {
    let Some(current_player_index) = game.current_player_index() else {return};
    let size = layout.size[0];
    game.world.iter().for_each(|(cube, tile)|
        if tile.army.as_ref().is_some_and(|x| x.can_move & x.owner_index.is_some_and(|x| x == current_player_index)) 
        {
            let vertical = match layout.orientation {
                OrientationKind::Pointy(_) => true,
                OrientationKind::Flat(_) => false,
            };
            let p = Cube::<f32>::from(*cube).to_pixel(&layout);
            let color = Color::from_rgba(255, 255, 0, 136);//0x8800ffff
            draw_hexagon(p.0, p.1, size, size/10., vertical, BLACK, color);
        }
    )
}

fn draw_army_legal_moves(game: &Game, &layout: &Layout<f32>) {
    // let selection = game.current_player().selection;
    let Some(player) = game.current_player() else {return};
    let size = layout.size[0];
    if let Some(selection) = player.selection {
        let color = Color::from_rgba(255, 255, 0, 136);//0x8800ffff
        let vertical = match layout.orientation {
            OrientationKind::Pointy(_) => true,
            OrientationKind::Flat(_) => false,
        };

        game.world.get_all_legal_moves(&selection, &game.current_player_index().unwrap()).iter().for_each(|cube| {
            let p = Cube::<f32>::from(*cube).to_pixel(&layout);
            draw_hexagon(p.0, p.1, size, size/10., vertical, BLACK, color);
        });
    }
}

fn draw_army_info(world: &World, &layout: &Layout<f32>) {
    let pos = mouse_position();
    let cube = pixel_to_cube(&layout, pos.into()).round();
    let mut nearest_cubes = cube.disc(2);
    nearest_cubes.push(cube);
    nearest_cubes.iter().for_each(|cube|
        if world.get(cube).is_some_and(|tile| tile.army.is_some()) {
            let p = Cube::<f32>::from(*cube).to_pixel(&layout);
            army_info(p, &layout, &world.get(cube).unwrap());
        }
    )

}

pub fn army_info(p: Pixel<f32>, layout: &Layout<f32>, tile: &Tile) {
    army_info_backdrop(p, &layout);
    army_info_text(p, &layout, &tile);
}

fn army_info_text(p: Pixel<f32>, layout: &Layout<f32>, tile: &Tile) {
    let size = layout.size[0] * 0.8;
    let offset = layout.size[0] * 0.2;
    let x = p.0 - offset * 2.;
    let y = p.1 - offset / 2.;
    let army = tile.army.as_ref().unwrap();
    let manpower_text = army.manpower.to_string();
    draw_text(manpower_text.as_str(), x, y, size, WHITE);

    let offset = -6.;
    let x = p.0 + offset / 2.;
    let y = p.1 - offset * 2.;
    let morale_text = army.morale.to_string();
    draw_text(morale_text.as_str(), x, y, size, RED);
}

fn draw_semicircle(center: [f32; 2], radius: f32, start_angle: f32, end_angle: f32, sides: usize, color: Color) {
    let [x, y] = center;
    let angle_range = end_angle - start_angle;
    let angle_step = angle_range / sides as f32;
    let v1 = Vec2::new(x, y);
    let mut v2 = Vec2::new(x + radius * start_angle.cos(), y + radius * start_angle.sin());

    for n in 1..=sides {
        let angle = start_angle + n as f32 * angle_step;
        let v3 = Vec2::new(x + radius * angle.cos(), y + radius * angle.sin());
        draw_triangle(v1, v2, v3, color);
        v2 = v3;
    }
}

fn draw_two_circles(center: [f32; 2], radius: f32, angle: f32, sides: usize) {
    let offset_angle = - std::f32::consts::PI / 6.;
    draw_semicircle(center, radius, 0.0 + offset_angle, std::f32::consts::PI + offset_angle, sides, WHITE);
    draw_semicircle(center, radius, std::f32::consts::PI + offset_angle, 2.0 * std::f32::consts::PI + offset_angle, sides, BLACK);
}

fn army_info_backdrop(p: Pixel<f32>, layout: &Layout<f32>) {
    let r = layout.size[0] * (3f32.sqrt())/2.0 * 0.8; // Radius of the semicircle
    let angle = std::f32::consts::PI / 4.0; // Angle between the two circles (45 degrees)
    let [x, y] = [p.0, p.1];

    let sides = 10; // Number of sides (points) to use for the semicircle

    draw_two_circles([x, y], r, angle, sides);
}

pub fn draw_tile_side(cube: &crate::river::CubeSide, layout: &Layout<f32>, thickness: f32, color: Color) {
    let direction_q = ((cube.half.q() as i32 as f32).copysign(cube.int.q() as f32) as i32).abs();
    let direction_r = ((cube.half.r() as i32 as f32).copysign(cube.int.r() as f32) as i32).abs() * (-1);
    let direction = Cube::new(direction_q, direction_r);

    let origin = Cube::<f32>::from(cube) - (direction / 2);
    let mut corners = origin.corners(layout);

    let mut idx = (DIRECTIONS.iter().position(|dir| dir == &direction).unwrap()) % 6;
    let idx_2 = (idx + 1) % 6;
    let p1 = corners[idx];
    let p2 = corners[idx_2];

    draw_line(p1.0, p1.1, p2.0, p2.1, thickness, color);
}

fn draw_river(cube: &crate::river::CubeSide, layout: &Layout<f32>) {
    let thickness = layout.size[0] / 4.;
    let color = BLUE;
    draw_tile_side(cube, layout, thickness, color);
}

// pub async fn run_editor(world: &World, layout: &Layout<f32>, assets: &Assets, time: f32) {
// pub async fn run_editor(assets: &Assets) {

//     let mut editor = Editor::new(World::new(), vec!());

//     let size = [32.,32.];
//     // let size = [0.1,0.1]; // use this if in local coords
//     let origin = [300., 300.];//[100.,600.];//[100.,300.];
//     let mut layout = cubic::Layout{orientation: cubic::OrientationKind::Flat(cubic::FLAT), size, origin};

//     let mut time = 0.0;

//     loop {
//         clear_background(DARKGRAY);

//         poll_map_editor_inputs(&mut editor, &mut layout);

//         if is_key_pressed(KeyCode::F1) {
//             break
//         }

//         draw_editor(&editor, &layout, &assets, time);

//         next_frame().await;
//         time += get_frame_time();
//     }
// }


pub fn draw_tile_selector(&layout: &Layout<f32>) {
    let vertical = match layout.orientation {
        OrientationKind::Pointy(_) => true,
        OrientationKind::Flat(_) => false,
    };
    let size = layout.size[0];
    let pos = mouse_position().into();
    // let pos = mouse_position_local().into();
    let cube = cubic::pixel_to_cube(&layout, pos).round();
    let p = cube.to_pixel(&layout);
    let [x, y] = [p.0, p.1];
    // println!("{:?}", pos);
    let color = Color::from_rgba(224, 208, 64, 136); // 0x88d0e040
    draw_hexagon(x, y, size, size/10., vertical, BLACK, color);
}

fn draw_locality_name(layout: &Layout<f32>, cube: &Cube<i32>, tile: &Tile, font: Font) {
    let params = TextParams{
        font: Some(&font),
        font_size: (0.9 * layout.size[0]) as u16, // 6,
        //font_scale: (0.7 * layout.size[0]),
        ..Default::default()
    };
    if let Some(locality) = &tile.locality {
        // if let Some(category) = locality.category {}
        let color = match locality.category {
            LocalityCategory::Capital(_) => WHITE, // tile.locality.starting_owner.color,
            _ => WHITE,
        };

        let p = Cube::<f32>::from(*cube).to_pixel(&layout);
        let [mut x, mut y] = [p.0, p.1];
        let center: Vec2 = get_text_center(locality.name.as_str(), params.font, params.font_size, params.font_scale, params.rotation);
        x -= center.x;
        y -= layout.size[1] + center.y;
        draw_text_ex(locality.name.as_str(), x, y, params);
    }

}


// Draws names of all localities within 5 cubes of the cursor.
pub fn draw_all_locality_names(world: &World, layout: &Layout<f32>, assets: &Assets) {
    let pos = mouse_position().into();
    let cube = cubic::pixel_to_cube(&layout, pos).round();

    let cubes_within_draw_range = cube.disc(5);
    for cube in cubes_within_draw_range {
        match world.get(&cube) {
            Some(tile) if tile.locality.is_some() => draw_locality_name(layout, &cube, tile, assets.font.clone()),
            _ => {},
        }
    }
}
