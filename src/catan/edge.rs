use std::sync::Arc;

use cairo::Context;

use catan::*;

pub struct EdgeStatic {
	pub hexes: [Option<HexCoord>; 2],
	pub vertices: [VertexID; 2],
}

#[derive(Clone)]
pub struct Edge {
	pub static_data: Arc<EdgeStatic>,

    pub road: Option<PlayerID>,
}
impl Edge {
    pub fn new(static_data: EdgeStatic) -> Self {
    	Edge {
        	static_data: Arc::new(static_data),
            road: None
        }
    }

    pub fn draw(ctx: &Context, pos: &HexCoord, side: usize) {

    }
}