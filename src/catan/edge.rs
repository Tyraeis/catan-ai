use std::sync::Arc;

use cairo::Context;

use catan::*;

pub struct EdgeStatic {
	pub hexes: [Option<HexCoord>; 2],
	pub vertices: [VertexID; 2],

    pub hex_position: HexCoord,
    pub hex_side: usize,
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

    pub fn draw(&self, ctx: &Context, id: EdgeID) {
        let pos = self.static_data.hex_position;
        let side = self.static_data.hex_side;

        let (hex_x, hex_y) = pos.to_point();
        let dx = (HEX_POINTS[side].0 + HEX_POINTS[(side + 1) % 6].0) / 2.0;
        let dy = (HEX_POINTS[side].1 + HEX_POINTS[(side + 1) % 6].1) / 2.0;

        /*ctx.move_to(hex_x + dx, hex_y + dy);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.show_text(&format!("{}", id));
        ctx.fill();*/
    }
}