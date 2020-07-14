use std::collections::HashMap;
use catan::*;
use catan::hex::HexStatic;
use catan::edge::EdgeStatic;
use catan::vertex::VertexStatic;
use catan::player::PlayerStatic;

pub struct BoardBuilder {
	pub hexes: HashMap<HexCoord, HexStatic>,
	pub edges: HashMap<EdgeID, EdgeStatic>,
	pub vertices: HashMap<VertexID, VertexStatic>,
	pub players: HashMap<PlayerID, PlayerStatic>,

	last_edge_id: EdgeID,
	last_vertex_id: VertexID,
	last_player_id: PlayerID,

	pub first_player: PlayerID,
}

fn add_edge_to_vertex(edge: EdgeID, vertex: &mut VertexStatic) {
	if vertex.edges[0] == Some(edge) || vertex.edges[1] == Some(edge) || vertex.edges[2] == Some(edge) {
		return;
	}

	if vertex.edges[0].is_none() {
		vertex.edges[0] = Some(edge);
	} else if vertex.edges[1].is_none() {
		vertex.edges[1] = Some(edge);
	} else {
		vertex.edges[2] = Some(edge);
	}
}

impl BoardBuilder {
	pub fn new() -> Self {
		BoardBuilder {
			hexes: HashMap::new(),
			edges: HashMap::new(),
			vertices: HashMap::new(),
			players: HashMap::new(),

			last_edge_id: 0,
			last_vertex_id: 0,
			last_player_id: 0,

			first_player: 0,
		}
	}

	fn get_edge(&mut self, pos: HexCoord, n: usize, vertices: [VertexID; 6]) -> EdgeID {
		let neighbor_pos = &pos + &HEX_DIRECTIONS[n];

		if let Some(neighbor) = self.hexes.get(&neighbor_pos) {
			let edge_id = neighbor.edges[(n + 3) % 6];
			self.edges.get_mut(&edge_id).unwrap().hexes[1] = Some(pos);
			edge_id
		} else {
			self.last_edge_id += 1;

			self.edges.insert(self.last_edge_id, EdgeStatic {
				hexes: [Some(pos), None],
				vertices: [vertices[n], vertices[(n + 1) % 6]],

				hex_position: pos,
				hex_side: n,
			});
			add_edge_to_vertex(self.last_edge_id, self.vertices.get_mut(&vertices[n]).unwrap());
			add_edge_to_vertex(self.last_edge_id, self.vertices.get_mut(&vertices[(n + 1) % 6]).unwrap());
			self.last_edge_id
		}
	}

	fn get_vertex(&mut self, pos: HexCoord, n: usize) -> VertexID {
		let neighbor_pos = &pos + &HEX_DIRECTIONS[n];
		if let Some(neighbor) = self.hexes.get(&neighbor_pos) {
			let vertex_id = neighbor.vertices[(n + 4) % 6];
			let vertex = self.vertices.get_mut(&vertex_id).unwrap();
			if vertex.hexes[1].is_none() {
				vertex.hexes[1] = Some(pos);
			} else {
				vertex.hexes[2] = Some(pos);
			}
			vertex_id
		} else {
			let neighbor2_pos = &pos + &HEX_DIRECTIONS[(n + 6 - 1) % 6];
			if let Some(neighbor2) = self.hexes.get(&neighbor2_pos) {
				let vertex_id = neighbor2.vertices[(n + 2) % 6];
				let vertex = self.vertices.get_mut(&vertex_id).unwrap();
				if vertex.hexes[1].is_none() {
					vertex.hexes[1] = Some(pos);
				} else {
					vertex.hexes[2] = Some(pos);
				}
				vertex_id
			} else {
				self.last_vertex_id += 1;
				self.vertices.insert(self.last_vertex_id, VertexStatic {
					hexes: [Some(pos), None, None],
					edges: [None, None, None],

					hex_position: pos,
					hex_side: n,
				});
				self.last_vertex_id
			}
		}
	}

	pub fn add_hex(&mut self, pos: HexCoord, typ: HexType, roll: u8) {
		let vertices = [
			self.get_vertex(pos, 0), self.get_vertex(pos, 1), self.get_vertex(pos, 2), 
			self.get_vertex(pos, 3), self.get_vertex(pos, 4), self.get_vertex(pos, 5)
		];

		let edges = [
			self.get_edge(pos, 0, vertices), self.get_edge(pos, 1, vertices), self.get_edge(pos, 2, vertices), 
			self.get_edge(pos, 3, vertices), self.get_edge(pos, 4, vertices), self.get_edge(pos, 5, vertices)
		];

		let hex = HexStatic {
			typ,
			roll,

			edges,
			vertices,
		};
		self.hexes.insert(pos, hex);
	}

	pub fn add_player(&mut self, color: [f64; 3]) -> PlayerID {
		let player = PlayerStatic {
			color,
			next_player: 0,
			prev_player: 0,
		};
		self.last_player_id += 1;
		self.players.insert(self.last_player_id, player);
		self.last_player_id
	}

	pub fn set_player_order(&mut self, order: Vec<PlayerID>) {
		let mut order0 = order.clone();
		let last = order0.pop().unwrap();
		order0.insert(0, last);

		let mut order2 = order.clone();
		let first = order2.remove(0);
		order2.push(first);

		for (id1, (beforeid, afterid)) in order.iter().zip(order0.iter().zip(order2)) {
			let pl = self.players.get_mut(id1).unwrap();
			pl.prev_player = *beforeid;
			pl.next_player = afterid;
		}

		self.first_player = order[0];
	}
}