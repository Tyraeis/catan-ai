use std::collections::HashMap;
use std::f64::INFINITY;

use super::Game;
use super::ai::*;

#[derive(PartialEq, Clone)]
pub(in super) enum Children<G: Game> {
	Random(HashMap<G::Move, (NodeID, f64)>),
	Choice(HashMap<G::Move, NodeID>)
}

/*pub(in super) struct Child {
	pub games: u32,
	pub node: NodeID,
}*/

pub(in super) struct MoveTreeNode<G: Game> {
	pub game: G,
	pub player: G::Player,

	pub score: f64,
	pub games: u32,
	pub wins: u32,
	pub simulations: u32,
	
	pub children: HashMap<G::Move, NodeID>,
	pub weights: Option<HashMap<G::Move, f64>>,
}

impl<G> MoveTreeNode<G> where G: Game {
	pub fn new(game: G) -> Self {
		let player = game.get_cur_player();

		MoveTreeNode {
			game, player,

			score: INFINITY,
			games: 0,
			wins: 0,
			simulations: 0,

			children: HashMap::new(),
			weights: None,
		}
	}
}