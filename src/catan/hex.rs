use std::sync::Arc;
use std::collections::HashMap;
use std::cell::RefCell;
use std::f64::consts::PI;

use cairo::Context;

use catan::*;
use catan::player::Player;
use catan::hex_coord::HEX_SCALE;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HexType {
    Water,
    Port2to1(Resource, u8),
    Port3to1(u8),
    Desert,
    Land(Resource),
}

pub struct HexStatic {
    pub typ: HexType,
    pub roll: u8,

    pub edges: [EdgeID; 6],
    pub vertices: [VertexID; 6],
}

#[derive(Clone)]
pub struct Hex {
    pub static_data: Arc<HexStatic>,
}

const fn color(hex: u32) -> [f64; 3] {
    [
        (hex >> 16 & 0xFF) as f64 / 0xFF as f64,
        (hex >> 8  & 0xFF) as f64 / 0xFF as f64,
        (hex       & 0xFF) as f64 / 0xFF as f64,
    ]
}

fn draw_port_lines(ctx: &Context, side: usize) {
    ctx.set_source_rgb(1.0, 1.0, 1.0);
    ctx.move_to(0.0, 0.0);
    ctx.line_to(HEX_POINTS[side].0, HEX_POINTS[side].1);
    ctx.stroke();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(HEX_POINTS[(side+1)%6].0, HEX_POINTS[(side+1)%6].1);
    ctx.stroke();
}

impl Hex {
    pub fn new(static_data: HexStatic) -> Self {
        Hex {
            static_data: Arc::new(static_data),
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
            HexType::Water | HexType::Port2to1(_, _) | HexType::Port3to1(_) 
                                => color(0x2D4CE5),
            HexType::Desert     => color(0xFFD000),
            HexType::Land(resource) => match resource {
                Resource::Wheat => color(0xFFE032),
                Resource::Sheep => color(0xB7E045),
                Resource::Brick => color(0xFF5126),
                Resource::Wood  => color(0x779B0C),
                Resource::Rock  => color(0x7C7C7C),
                _               => color(0xFF00FF),
            },
        };

        ctx.set_source_rgb(col[0], col[1], col[2]);

        match self.static_data.typ {
            HexType::Desert | HexType::Land(_) => {
                ctx.fill_preserve();
                let border_col = color(0xFFEBB2);
                ctx.line_to(HEX_POINTS[0].0, HEX_POINTS[0].1);
                ctx.set_source_rgb(border_col[0], border_col[1], border_col[2]);
                ctx.set_line_width(3.0);
                ctx.stroke();
            }
            _ => {
                ctx.fill();
            }
        }

        let roll = self.static_data.roll;
        if roll != 0 {
            ctx.arc(0.0, 0.0, HEX_SCALE / 4.0, 0.0, 2.0*PI);
            let disc_col = color(0xFFEBB2);
            ctx.set_source_rgb(disc_col[0], disc_col[1], disc_col[2]);
            ctx.fill_preserve();
            ctx.set_source_rgb(0.0, 0.0, 0.0);
            ctx.set_line_width(1.5);
            ctx.stroke();

            ctx.set_font_size(16.0);
            let text = roll.to_string();
            let extents = ctx.text_extents(&text);
            ctx.move_to(-extents.x_advance / 2.0, extents.height / 2.0);
            if roll == 6 || roll == 8 {
                ctx.set_source_rgb(1.0, 0.0, 0.0);
            }
            ctx.show_text(&text);
        }

        if let HexType::Port2to1(resource, side) = self.static_data.typ {
            draw_port_lines(ctx, side as usize);
            ctx.arc(0.0, 0.0, HEX_SCALE / 3.0, 0.0, 2.0*PI);
            let color = match resource {
                Resource::Wheat => color(0xFFE032),
                Resource::Sheep => color(0xB7E045),
                Resource::Brick => color(0xFF5126),
                Resource::Wood  => color(0x779B0C),
                Resource::Rock  => color(0x7C7C7C),
                _               => color(0xFF00FF),
            };
            ctx.set_source_rgb(color[0], color[1], color[2]);
            ctx.fill_preserve();
            ctx.set_source_rgb(1.0, 1.0, 1.0);
            ctx.stroke();
        }
        if let HexType::Port3to1(side) = self.static_data.typ {
            draw_port_lines(ctx, side as usize);
            ctx.arc(0.0, 0.0, HEX_SCALE / 3.0, 0.0, 2.0*PI);
            ctx.set_source_rgb(1.0, 1.0, 1.0);
            ctx.fill_preserve();
            ctx.set_source_rgb(0.0, 0.0, 0.0);
            ctx.stroke();
        }

        /*ctx.move_to(0.0, 0.0);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.show_text(&format!("{:?}", pos));*/

        ctx.restore();
    }
}