use std::collections::HashMap;

use ai::Game;

use catan::hex_coord::*;
use catan::hex::*;
use catan::edge::*;
use catan::vertex::*;
use catan::player::*;

pub enum Resource {
    Wheat, Sheep, Brick, Wood, Rock,
    Paper, Coins, Cloth
}

pub type EdgeID   = usize;
pub type VertexID = usize;
pub type PlayerID = usize;

#[derive(Clone)]
pub struct Catan {
    hexes:    HashMap<HexCoord, Hex>,
    edges:    HashMap<EdgeID,   Edge>,
    vertices: HashMap<VertexID, Vertex>,
    players:  HashMap<PlayerID, Player>,
}

impl Catan {
    pub fn new() -> Self {
        Catan {
            hexes:    HashMap::new(),
            edges:    HashMap::new(),
            vertices: HashMap::new(),
            players:  HashMap::new()
        }
    }
}

impl Game for Catan {
    type Move = ();
    type Player = PlayerID;

    fn available_moves(&self) -> Vec<Self::Move> {
        Vec::new()
    }
    fn make_move_mut(&mut self, m: &Self::Move) -> bool {
        false
    }
    fn get_cur_player(&self) -> Self::Player {
        0
    }
    fn get_winner(&self) -> Option<Self::Player> {
        Some(0)
    }
}