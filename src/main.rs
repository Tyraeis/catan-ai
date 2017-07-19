#![feature(inclusive_range_syntax)]
#![feature(const_fn)]

extern crate gtk;
extern crate cairo;
extern crate rand;
extern crate num_cpus;
extern crate lazy_static;

use std::rc::Rc;

use gtk::prelude::*;
use gtk::{ Window, DrawingArea };
use gtk::{ WindowType, WindowPosition };

mod catan;
mod ai;

use catan::*;

fn main() {
    let mut builder = BoardBuilder::new();
    builder.add_hex(HexCoord::new(0, 0), HexType::Land(Resource::Wood));
    builder.add_hex(HexCoord::new(1, 0), HexType::Land(Resource::Sheep));
    builder.add_hex(HexCoord::new(0, -1), HexType::Land(Resource::Brick));
    builder.add_hex(HexCoord::new(1, -1), HexType::Land(Resource::Rock));
    builder.add_hex(HexCoord::new(-1, 0), HexType::Land(Resource::Wheat));
    builder.add_hex(HexCoord::new(-1, 1), HexType::Water);
    builder.add_hex(HexCoord::new(0, 1), HexType::Desert);

    let p1 = builder.add_player([1.0, 0.0, 0.0]);
    let p2 = builder.add_player([0.0, 1.0, 0.0]);
    let p3 = builder.add_player([0.0, 0.0, 1.0]);

    builder.set_player_order(vec![p1, p2, p3]);

    let mut catan = Catan::new(builder);

    if gtk::init().is_err() {
    	println!("Failed to initialize GTK.");
    	return;
    }


    let draw_area = DrawingArea::new();
    {
    	draw_area.connect_draw(move |this, ctx| {
    		let w = this.get_allocated_width() as f64;
    		let h = this.get_allocated_height() as f64;

    		ctx.set_source_rgb(1.0, 1.0, 1.0);
			ctx.paint();

			catan.draw(ctx, w, h);

			Inhibit(false)
    	});
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_title("Catan AI");
    window.set_position(WindowPosition::Center);
    window.set_default_size(1200, 720);
    window.connect_delete_event(|_, _| {
    	gtk::main_quit();
    	Inhibit(false)
    });
    window.add(&draw_area);
    window.show_all();

    gtk::main();
}