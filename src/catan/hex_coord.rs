use std::ops::*;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct HexCoord {
    q: i16,
    r: i16,
}

static HEX_DIRECTIONS: [HexCoord; 6] = [
    HexCoord { q:  1, r: 0 }, HexCoord { q:  1, r: -1 }, HexCoord { q: 0, r: -1 },
    HexCoord { q: -1, r: 0 }, HexCoord { q: -1, r:  1 }, HexCoord { q: 0, r:  1 },
];

impl HexCoord {
    pub fn neighbors(&self) -> [HexCoord; 6] {
        [
            self.neighbor(0), self.neighbor(1), self.neighbor(2),
            self.neighbor(3), self.neighbor(4), self.neighbor(5)
        ]
    }

    pub fn neighbor(&self, n: usize) -> HexCoord {
        self + &HEX_DIRECTIONS[n % 6]
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