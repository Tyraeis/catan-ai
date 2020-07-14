use std::sync::Arc;
use std::hash::*;

use cairo::Context;

use catan::*;

#[derive(Clone, Copy, Hash)]
pub enum Structure {
    Settlement, City
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

    pub fn draw(&self, ctx: &Context, catan: &Catan, id: VertexID) {
        if let Some((structure, player)) = self.structure {
            let pos = self.static_data.hex_position;
            let side = self.static_data.hex_side;

            let (hex_x, hex_y) = pos.to_point();
            let dx = HEX_POINTS[side].0;
            let dy = HEX_POINTS[side].1;

            ctx.save();
            ctx.translate(hex_x + dx, hex_y + dy);

            let player_color = catan.get_player_color(player);
            ctx.set_source_rgb(player_color[0], player_color[1], player_color[2]);

            let size = HEX_SCALE / 8.0;
            match structure {
                Structure::Settlement => {
                    ctx.move_to(-size, size);
                    ctx.line_to(-size, -size);
                    ctx.line_to(0.0, -2.0*size);
                    ctx.line_to(size, -size);
                    ctx.line_to(size, size);
                    ctx.close_path();
                }
                Structure::City => {
                    ctx.move_to(-2.0*size, size);
                    ctx.line_to(-2.0*size, -2.0*size);
                    ctx.line_to(-1.0*size, -3.0*size);
                    ctx.line_to(0.0, -2.0*size);
                    ctx.line_to(0.0, -size);
                    ctx.line_to(2.0*size, -size);
                    ctx.line_to(2.0*size, size);
                    ctx.close_path();
                }
            }

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

impl Hash for Vertex {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        //if let Some(structure) = self.structure {
            self.structure.hash(state);
        //}
    }
}