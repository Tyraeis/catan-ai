use std::sync::Arc;

use cairo::Context;

use catan::*;

#[derive(Clone, Copy)]
pub enum Structure {
    Settlement, City, Metropolis,
    KnightT1(bool), KnightT2(bool), KnightT3(bool),
}

pub struct VertexStatic {
	pub hexes: [Option<HexCoord>; 3],
	pub edges: [Option<EdgeID>; 3],

    pub hex_position: HexCoord,
    pub hex_side: usize,
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

    pub fn draw(&self, ctx: &Context, id: VertexID) {
        let pos = self.static_data.hex_position;
        let side = self.static_data.hex_side;

        let (hex_x, hex_y) = pos.to_point();
        let dx = HEX_POINTS[side].0;
        let dy = HEX_POINTS[side].1;

        /*ctx.move_to(hex_x + dx, hex_y + dy);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.show_text(&format!("{}", id));
        ctx.fill();*/
    }
}