// River tiles exist along the edges of hex tiles.
// They are represented by Cube<f32> and stored as a hashable CubeSide.
// For example, to represent a river tile in between (1, 0, 1) and (1, -1, 0)
// the position (1, -0.5, 0.5) is chosen based on {(1, 0, 1) + (1, -1, 0)} / 2
// this is stored as CubeSide(int: (1, 0, 0), half: (false, true, true), sign: (true, false, true))

use rand::seq::index::sample;

use crate::cubic;
use cubic::{Cube, DIRECTIONS, REV_DIRECTIONS};

use std::collections::HashSet;
use std::ops::Add;
use std::ops::Deref;
use std::ops::Index;

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::Mul;
use std::ops::Sub;

// Cube(1, -0.5, 0.5) becomes CubeSide(Cube(1, -0, 0), Cube(false, true, true))
// q = side.int.q + (side.half.q).copysign(side.int.q)
// CubeSide = (Cube, avg(Cube, (Cube + dir)).floatpart.asbool)
// HOWEVER, signed 0 does not exist, which introduces the need for the 3rd component sign
// so now
// Cube(1, -0.5, 0.5) becomes CubeSide(Cube(1, 0, 0), Cube(false, true, true), Cube(true, false, true))
// q = side.int.q + ( side.half.q * (if side.sign.q { 1.0 } else { -1.0 }) )
// OR one could simply always add/always sub the half component, removing the need for the sign component.
// todo: replace half with DIRECTION, make it an enum?
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CubeSide {pub int: Cube<i32>, pub half: Cube<bool>, pub sign: Cube<bool>}


impl CubeSide {
    fn is_first_decimal_half<T: Into<f32> + Copy + Display>(value: T) -> bool {
        let first_decimal = (value.into() * 10.0).abs() as u32 % 10;
        let half = match first_decimal {
            5 => true,
            0 => false,
            _ => panic!("Invalid CubeSide: {:}, first_decimal: {:}", value, first_decimal),
        };
        half
    }

    fn new<T: Into<f32> + Copy + Display, U: Into<f32> + Copy + Display>(q: T, r: U) -> Self {
        let q_int = q.into() as i32;
        let r_int = r.into() as i32;
        let q_half = CubeSide::is_first_decimal_half(q);
        let r_half = CubeSide::is_first_decimal_half(r);
        let q_sign = q.into().is_sign_positive();
        let r_sign = r.into().is_sign_positive();
        // println!("q:{:}, qint:{:}, qhalf:{:}, qsign:{:}, r:{:}, rint:{:}, rhalf:{:}, rsign:{:}", q, q_int, q_half, q_sign, r, r_int, r_half, r_sign);
        let s = Self {int: Cube::new(q_int, r_int),
              half: Cube::new(q_half, r_half),
              sign: Cube::new(q_sign, r_sign)};
        println!("{:}", s);
        s
    }
    pub fn int(self) -> Cube<i32> {
        let q = self.int.q().abs() * if self.sign.q() {1} else {-1};
        let r = self.int.r().abs() * if self.sign.r() {1} else {-1};
        Cube::new(q, r)
    }
}

impl From<&CubeSide> for Cube<f32> {
    fn from(cube: &CubeSide) -> Self {
        let q = cube.int.q() as f32 + (if cube.half.q() {0.5} else {0.}) * (if cube.sign.q() {1.} else {-1.});
        let r = cube.int.r() as f32 + (if cube.half.r() {0.5} else {0.}) * (if cube.sign.r() {1.} else {-1.});
        Self::new(q, r)
    }
}

impl From<CubeSide> for Cube<f32> {
    fn from(cube: CubeSide) -> Self {
        let q = cube.int.q() as f32 + (if cube.half.q() {0.5} else {0.}) * (if cube.sign.q() {1.} else {-1.});
        let r = cube.int.r() as f32 + (if cube.half.r() {0.5} else {0.}) * (if cube.sign.r() {1.} else {-1.});
        Self::new(q, r)
    }
}

impl From<Cube<f32>> for CubeSide {
    fn from(cube: Cube<f32>) -> Self {
        Self::new(cube.q(), cube.r())
    }
}

impl Display for CubeSide {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let cube_float = Cube::<f32>::from(self);
        // let q = self.int.q() as f32 + (self.half.q() as i32 as f32 / 2.);
        // let r = self.int.r() as f32 + (self.half.r() as i32 as f32 / 2.);
        write!(f, "({}, {})", cube_float.q(), cube_float.r())
    }
}

impl Debug for CubeSide {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let cube_float = Cube::<f32>::from(self);
        // let q = self.int.q() as f32 + ((self.half.q() as i32) as f32 / 2.);
        // let r = self.int.r() as f32 + ((self.half.r() as i32) as f32 / 2.);
        // let s = -q -r;
        f.debug_tuple("Cube")
         .field(&cube_float.q())
         .field(&cube_float.r())
         .field(&cube_float.s())
         .finish()
    }
}

// impl Add<Cube<i32>> for CubeSide {
//     type Output = Self;
//     fn add(mut self, rhs: Cube<i32>) -> Self::Output {
//         let sign_before = self.int();
//         self.int = self.int() + rhs;
//         let sign_after = self.int();
//         self
//     }
// }

// impl Sub<Cube<i32>> for CubeSide {
//     type Output = Self;
//     fn sub(mut self, rhs: Cube<i32>) -> Self::Output {
//         self.int = self.int() - rhs;
//         self
//     }
// }

impl Mul<i32> for CubeSide {
    type Output = Self;
    fn mul(mut self, rhs: i32) -> Self::Output {
        println!("mul<i32> for cubeside, i32={:}", rhs);
        println!("bf self: {:}, self.int: {:}", self, self.int);
        self.int = self.int() * rhs;
        println!("af self: {:}, self.int: {:}", self, self.int);
        let div = (rhs / 2, rhs % 2);
        if self.half.q() {
            self.int += Cube::new((div.0 != 0) as i32, 0);
        }
        if self.half.r() {
            self.int += Cube::new(0, (div.0 != 0) as i32);
        }
        println!("af if self: {:}, self.int: {:}", self, self.int);
        // self.int += Cube::new(div.0, div.0);
        self.half = Cube::new((div.1 != 0) & self.half.q(), (div.1 != 0) & self.half.r());
        self.sign = if div.1 == 0 {self.sign} else {Cube::new(!self.sign.q(), !self.sign.r())};
        self
    }
}

// generate a continuous length of hexagon segments
//
// pick a random tile as origin
// keep adding segments of the origin hex in clockwise manner until a random roll
// advances the origin to a new hex, in the direction of the current segment.
// repeat last step, reversing the clockwise direction
pub fn generate_river(land_tiles: HashSet<&Cube<i32>>, max_length: usize, threshold: f32) -> HashSet<CubeSide> {
    let mut river = HashSet::new();
    // pick a random starting position
    let starting_pos_idx = sample(&mut rand::thread_rng(), land_tiles.len(), 1).index(0);
    let mut origin: Cube<i32> = *land_tiles.iter().skip(starting_pos_idx).next().unwrap().clone();
    let mut reverse = true;
    let mut d = REV_DIRECTIONS;
    let mut dirs = DIRECTIONS.iter().cycle().skip(6);
    let mut next_direction = dirs.next().unwrap().clone();
    let mut current_dir = next_direction;
    // add half of the direction
    let mut next_pos = CubeSide::from(Cube::<f32>::from(origin) + (next_direction / 2));
    for _ in 0..max_length {
        river.insert(next_pos);
        let advance_a_tile = rand::random::<f32>();
        if advance_a_tile > threshold {
            // advance to a new tile in the direction of the current segment
            origin += current_dir;

            // if !land_tiles.contains(&origin) {break}

            // at this new origin, the direction represented by the current segment is mirrored
            current_dir = current_dir * -1;

            // reverse the iterator on every tile advancement
            // find the current direction within the new iterator, and skip to it
            d = if reverse {REV_DIRECTIONS} else {DIRECTIONS};
            let idx = d.iter().position(|&c| c == (current_dir)).unwrap();
            dirs = d.iter().cycle().skip(idx + 1);
            reverse = !reverse;
        }
        next_direction = dirs.next().unwrap().clone();
        next_pos = CubeSide::from(Cube::<f32>::from(origin) + (next_direction / 2));
        current_dir = next_direction;
    }
    river
}
