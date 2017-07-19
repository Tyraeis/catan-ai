use std::collections::HashMap;
use std::cell::RefCell;

use cairo::Context;

#[allow(unused_imports)]
use lazy_static::*;

use ai::{ Game, MoveList };

use catan::hex_coord::*;
use catan::hex::*;
use catan::edge::*;
use catan::vertex::*;
use catan::player::*;
use catan::board_builder::*;

pub type EdgeID   = usize;
pub type VertexID = usize;
pub type PlayerID = usize;

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

const GREEN_CARDS: [(DevCard, f64); 10] = [
    (DevCard::Alchemist,        2.0 / 18.0),
    (DevCard::Crane,            2.0 / 18.0),
    (DevCard::Engineer,         2.0 / 18.0),
    (DevCard::Inventor,         1.0 / 18.0),
    (DevCard::Irrigation,       2.0 / 18.0),
    (DevCard::Medicine,         2.0 / 18.0),
    (DevCard::Mining,           2.0 / 18.0),
    (DevCard::Printer,          1.0 / 18.0),
    (DevCard::RoadBuilding,     2.0 / 18.0),
    (DevCard::Smith,            2.0 / 18.0),
];
const BLUE_CARDS: [(DevCard, f64); 9] = [
    (DevCard::Bishop,           2.0 / 18.0),
    (DevCard::Constitution,     1.0 / 18.0),
    (DevCard::Deserter,         2.0 / 18.0),
    (DevCard::Diplomat,         2.0 / 18.0),
    (DevCard::Intrigue,         2.0 / 18.0),
    (DevCard::Saboteur,         2.0 / 18.0),
    (DevCard::Spy,              3.0 / 18.0),
    (DevCard::Warlord,          2.0 / 18.0),
    (DevCard::Wedding,          2.0 / 18.0),
];
const YELLOW_CARDS: [(DevCard, f64); 6] = [
    (DevCard::CommercialHarbor, 2.0 / 18.0),
    (DevCard::MasterMerchant,   2.0 / 18.0),
    (DevCard::Merchant,         6.0 / 18.0),
    (DevCard::MerchantFleet,    2.0 / 18.0),
    (DevCard::ResourceMonopoly, 4.0 / 18.0),
    (DevCard::TradeMonopoly,    2.0 / 18.0),
];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    PreRoll,
    Roll,
    Turn,
    ResolvingDevCard(DevCard, u8),
    HandleSpecialRoll {
        special_die: SpecialRoll, 
        red_die: u8,
        player: PlayerID,
    },
    GameOver,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum CatanMove {
    NoAction,
    Series(Vec<CatanMove>),
    SetState(GameState),

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
        give: Vec<(Resource, u8)>,
        take: Vec<(Resource, u8)>,
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

#[derive(Clone)]
pub struct Catan {
    hexes:    HashMap<HexCoord, Hex>,
    edges:    HashMap<EdgeID,   Edge>,
    vertices: HashMap<VertexID, Vertex>,
    players:  HashMap<PlayerID, RefCell<Player>>,

    barbarian_pos: u8,

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

            barbarian_pos: 0,

            cur_player: 0,
            state: GameState::Roll,
        };

        for (pos, hex) in builder.hexes.drain() {
            println!("Hex {:?}:\n  Edges: {:?}\n  Vertices: {:?}",
                pos, hex.edges, hex.vertices
            );
            catan.hexes.insert(pos, Hex::new(hex));
        }

        for (id, edge) in builder.edges.drain() {
            println!("Edge {}:\n  Hexes: {:?}\n  Vertices: {:?}",
                id, edge.hexes, edge.vertices
            );
            catan.edges.insert(id, Edge::new(edge));
        }

        for (id, vertex) in builder.vertices.drain() {
            println!("Vertex {}:\n  Hexes: {:?}\n  Edges: {:?}",
                id, vertex.hexes, vertex.edges
            );
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

        for (id, edge) in self.edges.iter() {
            edge.draw(ctx, *id);
        }

        for (id, vertex) in self.vertices.iter() {
            vertex.draw(ctx, *id);
        }
    }
}

impl Game for Catan {
    type Move = CatanMove;
    type Player = PlayerID;

    fn available_moves(&self) -> MoveList<Self::Move> {
        let player = self.players.get(&self.cur_player).expect("Current player does not exist").borrow();
        match self.state {
            GameState::PreRoll => {
                if player.dev_cards.contains(&DevCard::Alchemist) {
                    vec![
                        CatanMove::PlayDevCard(DevCard::Alchemist),
                        CatanMove::NoAction,
                    ]
                } else {
                    MoveList::Random((*ALL_ROLLS).clone())
                }
            },
            GameState::Roll => {
                MoveList::Random((*ALL_ROLLS).clone())
            },
            GameState::HandleSpecialRoll { special_die, red_die, player } => {
                Vec::new()
            },
            GameState::Turn => {
                let mut moves = Vec::new();
                moves.push(CatanMove::EndTurn);

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
                    match card {
                        &DevCard::Alchemist => (),
                        _ => {
                            moves.push(CatanMove::PlayDevCard(*card));
                        }
                    }
                }

                // TODO

                moves
            },
            GameState::HandleSpecialRoll { special_die, red_die, player: player_id } => {
                let player = self.players.get(&player_id).unwrap().borrow_mut();
                match special_die {
                    SpecialRoll::Green =>  {
                        if player.green_prog > 0 && player.green_prog <= red_die + 1 {
                            
                        }
                    }
                    SpecialRoll::Blue =>   {
                        if player.blue_prog > 0 && player.blue_prog <= red_die + 1 {
                            
                        }
                    }
                    SpecialRoll::Yellow => {
                        if player.yellow_prog > 0 && player.yellow_prog <= red_die + 1 {
                            
                        }
                    }
                    _ => unreachable!()
                }
                Vec::new()
            },
            GameState::ResolvingDevCard(card, step) => {
                match card {
                    // Green
                    DevCard::Alchemist => {
                        (*ALL_ROLLS).clone()
                    },
                    DevCard::Crane => {
                        Vec::new()
                    },
                    DevCard::Engineer => {
                        Vec::new()
                    },
                    DevCard::Inventor => {
                        Vec::new()
                    },
                    DevCard::Irrigation => {
                        Vec::new()
                    },
                    DevCard::Medicine => {
                        Vec::new()
                    },
                    DevCard::Mining => {
                        Vec::new()
                    },
                    DevCard::Printer => {
                        Vec::new()
                    },
                    DevCard::RoadBuilding => {
                        let (buildable_edges, _) = player.get_buildable_spaces(&self);
                        let next_state = if step == 0 {
                            GameState::ResolvingDevCard(DevCard::RoadBuilding, 1)
                        } else {
                            GameState::Turn
                        };
                        buildable_edges.iter().map(|id| {
                            CatanMove::Series(vec![CatanMove::BuildRoad(*id), CatanMove::SetState(next_state)])
                        }).collect()
                    },
                    DevCard::Smith => {
                        Vec::new()
                    },

                    // Blue
                    DevCard::Bishop => {
                        Vec::new()
                    },
                    DevCard::Deserter => {
                        Vec::new()
                    },
                    DevCard::Diplomat => {
                        Vec::new()
                    },
                    DevCard::Intrigue => {
                        Vec::new()
                    },
                    DevCard::Saboteur => {
                        Vec::new()
                    },
                    DevCard::Spy => {
                        Vec::new()
                    },

                    // Yellow
                    DevCard::CommercialHarbor => {
                        Vec::new()
                    },
                    DevCard::MasterMerchant => {
                        Vec::new()
                    },
                    DevCard::Merchant => {
                        Vec::new()
                    },
                    DevCard::ResourceMonopoly => {
                        Vec::new()
                    },
                    DevCard::TradeMonopoly => {
                        Vec::new()
                    },
                    _ => panic!()
                }
            },
            GameState::GameOver => {
                Vec::new()
            }
        }
    }
    fn make_move_mut(&mut self, m: &Self::Move) -> bool {
        let mut ret = true;
        let mut state_change = None;
        match m {
            &CatanMove::NoAction => { },
            &CatanMove::SetState(state) => {
                state_change = Some(state)
            }
            &CatanMove::Series(ref actions) => {
                for mv in actions.iter() {
                    self.make_move_mut(mv);
                }
            }
            &CatanMove::Roll(white, red, special) => {
                let roll = white + red;

                for (hex_pos, hex) in self.hexes.iter() {
                    if hex.roll == roll {
                        if let HexType::Land(resource) = hex.static_data.typ {
                            for vertex_id in hex.static_data.vertices.iter() {
                                let vertex = self.vertices.get(vertex_id).unwrap();
                                if let Some((structure, player_id)) = vertex.structure {
                                    match structure {
                                        Structure::Settlement => {
                                            let mut player = self.players.get(&player_id).unwrap().borrow_mut();
                                            player.give_resource(resource, 1);
                                        },
                                        Structure::City | Structure::Metropolis => {
                                            let mut player = self.players.get(&player_id).unwrap().borrow_mut();
                                            player.give_resource(resource, 1);
                                            match resource {
                                                Resource::Wheat | Resource::Brick => {
                                                    player.give_resource(resource, 1);
                                                },
                                                Resource::Sheep => {
                                                    player.give_resource(Resource::Cloth, 1);
                                                },
                                                Resource::Wood => {
                                                    player.give_resource(Resource::Paper, 1);                                    
                                                },
                                                Resource::Rock => {
                                                    player.give_resource(Resource::Coins, 1);
                                                },
                                                _ => panic!()
                                            }
                                        },
                                        _ => { }
                                    }
                                }
                            }
                        }
                    }
                }

                if special == SpecialRoll::Barbarian {
                    self.barbarian_pos += 1;
                    if self.barbarian_pos >= 7 {
                        // TODO
                    }
                    state_change = Some(GameState::Turn)
                } else {
                    state_change = Some(GameState::HandleSpecialRoll {
                        special_die: special,
                        red_die: red,
                        player: self.cur_player
                    })
                }
            },
            &CatanMove::EndTurn => {
                // TODO
                state_change = Some(GameState::PreRoll)
            },

            // Building / Upgrading
            &CatanMove::BuildRoad(edge) => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(Resource::Wood, 1);
                player.consume_resource(Resource::Brick, 1);
                self.edges.get_mut(&edge).unwrap().road = Some(self.cur_player);
            },
            &CatanMove::BuildSettlement(vertex) => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(Resource::Wood, 1);
                player.consume_resource(Resource::Brick, 1);
                player.consume_resource(Resource::Sheep, 1);
                player.consume_resource(Resource::Wheat, 1);
                self.vertices.get_mut(&vertex).unwrap().structure = Some((Structure::Settlement, self.cur_player));
            },
            &CatanMove::UpgradeSettlement(vertex) => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(Resource::Wheat, 2);
                player.consume_resource(Resource::Rock, 3);
                self.vertices.get_mut(&vertex).unwrap().structure = Some((Structure::City, self.cur_player));
            },
            &CatanMove::BuildKnight(vertex) => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(Resource::Sheep, 1);
                player.consume_resource(Resource::Rock, 1);
                self.vertices.get_mut(&vertex).unwrap().structure = Some((Structure::KnightT1(false), self.cur_player));
            },
            &CatanMove::UpgradeKnight(vertex) => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(Resource::Sheep, 1);
                player.consume_resource(Resource::Rock, 1);
                let vertex = self.vertices.get_mut(&vertex).unwrap();
                vertex.structure = Some((match vertex.structure {
                    Some((Structure::KnightT1(active), _)) => Structure::KnightT2(active),
                    Some((Structure::KnightT2(active), _)) => Structure::KnightT3(active),
                    _ => panic!()
                }, self.cur_player));
            },
            &CatanMove::ActivateKnight(vertex) => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(Resource::Wheat, 1);
                let vertex = self.vertices.get_mut(&vertex).unwrap();
                vertex.structure = Some((match vertex.structure.unwrap().0 {
                    Structure::KnightT1(_) => Structure::KnightT1(true),
                    Structure::KnightT2(_) => Structure::KnightT2(true),
                    Structure::KnightT3(_) => Structure::KnightT3(true),
                    _ => panic!()
                }, self.cur_player));
            },

            // Development Cards
            &CatanMove::UpgradeYellow => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(Resource::Cloth, 1);
                player.yellow_prog += 1;
                // TODO
            },
            &CatanMove::UpgradeGreen => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(Resource::Paper, 1);
                player.green_prog += 1;
                // TODO
            },
            &CatanMove::UpgradeBlue => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(Resource::Coins, 1);
                player.blue_prog += 1;
                // TODO
            },
            &CatanMove::PlayDevCard(card) => {
                match card {
                    // VP cards should have been played immediately
                    DevCard::Printer | DevCard::Constitution => { ret = false },

                    // Green
                    DevCard::Irrigation => {

                    },
                    DevCard::Mining => {

                    },

                    // Blue
                    DevCard::Warlord => {

                    },
                    DevCard::Wedding => {

                    },

                    // Yellow
                    DevCard::MerchantFleet => {

                    },

                    // 
                    _ => {
                        // Crane, Engineer, Inventory, Medicine, RoadBuilding, Smith
                        // Bishop, Deserter, Diplomat, Intrigue, Saboteur, Spy
                        // CommercialHarbor, MasterMerchant, Merchant, ResourceMonopoly, TradeMonopoly
                    },
                }
            },

            // Trade
            &CatanMove::Trade {to, ref give, ref take} => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                let mut other_player = self.players.get(&to).unwrap().borrow_mut();
                for &(res, num) in give {
                    player.consume_resource(res, num);
                    other_player.give_resource(res, num);
                }
                for &(res, num) in take {
                    player.give_resource(res, num);
                    other_player.consume_resource(res, num);
                }
            },

            &CatanMove::Convert4x {from, to}  => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(from, 4);
                player.give_resource(to, 1);
            },
            &CatanMove::Convert2x {from, to}  => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(from, 2);
                player.give_resource(to, 1);
            },

            //_ => { ret = false; self.state }
        }
        if let Some(state) = state_change {
            self.state = state;
        }
        ret
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