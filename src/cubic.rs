use num::Signed;

use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::Mul;
use std::ops::Div;

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

use std::f64::consts::PI;
use std::ops::SubAssign;

use serde::{Serialize, Deserialize};

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Cube<T>(T, T);

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Pixel<T>(pub T, pub T);

impl<T: Add<T, Output = T>> Add<Pixel<T>> for Pixel<T> {
    type Output = Self;
    fn add(self, rhs: Pixel<T>) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Div<f32> for Pixel<f32> {
    type Output = Pixel<f32>;
    fn div(self, rhs: f32) -> Self::Output {
        if rhs == 0. { panic!("Cannot divide by zero!"); }
        Pixel(self.0 / rhs, self.1 / rhs)
    }
}

impl<T> Cube<T> where T: Copy {
    pub fn new(q: T, r: T) -> Self {
        Self(q,r)
    }

    pub fn q(&self) -> T {
        self.0
    }

    pub fn r(&self) -> T {
        self.1
    }
}

impl<T> Cube<T> where T: Copy + Signed {
    pub fn s(&self) -> T {
        -self.0 - self.1
    }
}

impl<T: std::fmt::Display> Serialize for Cube<T> {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = format!("{}_{}", self.0, self.1);
        serializer.serialize_str(&s)
    }
}

impl<'de, T: std::str::FromStr + std::fmt::Debug> Deserialize<'de> for Cube<T>
where
    T::Err: std::fmt::Debug,
{
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut parts = s.split('_');
        let x = parts.next().ok_or_else(|| serde::de::Error::custom(""))?
            .parse().map_err(|_| serde::de::Error::custom(""))?;
        let y = parts.next().ok_or_else(|| serde::de::Error::custom(""))?
            .parse().map_err(|_| serde::de::Error::custom(""))?;
        
        if parts.next().is_some() {
            return Err(serde::de::Error::custom(""));
        }
        
        Ok(Cube(x, y))
    }
}

impl<T: std::fmt::Display> Display for Cube<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

impl<T: Signed + Copy + Debug> Debug for Cube<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_tuple("Cube")
         .field(&self.0)
         .field(&self.1)
         .field(&self.s())
         .finish()
    }
}

// Allow casting Cube(1,2) to Cube(1.0, 2.0)
impl From<Cube<i32>> for Cube<f64> {
    fn from(cube_in: Cube<i32>) -> Self {
        Cube(f64::from(cube_in.0), f64::from(cube_in.1))
    }
}

// Allow casting Cube(1,2) to Cube(1.0, 2.0)
impl From<Cube<i32>> for Cube<f32> {
    fn from(cube_in: Cube<i32>) -> Self {
        Cube(cube_in.0 as f32, cube_in.1 as f32)
    }
}

impl From<Cube<f32>> for Cube<i32> {
    fn from(cube_in: Cube<f32>) -> Self {
        Cube(cube_in.0 as i32, cube_in.1 as i32)
    }
}

// // dirty hack
// impl From<&Cube<i32>> for Cube<f64> {
//     fn from(cube_in: &Cube<i32>) -> Self {
//         Cube(cube_in.0 as f64, cube_in.1 as f64)
//     }
// }

// This doesn't work as I cannot constrain T != U and thus it overrides the default implementation
// impl<T, U: From<T>> From<Cube<U>> for Cube<T> {
//     fn from(arg: U) -> Self {
//         T::From(arg)
//     }
// }


impl<T, U> Add<Cube<U>> for Cube<T> where T: Add<U> {
    type Output = Cube<<T as Add<U>>::Output>;

    fn add(self, rhs: Cube<U>) -> Self::Output {
        Cube(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T: Copy, U> AddAssign<Cube<U>> for Cube<T> where T: Add<U, Output=T> {
    fn add_assign(&mut self, other: Cube<U>) {
        *self = Self (
            self.0 + other.0,
            self.1 + other.1,
        );
    }
}

impl<T, U> Sub<Cube<U>> for Cube<T> where T: Sub<U> {
    type Output = Cube<<T as Sub<U>>::Output>;

    fn sub(self, rhs: Cube<U>) -> Self::Output {
        Cube(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<T: Copy, U> SubAssign<Cube<U>> for Cube<T> where T: Sub<U, Output=T> {
    fn sub_assign(&mut self, other: Cube<U>) {
        *self = Self (
            self.0 - other.0,
            self.1 - other.1,
        );
    }
}

impl<T, U: Copy> Mul<U> for Cube<T> where T: Mul<U> {
    type Output = Cube<<T as Mul<U>>::Output>;

    fn mul(self, rhs: U) -> Self::Output {
        Cube(self.0 * rhs, self.1 * rhs)
    }
}

// trait ToPixel<T> {
//     fn to_pixel(&self, layout: &Layout<T>) -> [T; 2];
// }

// impl<T> ToPixel<T> for Cube<T> where T: Copy + Signed + Mul {
//     fn to_pixel(&self, &layout: &Layout<T>) -> [T; 2] {
//         let matrix = layout.orientation.inner();
//         let size = layout.size;
//         let origin = layout.origin;

//         let x = (matrix.f0 * self.q() + matrix.f1 * self.r()) * size[0];
//         let y = (matrix.f2 * self.q() + matrix.f3 * self.r()) * size[1];
//         [x + origin[0] as T, y + origin[1]]
//     }
// }

impl<T> Cube<T> where T: Copy + Signed + Mul + From<f32> + num::Float {
    pub fn to_pixel(&self, &layout: &Layout<T>) -> Pixel<T> {
        let matrix = layout.orientation.inner();
        let size = layout.size;
        let origin = layout.origin;

        // let hw = (screen_width() / 2.).into();
        // let hh = (screen_height() / 2.).into();

        let x = ((matrix.f0 * self.q() + matrix.f1 * self.r()) * size[0]);// / hw;
        let y = ((matrix.f2 * self.q() + matrix.f3 * self.r()) * size[1]);// / hh;
        Pixel(x + origin[0], y + origin[1])
    }
    fn corner_offset(&layout: &Layout<T>, corner: u8) -> Pixel<T> {
        let matrix = layout.orientation.inner();
        let size = layout.size;
        let angle: T = <T as num::NumCast>::from(2.0 * PI).unwrap() * (matrix.start_angle - (corner as f32).into()) / (6.0).into();
        Pixel(size[0] * angle.cos(), size[1] * angle.sin())
    }
    pub fn corners(&self, &layout: &Layout<T>) -> [Pixel<T>; 6] {
        let mut corners: [Pixel<T>; 6] = [Pixel(<T as num::NumCast>::from(0.0).unwrap(), <T as num::NumCast>::from(0.0).unwrap()); 6];
        let center = self.to_pixel(&layout);
        for i in 0..6 {
            let offset = Cube::corner_offset(&layout, i);
            corners[i as usize] = Pixel(center.0 + offset.0, center.1 + offset.1);
        }
        corners.reverse();
        let mut shift = 3;
        if matches!(layout.orientation, OrientationKind::Flat(_)) {shift = 4}
        corners.rotate_left(shift);
        corners
    }
}

impl Cube<f32> {
    pub fn round<T> (&self) -> Cube<T>
    where Cube<T>: From<Cube<f32>>
    {
        let mut qi = self.q().round();
        let mut ri = self.r().round();
        let si = self.s().round();
        let q_diff = qi - self.q();
        let r_diff = ri - self.r();
        let s_diff = si - self.s();
        if q_diff > r_diff && q_diff > s_diff {
            qi = -ri - si;
        } else if r_diff > s_diff {
            ri = -qi - si;
        }
        Cube(qi, ri).into()
    }
    fn lerp(&self, &other: &Cube<f32>, t: f32) -> Cube<f32> {
        Cube(
            self.q() * (1.0 - t) + other.q() * t,
            self.r() * (1.0 - t) + other.r() * t,
        )
    }
}

impl Cube<i32> {
    // https://gamedev.stackexchange.com/a/51267
    fn ring(&self, n: usize) -> Vec<Cube<i32>> {
        let mut result = vec![Cube(0,0); 6 * n];
        let mut cube = *self + DIRECTIONS[4] * n as i32; // 4 = starting_index (i.e. 0) - 2 
        for (i, direction) in DIRECTIONS.iter().enumerate() {
            for j in 0..n {
                let counter = i * n + j;
                result[counter] = cube;
                cube += *direction;
            };
        };
        result
    }
    pub fn disc(&self, n: usize) -> Vec<Cube<i32>> {
        // note: output vec len is equal to the triangular number of n times 6
        let mut result = self.ring(0);
        for ring_number in 1..n+1 {
            let mut ring = self.ring(ring_number);
            result.append(&mut ring);
        }
        result
    }
}

impl<T> Cube<T> where T: Copy + Signed + Div<i32, Output=T> { // , Cube<T>: From<Cube<i32>>  From<Cube<i32>> + Copy + Signed...
    fn length(&self) -> T {
        (self.q().abs() + self.r().abs() + self.s().abs()) / 2
    }
    fn distance(&self, &rhs: &Cube<T>) -> T {
        let delta_cube = *self - rhs;
        (delta_cube.q().abs() + delta_cube.r().abs() + delta_cube.s().abs()) / 2
    }
}

// pretend DIRECTIONS is a 2x3 matrix
//      6th
//      ___
// 5th /   \ 1st
// 4th \   / 2nd
//      ---
//      3rd
pub const DIRECTIONS: [Cube<i32>; 6] = [
    Cube(1, -1), Cube(1, 0), Cube(0, 1),
    Cube(-1, 1), Cube(-1, 0), Cube(0, -1),
];

// Because Rev<Iter> is a different type to Iter
pub const REV_DIRECTIONS: [Cube<i32>; 6] = [
    Cube(0, -1), Cube(-1, 0), Cube(-1, 1),
    Cube(0, 1), Cube(1, 0), Cube(1, -1),
];

// impl<T> Point<T> where T: Copy + Signed + Div<i32, Output=T> {
    // fn get_neighbour(&self, direction: usize) -> Point<T> {
    //     *self + DIRECTIONS[direction]
    // }

    // fn get_nearest_neighbours(&self) -> [Point<T>; 6] {
    //     DIRECTIONS.map(|other| *self + other)
    // }

    // fn get_n_nearest_neighbours(&self, n: u32) -> Vec<Point<T>> {
    //     let mut nnn: HashSet<Point<T>> = HashSet::from_iter([*self]);
    //     for _ in 0..n {
    //         for neighbour in nnn.clone() {
    //             let nn = neighbour.get_nearest_neighbours();
    //             nnn.extend(nn);
    //         };
    //     };
    //     nnn.remove(self);
    //     Vec::from_iter(nnn)
    // }
// }

// #[derive(Default)]
// struct Board<T: Default, const N: usize> {
//     elems: [T; N]
// }

#[derive(Clone, Copy)]
pub struct Orientation<T> {
    f0: T,
    f1: T,
    f2: T,
    f3: T,
    b0: T,
    b1: T,
    b2: T,
    b3: T,
    start_angle: T,
}

#[derive(Clone, Copy)]
pub struct Layout<T> {
    pub orientation: OrientationKind<T>,
    pub size: [T; 2],
    pub origin: [T; 2]
}

impl<T: Copy + From<f32> + Mul<Output = T> + Div<Output = T> + Sub<Output = T> + Add<Output = T>> Layout<T> {
    fn align_top_left(&mut self, radius: T) {
        // R = Layout.size
        // r = cos30 R = sqrt(3)/2 R
        let (_2, _4) = (T::from(2.), T::from(4.));
        let factor =  T::from(3f32.sqrt() / 2.);
        let r = [self.size[0] * factor, self.size[1] * factor];
        let R = self.size;
        let (mut x, mut y) = (T::from(0.), T::from(0.));
        match self.orientation {
            OrientationKind::Pointy(_) => {
                x = (_2 * r[0] * radius) - r[0];
                // every 4 tiles skip one R
                y = (_2 * R[1] * radius) - (_2 * R[1] * radius/_4) + R[1];
            }
            OrientationKind::Flat(_) => {
                y = (_2 * r[1] * radius);
                // every 4 tiles skip one R
                x = (_2 * R[0] * radius) - (_2 * R[0] * radius/_4) -R[0]/_2;
            }
        }
        self.origin = [x, y];
    }
}

// no const sqrt() yet...
// const SQRT3: f64 = 1.732050807568877293527446341505872366942805253810380628055806; // sqrt(3)
const SQRT3: f32 = 1.732050807568877293527446341505872366942805253810380628055806; // sqrt(3)


#[derive(Clone, Copy)]
pub enum OrientationKind<T> {
    Pointy(Orientation<T>),
    Flat(Orientation<T>),
    // Pointy(T),
    // Flat(T),
}

// impl<T: From<f32> + Div<Output = T>> OrientationKind<T> {
//     fn pointy() -> Orientation<T> {
//         let sqrt3 = 3f32.sqrt();
//         return Orientation {
//             f0: T::from(sqrt3),
//             f1: T::from(sqrt3 / 2.0),
//             f2: T::from(0.0), 
//             f3: T::from(3.0 / 2.0),
//             b0: T::from(sqrt3 / 3.0),
//             b1: T::from(-1.0 / 3.0), 
//             b2: T::from(0.0), 
//             b3: T::from(2.0 / 3.0),
//             start_angle: T::from(0.5)
//         };
//     }
//     fn new(kind: &str) -> Self {
//         match kind {
//             "pointy" => OrientationKind::Pointy((OrientationKind::pointy())),
//             "flat" => OrientationKind::Flat(/* OrientationKind::flat() */),
//             _ => panic!("bad enum option")
//         }
//     }
// }

impl<T> OrientationKind<T> {
    fn inner(&self) -> &Orientation<T> {
        match self {
            OrientationKind::Pointy(v) => v,
            OrientationKind::Flat(v) => v,
        }
    }
}

pub const POINTY: Orientation<f32> = Orientation {
    f0: SQRT3, 
    f1: SQRT3 / 2.0, 
    f2: 0.0, 
    f3: 3.0 / 2.0,
    b0: SQRT3 / 3.0, 
    b1: -1.0 / 3.0, 
    b2: 0.0, 
    b3: 2.0 / 3.0, 
    start_angle: 0.5
};

pub const FLAT: Orientation<f32> = Orientation {
    f0: 3.0 / 2.0,
    f1: 0.0,
    f2: SQRT3 / 2.0,
    f3: SQRT3,
    b0: 2.0 / 3.0,
    b1: 0.0,
    b2: -1.0 / 3.0,
    b3: SQRT3 / 3.0,
    start_angle: 0.0,
};

// impl<T> TryFrom<OrientationKind<T>> for Orientation<T> {
//     fn try_from(other: OrientationKind<T>) -> core::result::Result<Self, Self::Error> {
//         type Error = OrientationKind<f32>;

//         match other {
//             OrientationKind::Pointy(c) => Ok(c),
//             a => Err(a),
//         }
//     }
// }

// impl<T> OrientationKind<T> {
//     fn value(&self) -> Orientation<T> {
//         match self {
//             OrientationKind::Pointy => Orientation {
//                 f0: SQRT3, 
//                 f1: SQRT3 / 2.0, 
//                 f2: 0.0, 
//                 f3: 3.0 / 2.0,
//                 b0: SQRT3 / 3.0, 
//                 b1: -1.0 / 3.0, 
//                 b2: 0.0, 
//                 b3: 2.0 / 3.0, 
//                 start_angle: 0.5
//             },
//             OrientationKind::Flat => Orientation {
//                 f0: 3.0 / 2.0,
//                 f1: 0.0,
//                 f2: SQRT3 / 2.0,
//                 f3: SQRT3,
//                 b0: 2.0 / 3.0,
//                 b1: 0.0,
//                 b2: -1.0 / 3.0,
//                 b3: SQRT3 / 3.0,
//                 start_angle: 0.0,
//             },
//         }
//     }
// }

trait Float<T = Self> = Copy + From<f32> + Mul<Output = T> + Div<Output = T> + Sub<Output = T> + Add<Output = T>;

// TODO: I removed a From<f32> here, I suspect this breaks stuff but so far everything works.
pub fn pixel_to_cube<T: Copy + Mul<Output = T> + Div<Output = T> + Sub<Output = T> + Add<Output = T>>(&layout: &Layout<T>, pixel: [T; 2]) -> Cube<T> {
    let matrix = layout.orientation.inner();
    let size = layout.size;
    let origin = layout.origin;
    let pt = [(pixel[0] - origin[0]) / size[0], (pixel[1] - origin[1]) / size[1]];
    let q = matrix.b0 * pt[0] + matrix.b1 * pt[1];
    let r = matrix.b2 * pt[0] + matrix.b3 * pt[1];
    Cube(q, r)
}

impl Div<i32> for Cube<i32> {
    type Output = Cube<f32>;

    fn div(self, rhs: i32) -> Self::Output {
        if rhs == 0 {
            panic!("Cannot divide by zero!");
        }

        Cube::new(self.q() as f32 / rhs as f32, self.r() as f32 / rhs as f32)
    }
}

impl Neg for Cube<i32> {
    type Output = Cube<i32>;

    fn neg(self) -> Self::Output {
        Cube::new(-self.q(), -self.r())
    }
}

#[allow(dead_code)]
fn triangular_number(n: i32) -> i32 {
    let mut triangular_number = n;
    for i in 1..n {
        triangular_number += n - i;
    }
    triangular_number
}

fn equal_cube(name: &str, a:Cube<f32>, b:Cube<f32>){
    // assert!(a.q() == b.q() && a.s() == b.s() && a.r() == b.r());
    println!{"{}: {}, a: {:?} b: {:?}", name, a == b, a, b};
    //complain(name)
}

fn test_cube_round() {
    let a = Cube(0.0, 0.0);
    let b = Cube(1.0, -1.0);
    let c = Cube(0.0, -1.0);
    equal_cube("cube_round 1", Cube(5., -10.), (Cube(0.0, 0.0).lerp(&Cube(10.0, -20.0), 0.5)).round());
    equal_cube("cube_round 2", a.round(), a.lerp(&b, 0.499).round());
    equal_cube("cube_round 3", b.round(), a.lerp(&b, 0.501).round());
    let right1 = Cube(a.q() * 0.4 + b.q() * 0.3 + c.q() * 0.3, a.r() * 0.4 + b.r() * 0.3 + c.r() * 0.3); //, a.s() * 0.4 + b.s() * 0.3 + c.s() * 0.3)).round());
    equal_cube("cube_round 4", a.round(), right1.round());
    let right2 = Cube(a.q() * 0.3 + b.q() * 0.3 + c.q() * 0.4, a.r() * 0.3 + b.r() * 0.3 + c.r() * 0.4); //, a.s() * 0.3 + b.s() * 0.3 + c.s() * 0.4)).round());
    equal_cube("cube_round 5", c.round(), right2.round());
}