use std::ops::*;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct HexCoord {
    q: i16,
    r: i16,
}

pub static HEX_DIRECTIONS: [HexCoord; 6] = [
    HexCoord { q:  1, r: 0 }, HexCoord { q:  1, r: -1 }, HexCoord { q: 0, r: -1 },
    HexCoord { q: -1, r: 0 }, HexCoord { q: -1, r:  1 }, HexCoord { q: 0, r:  1 },
];
const SQRT_3: f64 = 1.732050807568877;
pub const HEX_SCALE: f64 = 48.0;
pub static HEX_POINTS: [(f64, f64); 6] = [
    (HEX_SCALE *  1.0, HEX_SCALE * 0.0), (HEX_SCALE *  0.5, HEX_SCALE *  SQRT_3 / 2.0), (HEX_SCALE * -0.5, HEX_SCALE *  SQRT_3 / 2.0),
    (HEX_SCALE * -1.0, HEX_SCALE * 0.0), (HEX_SCALE * -0.5, HEX_SCALE * -SQRT_3 / 2.0), (HEX_SCALE *  0.5, HEX_SCALE * -SQRT_3 / 2.0) 
];

impl HexCoord {
    pub fn new(q: i16, r: i16) -> Self {
        HexCoord {q, r}
    }

    pub fn neighbors(&self) -> [HexCoord; 6] {
        [
            self.neighbor(0), self.neighbor(1), self.neighbor(2),
            self.neighbor(3), self.neighbor(4), self.neighbor(5)
        ]
    }

    pub fn neighbor(&self, n: usize) -> HexCoord {
        self + &HEX_DIRECTIONS[n % 6]
    }

    pub fn to_point(&self) -> (f64, f64) {
        let x = HEX_SCALE * 1.5 * self.q as f64;
        let y = HEX_SCALE * SQRT_3 * (self.r as f64 + self.q as f64 / 2.0);
        (x, y)
    }
}

use std::fmt::{ Debug, Formatter, Error };
impl Debug for HexCoord {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "({}, {})", self.q, self.r)
    }
}

impl<'a> Add for &'a HexCoord {
    type Output = HexCoord;
    fn add(self, other: &'a HexCoord) -> HexCoord {
        HexCoord { q: self.q + other.q, r: self.r + other.r }
    }
}

impl<'a> Sub for &'a HexCoord {
    type Output = HexCoord;
    fn sub(self, other: &'a HexCoord) -> HexCoord {
        HexCoord { q: self.q - other.q, r: self.r - other.r }
    }
}

impl<'a> Mul for &'a HexCoord {
    type Output = HexCoord;
    fn mul(self, other: &'a HexCoord) -> HexCoord {
        HexCoord { q: self.q * other.q, r: self.r * other.r }
    }
}

impl<'a> Mul<f64> for &'a HexCoord {
    type Output = HexCoord;
    fn mul(self, other: f64) -> HexCoord {
        HexCoord { q: (self.q as f64 * other) as i16, r: (self.r as f64 * other) as i16 }
    }
}