use std::sync::Arc;
use std::collections::HashMap;
use std::cell::RefCell;

use cairo::Context;

use catan::*;
use catan::player::Player;

#[derive(Clone, Copy)]
pub enum HexType {
    Water,
    Port(Resource, u8),
    Land(Resource),
    Desert,
}

pub struct HexStatic {
    pub typ: HexType,

    pub edges: [EdgeID; 6],
    pub vertices: [VertexID; 6],
}

#[derive(Clone)]
pub struct Hex {
    pub static_data: Arc<HexStatic>,
    pub roll: u8,
}

impl Hex {
    pub fn new(static_data: HexStatic) -> Self {
        Hex {
            static_data: Arc::new(static_data),
            roll: 0
        }
    }

    pub fn draw(&self, ctx: &Context, pos: &HexCoord) {
        ctx.save();

        let (offx, offy) = pos.to_point();
        ctx.translate(offx, offy);
        ctx.move_to(HEX_POINTS[0].0, HEX_POINTS[0].1);
        for &(x, y) in HEX_POINTS.iter().skip(1) {
            ctx.line_to(x, y);
        }

        let col = match self.static_data.typ {
            HexType::Water => [0.4, 0.4, 1.0],
            HexType::Port(_, _) => [0.4, 0.4, 1.0],
            HexType::Desert => [0.6, 0.5, 0.2],
            HexType::Land(resource) => match resource {
                Resource::Wheat => [1.0, 1.0, 0.3],
                Resource::Sheep => [0.3, 1.0, 0.3],
                Resource::Brick => [1.0, 0.3, 0.3],
                Resource::Wood  => [0.3, 0.6, 0.3],
                Resource::Rock  => [0.3, 0.3, 0.3],
                _     => [1.0, 0.0, 1.0],
            },
        };

        ctx.set_source_rgb(col[0], col[1], col[2]);
        ctx.fill();

        /*ctx.move_to(0.0, 0.0);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.show_text(&format!("{:?}", pos));
        ctx.fill();*/

        ctx.restore();
    }
}