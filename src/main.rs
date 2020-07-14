#![feature(inclusive_range_syntax)]
#![feature(const_fn)]
#![feature(vec_remove_item)]
#![feature(ord_max_min)]

extern crate gtk;
extern crate cairo;
extern crate rand;
extern crate num_cpus;
extern crate lazy_static;

use std::rc::Rc;
use std::cell::{ Cell, RefCell };
use std::time::{ Instant, Duration };

use gtk::prelude::*;
use gtk::{ Window, Label, DrawingArea, Paned };
use gtk::{ WindowType, WindowPosition, Orientation };
use cairo::{ FontSlant, FontWeight };
use rand::{ thread_rng, Rng };

mod catan;
mod ai;

use catan::*;
use ai::*;

const AI_TURN_TIME: u64 = 3; // seconds
const HUMAN_PLAYER: bool = false;

const fn color(hex: u32) -> [f64; 3] {
    [
        (hex >> 16 & 0xFF) as f64 / 0xFF as f64,
        (hex >> 8  & 0xFF) as f64 / 0xFF as f64,
        (hex       & 0xFF) as f64 / 0xFF as f64,
    ]
}
const WATER_COLOR: [f64; 3] = color(0x0026E5);


fn main() {
    let mut builder = BoardBuilder::new();

    builder.add_hex(HexCoord::new(-3,  0), HexType::Water, 0);
    builder.add_hex(HexCoord::new(-3,  1), HexType::Port3to1(0), 0);
    builder.add_hex(HexCoord::new(-3,  2), HexType::Water, 0);
    builder.add_hex(HexCoord::new(-3,  3), HexType::Port2to1(Resource::Rock, 5), 0);

    builder.add_hex(HexCoord::new(-2, -1), HexType::Port3to1(0), 0);
    builder.add_hex(HexCoord::new(-2,  0), HexType::Desert, 0);
    builder.add_hex(HexCoord::new(-2,  1), HexType::Land(Resource::Brick), 8);
    builder.add_hex(HexCoord::new(-2,  2), HexType::Land(Resource::Rock), 5);//
    builder.add_hex(HexCoord::new(-2,  3), HexType::Water, 0);

    builder.add_hex(HexCoord::new(-1, -2), HexType::Water, 0);
    builder.add_hex(HexCoord::new(-1, -1), HexType::Land(Resource::Brick), 4);
    builder.add_hex(HexCoord::new(-1,  0), HexType::Land(Resource::Wood), 3);
    builder.add_hex(HexCoord::new(-1,  1), HexType::Land(Resource::Sheep), 10);
    builder.add_hex(HexCoord::new(-1,  2), HexType::Land(Resource::Wheat), 2);//
    builder.add_hex(HexCoord::new(-1,  3), HexType::Port2to1(Resource::Brick, 4), 0);

    builder.add_hex(HexCoord::new( 0, -3), HexType::Port2to1(Resource::Sheep, 1), 0);
    builder.add_hex(HexCoord::new( 0, -2), HexType::Land(Resource::Wood), 11);
    builder.add_hex(HexCoord::new( 0, -1), HexType::Land(Resource::Rock), 6);
    builder.add_hex(HexCoord::new( 0,  0), HexType::Land(Resource::Wheat), 11);
    builder.add_hex(HexCoord::new( 0,  1), HexType::Land(Resource::Sheep), 9);
    builder.add_hex(HexCoord::new( 0,  2), HexType::Land(Resource::Wood), 6);//
    builder.add_hex(HexCoord::new( 0,  3), HexType::Water, 0);

    builder.add_hex(HexCoord::new( 1, -3), HexType::Water, 0);
    builder.add_hex(HexCoord::new( 1, -2), HexType::Land(Resource::Sheep), 12);
    builder.add_hex(HexCoord::new( 1, -1), HexType::Land(Resource::Brick), 5);
    builder.add_hex(HexCoord::new( 1,  0), HexType::Land(Resource::Wood), 4);
    builder.add_hex(HexCoord::new( 1,  1), HexType::Land(Resource::Rock), 3);//
    builder.add_hex(HexCoord::new( 1,  2), HexType::Port3to1(4), 0);

    builder.add_hex(HexCoord::new( 2, -3), HexType::Port3to1(2), 0);
    builder.add_hex(HexCoord::new( 2, -2), HexType::Land(Resource::Wheat), 9);
    builder.add_hex(HexCoord::new( 2, -1), HexType::Land(Resource::Sheep), 10);
    builder.add_hex(HexCoord::new( 2,  0), HexType::Land(Resource::Wheat), 8);
    builder.add_hex(HexCoord::new( 2,  1), HexType::Water, 0);

    builder.add_hex(HexCoord::new( 3, -3), HexType::Water, 0);
    builder.add_hex(HexCoord::new( 3, -2), HexType::Port2to1(Resource::Wheat, 2), 0);
    builder.add_hex(HexCoord::new( 3, -1), HexType::Water, 0);
    builder.add_hex(HexCoord::new( 3,  0), HexType::Port2to1(Resource::Wood, 3), 0);

    let p1 = builder.add_player([1.0, 0.0, 0.0]);
    let p2 = builder.add_player([0.0, 1.0, 0.0]);
    let p3 = builder.add_player([0.0, 0.0, 1.0]);

    builder.set_player_order(vec![p1, p2, p3]);

    let ai_player = p1;

    let (_catan, _ai) = {
        let mut catan = Catan::new(builder);
        let mut ai = Ai::new(catan.clone());
        (
            Rc::new(RefCell::new(catan)),
            Rc::new(RefCell::new(ai))
        )
    };
    let pending_move = Rc::new(Cell::new(false));

    if gtk::init().is_err() {
    	println!("Failed to initialize GTK.");
    	return;
    }


    let draw_area = DrawingArea::new();
    {
        let catan = _catan.clone();
    	draw_area.connect_draw(move |this, ctx| {
    		let w = this.get_allocated_width() as f64;
    		let h = this.get_allocated_height() as f64;

            
    		ctx.set_source_rgb(WATER_COLOR[0], WATER_COLOR[1], WATER_COLOR[2]);
			ctx.paint();

            ctx.select_font_face("serif", FontSlant::Normal, FontWeight::Bold);
			catan.borrow().draw(ctx, w, h);

			Inhibit(false)
    	});
    }

    let player_label = Label::new("<tt>Player: <span foreground=\"#000000\">X</span></tt>");
	let best_move_label = Label::new("<tt>Best Move: None</tt>");
	let confidence_label = Label::new("<tt>Confidence: <span foreground=\"#ffff00\">0%</span></tt>");
	let num_sims_label = Label::new("<tt>Simulations: 0</tt>");
	let time_label = Label::new("<tt>Elapsed Time: 0 seconds</tt>");
	let rate_label = Label::new("<tt>0 sims/second</tt>");
    let num_moves_label = Label::new("<tt>0 possible moves</tt>");
	let ai_time_left_label = Label::new("");
	player_label.set_xalign(0.0);
	best_move_label.set_xalign(0.0);
	confidence_label.set_xalign(0.0);
	num_sims_label.set_xalign(0.0);
	time_label.set_xalign(0.0);
	rate_label.set_xalign(0.0);
    num_moves_label.set_xalign(0.0);
	ai_time_left_label.set_xalign(0.0);

	let right_container = gtk::Box::new(Orientation::Vertical, 8);
	right_container.set_border_width(8);
	right_container.pack_start(&player_label, false, false, 0);
	right_container.pack_start(&best_move_label, false, false, 0);
	right_container.pack_start(&confidence_label, false, false, 0);
	right_container.pack_start(&num_sims_label, false, false, 0);
	right_container.pack_start(&time_label, false, false, 0);
	right_container.pack_start(&rate_label, false, false, 0);
    right_container.pack_start(&num_moves_label, false, false, 0);
	right_container.pack_start(&ai_time_left_label, false, false, 0);

	let container = Paned::new(Orientation::Horizontal);
	container.set_position(900);
	container.pack1(&draw_area, true, true);
	container.pack2(&right_container, false, true);

    let window = Window::new(WindowType::Toplevel);
    window.set_title("Catan AI");
    window.set_position(WindowPosition::Center);
    window.set_default_size(1200, 720);
    window.connect_delete_event(|_, _| {
    	gtk::main_quit();
    	Inhibit(false)
    });
    window.add(&container);
    window.show_all();

    {
        _ai.borrow().send(Request::Info);
        let mut last_move = Instant::now();
        let mut rand = thread_rng();
        let da = draw_area.clone();
        
        gtk::idle_add(move || {
            let ai = _ai.borrow();
            let mut catan = _catan.borrow_mut();
            while let Some(res) = ai.recv() {
                match res {
                    Response::Info { best_move, is_random, confidence, total_sims, time_elapsed, possible_moves } => {
						let player = catan.get_cur_player().clone();

						let move_str = if !HUMAN_PLAYER || player == ai_player {
							best_move.clone().map(|i| format!("{:?}", i)).unwrap_or(String::from("None"))
						} else {
							"Hidden".to_owned()
						};
						let time = time_elapsed.as_secs();
						let subsec_time = time as f64 + (time_elapsed.subsec_nanos() as f64 / 1_000_000_000.0);
						let rate = (total_sims as f64 / subsec_time).floor();
						let confidence_pct = (confidence*100.0).floor();
						let confidence_col = format!("#{:02x}{:02x}00", (255.0*(1.0-confidence)) as u8, (255.0*confidence) as u8);

						best_move_label.set_markup(&format!("<tt>Best Move: {}</tt>", move_str));
						confidence_label.set_markup(&format!("<tt>Confidence: <span foreground=\"{}\">{}%</span></tt>", confidence_col, confidence_pct));
						num_sims_label.set_markup(&format!("<tt>Simulations: {}</tt>", total_sims));
						time_label.set_markup(&format!("<tt>Elapsed Time: {} seconds</tt>", time));
						rate_label.set_markup(&format!("<tt>{} sims/second</tt>", rate));
                        num_moves_label.set_markup(&format!("<tt>{} possible moves</tt>", possible_moves.len()));
						if !HUMAN_PLAYER || player == ai_player {
							let ai_time = Instant::now().duration_since(last_move).as_secs();
							if ai_time <= AI_TURN_TIME {
								ai_time_left_label.set_markup(&format!("<tt>{} seconds left</tt>", AI_TURN_TIME - ai_time));
							}
						} else {
							ai_time_left_label.set_text("");
						}

						ai.send(Request::Info);


                        if (is_random || possible_moves.len() == 1) && !pending_move.get() {
                            if let Some(mv) = rand.choose(&possible_moves) {
                                catan.make_move(mv);
                                da.queue_draw();

                                ai.make_move(mv.clone());
                                pending_move.set(true);
                            }
                        } else if (!HUMAN_PLAYER || player == ai_player) && !pending_move.get() && last_move.elapsed() > Duration::from_secs(AI_TURN_TIME) {
							if let Some(mv) = best_move {
								catan.make_move(&mv);
								da.queue_draw();

								ai.make_move(mv);
								pending_move.set(true);
							}
						}

						//let player = game.borrow().get_cur_player();
						let player_str = player.to_string();
						let p_col_arr = catan.get_player_color(player);
                        let player_col = format!("#{:02X}{:02X}{:02X}", 
                            (p_col_arr[0].max(0.0).min(255.0) * 255.0) as i8, 
                            (p_col_arr[1].max(0.0).min(255.0) * 255.0) as i8, 
                            (p_col_arr[2].max(0.0).min(255.0) * 255.0) as i8
                        );
						player_label.set_markup(&format!("<tt>Player: <span foreground=\"{}\">{}</span></tt>", player_col, player_str));
					},
					Response::Ok => {
						last_move = Instant::now();
						pending_move.set(false);
					},
                }
            }

            Continue(true)
        });
    }

    gtk::main();
}