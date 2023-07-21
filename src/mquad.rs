use std::f32::consts::PI;

use macroquad::prelude::*;
use macroquad::texture::load_image;

use crate::cubic;
use crate::World;
use crate::cubic::Cube;
use crate::cubic::Layout;
use crate::cubic::OrientationKind;
use crate::cubic::pixel_to_cube;
use crate::game::Game;
use crate::inputs::{draw_tile_selector, draw_all_locality_names};
use crate::map_editor::Editor;
use crate::world::LocalityCategory;
use crate::world::Tile;
use crate::world::TileCategory;

fn owner_to_color(&owner: &Option<usize>) -> macroquad::color::Color {
    match owner {
        Some(0) => Color { r: 0.9, g: 0.16, b: 0.22, a: 0.67 },
        Some(1) => Color { r: 0.0, g: 0.47, b: 0.95, a: 0.67 },
        Some(2) => Color { r: 0.0, g: 0.89, b: 0.19, a: 0.67 },
        Some(3) => Color { r: 0.53, g: 0.24, b: 0.75, a: 0.67 },
        _ => WHITE,
    }
}

pub struct Assets {
    pub locality_names: Vec<String>,
    pub font: Font,
    pub army: Texture2D,
    pub airport: Texture2D,
    pub fields: Texture2D,
    pub water_material: Material,
}

impl World {
    pub fn draw_base_tiles(&self, &layout: &Layout<f32>, assets: &Assets, time: f32) {
        // let lens_center = get_frame_time();
        assets.water_material.set_uniform("Time", time);
        let size = layout.size[0] as f32;
        for (cube, tile) in self.world.iter() {
            let pixel = Cube::<f32>::from(*cube).to_pixel(&layout);
            // let color = match tile.category {
            //     TileCategory::Farmland => LIGHTGRAY,
            //     TileCategory::Water => SKYBLUE,
            // };
            let x = pixel[0] as f32;
            let y: f32 = pixel[1] as f32;
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
                    gl_use_material(assets.water_material);
                    draw_hexagon(x, y, size, 0., vertical, BLACK, SKYBLUE);
                    gl_use_default_material();
                }
            }
        }
    }
    pub fn draw_game_tiles(&self, &layout: &Layout<f32>, assets: &Assets) {
        let size = layout.size[0] as f32;
        let mut army_params = DrawTextureParams::default();
        army_params.dest_size = Some(Vec2{x: layout.size[0] as f32*1.5, y: layout.size[1] as f32*1.5});
        let mut airport_params = DrawTextureParams::default();
        airport_params.dest_size = Some(Vec2{x: layout.size[0] as f32, y: layout.size[1] as f32});
        let airport_offset = layout.size[0] * 0.5;
        let x_army_offset = layout.size[0] as f32 * 0.7;
        let y_army_offset = layout.size[1] as f32 * 0.7;
        for (cube, tile) in self.world.iter() {
            let pixel = Cube::<f32>::from(*cube).to_pixel(&layout);
            let x = pixel[0] as f32;
            let y = pixel[1] as f32;
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
                    LocalityCategory::Capital => draw_circle(x, y, size/2., RED),
                    LocalityCategory::SatelliteCapital => draw_circle(x, y, size/2., PINK),
                    LocalityCategory::City => draw_circle(x, y, size/2., DARKBROWN),
                    LocalityCategory::PortCity => draw_circle(x, y, size/2., BLUE),
                    LocalityCategory::Airport => {
                        draw_rectangle(x - size/2., y - size/2., size, size, DARKGREEN);
                        draw_texture_ex(assets.airport, x - airport_offset, y - airport_offset, WHITE, airport_params.clone());
                    }
                }
            }
            if tile.army.is_some() {
                let color = owner_to_color(&tile.army.as_ref().unwrap().owner_index);
                // draw_texture(assets.army, x - x_army_offset, y - y_army_offset, color);
                draw_texture_ex(assets.army, x - x_army_offset, y - y_army_offset, color, army_params.clone());
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




pub fn draw(game: &Game, &layout: &Layout<f32>, assets: &Assets, time: f32) {
    let has_selection = game.current_player().selection.is_some();
    game.world.draw_base_tiles(&layout, &assets, time);
    game.world.draw_game_tiles(&layout, &assets);

    draw_tile_selector(&layout);

    if has_selection {
        draw_army_legal_moves(&game, &layout);
    } else {
        draw_army_can_move_indicator(&game, &layout);
    }

    draw_army_info(&game.world, &layout);
    draw_all_locality_names(&game.world, &layout, &assets);
}

pub fn draw_editor(editor: &Editor, layout: &Layout<f32>, assets: &Assets, time: f32) {
    editor.world.draw_base_tiles(&layout, &assets, time);
    editor.world.draw_game_tiles(&layout, &assets);

    draw_tile_selector(&layout);

    // draw_editor_brush(editor);

    draw_army_info(&editor.world, &layout);
    draw_all_locality_names(&editor.world, &layout, &assets);
}

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
    let current_player_index = game.current_player_index();
    let size = layout.size[0];
    game.world.iter().for_each(|(cube, tile)|
        if tile.army.as_ref().is_some_and(|x| x.can_move & x.owner_index.is_some_and(|x| x == current_player_index)) 
        {
            let vertical = match layout.orientation {
                OrientationKind::Pointy(_) => true,
                OrientationKind::Flat(_) => false,
            };
            let [x, y] = Cube::<f32>::from(*cube).to_pixel(&layout);
            let color = Color::from_rgba(255, 255, 0, 136);//0x8800ffff
            draw_hexagon(x, y, size, size/10., vertical, BLACK, color);
        }
    )
}

fn draw_army_legal_moves(game: &Game, &layout: &Layout<f32>) {
    // let selection = game.current_player().selection;
    let size = layout.size[0];
    if let Some(selection) = game.current_player().selection {
        let color = Color::from_rgba(255, 255, 0, 136);//0x8800ffff
        let vertical = match layout.orientation {
            OrientationKind::Pointy(_) => true,
            OrientationKind::Flat(_) => false,
        };

        game.world.get_all_legal_moves(&selection, &game.current_player_index()).iter().for_each(|cube| {
            let [x, y] = Cube::<f32>::from(*cube).to_pixel(&layout);
            draw_hexagon(x, y, size, size/10., vertical, BLACK, color);
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
            let pos = Cube::<f32>::from(*cube).to_pixel(&layout);
            army_info(pos, &layout, &world.get(cube).unwrap());
        }
    )

}

fn army_info(pos: [f32; 2], layout: &Layout<f32>, tile: &Tile) {
    army_info_backdrop(pos, &layout);
    army_info_text(pos, &layout, &tile);
}

fn army_info_text(pos: [f32; 2], layout: &Layout<f32>, tile: &Tile) {
    let size = layout.size[0] * 0.8;
    let offset = layout.size[0] * 0.2;
    let x = pos[0] - offset * 2.;
    let y = pos[1] - offset / 2.;
    let army = tile.army.as_ref().unwrap();
    let manpower_text = army.manpower.to_string();
    draw_text(manpower_text.as_str(), x, y, size, WHITE);

    let offset = -6.;
    let x = pos[0] + offset / 2.;
    let y = pos[1] - offset * 2.;
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

fn army_info_backdrop(pos: [f32; 2], layout: &Layout<f32>) {
    let r = layout.size[0] * (3f32.sqrt())/2.0 * 0.8; // Radius of the semicircle
    let angle = std::f32::consts::PI / 4.0; // Angle between the two circles (45 degrees)
    let [x, y] = pos;

    let sides = 10; // Number of sides (points) to use for the semicircle

    draw_two_circles([x, y], r, angle, sides);
}