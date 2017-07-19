use std::sync::Arc;
use std::collections::HashMap;
use std::collections::HashSet;

use catan::*;

pub struct PlayerStatic {
    pub color: [f64; 3],
	pub next_player: PlayerID,
}

#[derive(Clone)]
pub struct Player {
    static_data: Arc<PlayerStatic>,

    pub victory_points: u8,
    pub cities: HashSet<VertexID>,
    pub knights: HashSet<VertexID>,
    pub roads: HashSet<EdgeID>,
    pub ports: HashSet<Resource>,

    pub cards: HashMap<Resource, u8>,
    pub dev_cards: Vec<DevCard>,

    pub yellow_prog: u8,
    pub green_prog: u8,
    pub blue_prog: u8,
}

impl Player {
	pub fn get_color(&self) -> [f64; 3] {
		self.static_data.color
	}

	pub fn get_resource(&self, resource: Resource) -> u8 {
		*self.cards.get(&resource).unwrap_or(&0)
	}

	pub fn consume_resource(&mut self, resource: Resource, n: u8) {
		*self.cards.get_mut(&resource).unwrap() -= n;
	}

	pub fn give_resource(&mut self, resource: Resource, n: u8) {
		*self.cards.entry(resource).or_insert(0) += n;
	}

	pub fn get_buildable_spaces(&self, catan: &Catan) -> (HashSet<EdgeID>, HashSet<VertexID>) {
		let mut visited_edges = HashSet::new();
		let mut visited_vertices = HashSet::new();
		let mut buildable_edges = HashSet::new();
		let mut buildable_vertices = HashSet::new();

		// For each road this player owns:
		//   get the two vertices ajacent to it
		//     if nothing is built on that vertex, mark it as buildable
		//   get the four edges ajacent to it (really, that are ajacent to the 2 vertices)
		//     if nothing is built on that edge, mark it as buildable
		// this should mark all edges and vertices ajacent to the player's roads as buildable

		// this is nested way too deep, but idk of a good way to fix it

		// Iterate over the player's roads
		for edge_id in &self.roads {
			if !visited_edges.contains(edge_id) {
				visited_edges.insert(*edge_id);

				if let Some(edge) = catan.get_edge(edge_id) {
					// Get the vertices ajacent to each road
					for vertex_id in edge.static_data.vertices.iter() {
						if !visited_vertices.contains(vertex_id) {
							visited_vertices.insert(*vertex_id);

							if let Some(vertex) = catan.get_vertex(vertex_id) {
								// Mark the vertex as buildable if there is nothing built on it
								if vertex.structure.is_none() {
									buildable_vertices.insert(*vertex_id);
								}

								// Get the edges ajacent to each road
								for ajacent_edge_id in vertex.static_data.edges.iter().filter_map(|x| *x) {
									if !visited_edges.contains(&ajacent_edge_id) {
										visited_edges.insert(ajacent_edge_id);

										if let Some(ajacent_edge) = catan.get_edge(&ajacent_edge_id) {
											// Mark the edge as buildable if there is nothing built on it
											if ajacent_edge.road.is_none() {
												buildable_edges.insert(ajacent_edge_id);
											}
										}
									}
								}
							}
						}
					}
				}
			}
		}

		(buildable_edges, buildable_vertices)
	}
}