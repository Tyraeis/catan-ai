use std::collections::HashMap;

use cairo::Context;
use lazy_static::*;

use ai::Game;

use catan::hex_coord::*;
use catan::hex::*;
use catan::edge::*;
use catan::vertex::*;
use catan::player::*;
use catan::board_builder::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resource {
    Wheat, Sheep, Brick, Wood, Rock,
    Paper, Coins, Cloth
}
const ALL_RESOURCES: [Resource; 8] = 
    [ Resource::Wheat, Resource::Sheep, Resource::Brick, Resource::Wood, Resource::Rock,
      Resource::Paper, Resource::Coins, Resource::Cloth ];
const BASIC_RESOURCES: [Resource; 5] =
    [ Resource::Wheat, Resource::Sheep, Resource::Brick, Resource::Wood, Resource::Rock ];
const COMMODITIES: [Resource; 3] =
    [ Resource::Paper, Resource::Coins, Resource::Cloth ];

pub type EdgeID   = usize;
pub type VertexID = usize;
pub type PlayerID = usize;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    PreTurn,
    Turn,
    GameOver,
}

#[derive(Clone)]
pub struct Catan {
    hexes:    HashMap<HexCoord, Hex>,
    edges:    HashMap<EdgeID,   Edge>,
    vertices: HashMap<VertexID, Vertex>,
    players:  HashMap<PlayerID, Player>,

    cur_player: PlayerID,
    state: GameState,
}

impl Catan {
    pub fn new(mut builder: BoardBuilder) -> Self {
        let mut catan = Catan {
            hexes:    HashMap::new(),
            edges:    HashMap::new(),
            vertices: HashMap::new(),
            players:  HashMap::new(),

            cur_player: 0,
            state: GameState::PreTurn,
        };

        for (pos, hex) in builder.hexes.drain() {
            /*println!("Hex {:?}:\n  Edges: {:?}\n  Vertices: {:?}",
                pos, hex.edges, hex.vertices
            );*/
            catan.hexes.insert(pos, Hex::new(hex));
        }

        for (id, edge) in builder.edges.drain() {
            /*println!("Edge {}:\n  Hexes: {:?}\n  Vertices: {:?}",
                id, edge.hexes, edge.vertices
            );*/
            catan.edges.insert(id, Edge::new(edge));
        }

        for (id, vertex) in builder.vertices.drain() {
            /*println!("Vertex {}:\n  Hexes: {:?}\n  Edges: {:?}",
                id, vertex.hexes, vertex.edges
            );*/
            catan.vertices.insert(id, Vertex::new(vertex));
        }

        catan
    }

    pub fn get_hex(&self, pos: &HexCoord) -> Option<&Hex> {
        self.hexes.get(pos)
    }

    pub fn get_vertex(&self, id: &VertexID) -> Option<&Vertex> {
        self.vertices.get(id)
    }

    pub fn get_edge(&self, id: &EdgeID) -> Option<&Edge> {
        self.edges.get(id)
    }

    pub fn draw(&self, ctx: &Context, w: f64, h: f64) {
        ctx.translate(w / 2.0, h / 2.0);

        for (pos, hex) in self.hexes.iter() {
            hex.draw(ctx, pos);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialRoll {
    Yellow, Green, Blue, Barbarian
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum DevCard {
    // Green
    Alchemist,
    Crane,
    Engineer,
    Inventor,
    Irrigation,
    Medicine,
    Mining,
    Printer,
    RoadBuilding,
    Smith,

    // Blue
    Bishop,
    Constitution,
    Deserter,
    Diplomat,
    Intrigue,
    Saboteur,
    Spy,
    Warlord,
    Wedding,

    // Yellow
    CommercialHarbor,
    MasterMerchant,
    Merchant,
    MerchantFleet,
    ResourceMonopoly,
    TradeMonopoly
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum CatanMove {
    // Special
    Roll(u8, u8, SpecialRoll),
    EndTurn,

    // Building / Upgrading
    BuildRoad(EdgeID),
    BuildSettlement(VertexID),
    UpgradeSettlement(VertexID),
    BuildKnight(VertexID),
    UpgradeKnight(VertexID),
    ActivateKnight(VertexID),

    // Development Cards
    UpgradeYellow,
    UpgradeGreen,
    UpgradeBlue,
    PlayDevCard(DevCard),

    // Trade
    Trade {
        to: PlayerID,
        give: Vec<Resource>,
        take: Vec<Resource>,
    },
    Convert4x {
        from: Resource,
        to: Resource,
    },
    Convert2x {
        from: Resource,
        to: Resource,
    },
}

lazy_static! {
    static ref ALL_ROLLS: Vec<CatanMove> = {
        let mut rolls = Vec::new();
        for a in 1...6 {
            for b in 1...6 {
                for c in 1...6 {
                    let special = match c {
                        1 => SpecialRoll::Yellow,
                        2 => SpecialRoll::Green,
                        3 => SpecialRoll::Blue,
                        _ => SpecialRoll::Barbarian,
                    };
                    rolls.push(CatanMove::Roll(a, b, special));
                }
            }
        }
        rolls
    };
}

impl Game for Catan {
    type Move = CatanMove;
    type Player = PlayerID;

    fn available_moves(&self) -> Vec<Self::Move> {
        match self.state {
            GameState::PreTurn => {
                (*ALL_ROLLS).clone()
            },
            GameState::Turn => {
                let mut moves = Vec::new();
                moves.push(CatanMove::EndTurn);

                let player = self.players.get(&self.cur_player).expect("Current player does not exist");

                if player.get_resource(Resource::Cloth) > player.yellow_prog {
                    moves.push(CatanMove::UpgradeYellow);
                }
                if player.get_resource(Resource::Paper) > player.green_prog {
                    moves.push(CatanMove::UpgradeGreen);
                }
                if player.get_resource(Resource::Coins) > player.blue_prog {
                    moves.push(CatanMove::UpgradeBlue);
                }

                let wood = player.get_resource(Resource::Wood);
                let wheat = player.get_resource(Resource::Wheat);
                let brick = player.get_resource(Resource::Brick);
                let rock = player.get_resource(Resource::Rock);
                let sheep = player.get_resource(Resource::Sheep);

                let (buildable_edges, buildable_vertices) = player.get_buildable_spaces(&self);

                if wood >= 1 && brick >= 1 {
                    // road

                    for edge in &buildable_edges {
                        moves.push(CatanMove::BuildRoad(*edge));
                    }
                }

                if wood >= 1 && wheat >= 1 && brick >= 1 && sheep >= 1 {
                    // settlement

                    for vertex in &buildable_vertices {
                        moves.push(CatanMove::BuildSettlement(*vertex));
                    }
                }

                if rock >= 3 && wheat >= 2 {
                    // city

                    for vertex in &buildable_vertices {
                        moves.push(CatanMove::UpgradeSettlement(*vertex));
                    }
                }

                if rock >= 1 && sheep >= 1 {
                    // knight

                    for vertex in &buildable_vertices {
                        moves.push(CatanMove::BuildKnight(*vertex));
                    }

                    for knight in &player.knights {
                        moves.push(CatanMove::UpgradeKnight(*knight));
                    }
                }

                if wheat >= 1 {
                    // knight activation

                    for knight in &player.knights {
                        moves.push(CatanMove::ActivateKnight(*knight));
                    }
                }

                for (resource, num) in player.cards.iter() {
                    if *num >= 4 {
                        for other_resource in ALL_RESOURCES.iter() {
                            moves.push(CatanMove::Convert4x {
                                from: *resource,
                                to: *other_resource
                            });
                        }
                    }

                    if *num >= 2 {
                        if player.yellow_prog >= 3 && COMMODITIES.contains(resource) {
                            for other_resource in ALL_RESOURCES.iter() {
                                moves.push(CatanMove::Convert2x {
                                    from: *resource,
                                    to: *other_resource
                                });
                            }

                        } else if player.ports.contains(resource) {
                            for other_resource in ALL_RESOURCES.iter() {
                                moves.push(CatanMove::Convert2x {
                                    from: *resource,
                                    to: *other_resource
                                });
                            }

                        }
                    }
                }

                for card in player.dev_cards.iter() {
                    moves.push(CatanMove::PlayDevCard(*card));
                }

                // TODO

                moves
            },
            GameState::GameOver => {
                Vec::new()
            }
        }
    }
    fn make_move_mut(&mut self, m: &Self::Move) -> bool {
        match self.state {
            GameState::PreTurn => {
                if let &CatanMove::Roll(a, b, special) = m {
                    // TODO

                    true
                } else {
                    false
                }
            },
            GameState::Turn => {
                let mut ret = true;
                match m {
                    &CatanMove::EndTurn => {
                        self.state = GameState::PreTurn;
                        // TODO
                    },

                    // Building / Upgrading
                    &CatanMove::BuildRoad(EdgeID) => {

                    },
                    &CatanMove::BuildSettlement(VertexID) => {

                    },
                    &CatanMove::UpgradeSettlement(VertexID) => {

                    },
                    &CatanMove::BuildKnight(VertexID) => {

                    },
                    &CatanMove::UpgradeKnight(VertexID) => {

                    },
                    &CatanMove::ActivateKnight(VertexID) => {

                    },

                    // Development Cards
                    &CatanMove::UpgradeYellow => {

                    },
                    &CatanMove::UpgradeGreen => {

                    },
                    &CatanMove::UpgradeBlue => {

                    },
                    &CatanMove::PlayDevCard(DevCard) => {

                    },

                    // Trade
                    &CatanMove::Trade {to, ref give, ref take} => {

                    },
        
                    &CatanMove::Convert4x {from, to}  => {

                    },
                    &CatanMove::Convert2x {from, to}  => {

                    },

                    _ => { ret = false; }
                }
                ret
            },
            GameState::GameOver => {
                false
            }
        }
    }
    fn get_cur_player(&self) -> Self::Player {
        self.cur_player
    }
    fn get_winner(&self) -> Option<Self::Player> {
        if self.state == GameState::GameOver {
            Some(self.cur_player)
        } else {
            None
        }
    }
}