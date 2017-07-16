use std::sync::Arc;

use cairo::Context;

use catan::*;

#[derive(Clone, Copy)]
pub enum Structure {
    Settlement, City, Metropolis,
    KnightT1, KnightT2, KnightT3,
}

pub struct VertexStatic {
	pub hexes: [Option<HexCoord>; 3],
	pub edges: [Option<EdgeID>; 3],
}

#[derive(Clone)]
pub struct Vertex {
	pub static_data: Arc<VertexStatic>,

    pub structure: Option<(Structure, PlayerID)>,
}

impl Vertex {
    pub fn new(static_data: VertexStatic) -> Self {
    	Vertex {
        	static_data: Arc::new(static_data),
            structure: None
        }
    }

    pub fn draw(ctx: &Context, pos: &HexCoord, side: usize) {

    }
}