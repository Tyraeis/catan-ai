use std::sync::Arc;
use catan::{ EdgeID, VertexID, Resource, Catan };

pub enum HexType {
    Water,
    Port(Resource, u8),
    Land(Resource)
}

pub struct HexStatic {
    typ: HexType,

    edges: [EdgeID; 6],
    vertices: [VertexID; 6],
}

#[derive(Clone)]
pub struct Hex {
    static_data: Arc<HexStatic>,
    roll: u8,
}

impl Hex {
    pub fn new(game: &Catan, typ: HexType) -> Self {
        let static_data = HexStatic {
            typ,

            edges: [0, 0, 0, 0, 0, 0],
            vertices: [0, 0, 0, 0, 0, 0]
        };

        Hex {
            static_data: Arc::new(static_data),
            roll: 0
        }
    }
}