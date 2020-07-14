use std::sync::Arc;
use std::f64::consts::PI;
use std::hash::*;

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

const EDGE_ANGLES: [f64; 3] = [-PI / 3.0, 0.0, PI / 3.0];
impl Edge {
    pub fn new(static_data: EdgeStatic) -> Self {
    	Edge {
        	static_data: Arc::new(static_data),
            road: None
        }
    }

    pub fn draw(&self, ctx: &Context, catan: &Catan, id: EdgeID) {
        if let Some(player) = self.road {
            let pos = self.static_data.hex_position;
            let side = self.static_data.hex_side;

            let (hex_x, hex_y) = pos.to_point();
            let dx = (HEX_POINTS[side].0 + HEX_POINTS[(side + 1) % 6].0) / 2.0;
            let dy = (HEX_POINTS[side].1 + HEX_POINTS[(side + 1) % 6].1) / 2.0;

            ctx.save();
            ctx.translate(hex_x + dx, hex_y + dy);
            ctx.rotate(EDGE_ANGLES[side % 3]);

            let player_color = catan.get_player_color(player);
            ctx.set_source_rgb(player_color[0], player_color[1], player_color[2]);
            ctx.rectangle(-HEX_SCALE / 2.0, -3.0, HEX_SCALE, 6.0);
            ctx.fill_preserve();
            ctx.set_source_rgb(0.0, 0.0, 0.0);
            ctx.stroke();

            ctx.restore();
        }
        /*ctx.move_to(hex_x + dx, hex_y + dy);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.show_text(&format!("{}", id));*/
    }
}

impl Hash for Edge {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.road.hash(state);
    }
}