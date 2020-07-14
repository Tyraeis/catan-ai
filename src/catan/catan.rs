use std::collections::HashMap;
use std::cell::RefCell;
use std::fmt::Debug;
use std::hash::*;
use std::f64::consts::PI;

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Resource {
    Wheat, Sheep, Brick, Wood, Rock
}
const ALL_RESOURCES: [Resource; 5] = 
    [ Resource::Wheat, Resource::Sheep, Resource::Brick, Resource::Wood, Resource::Rock ];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DevCard {
    Soldier,
    YearOfPlenty,
    Monopoly,
    RoadBuilding,
    VictoryPoint, // includes Chapel, Governor's House, Library, Market, and University of Catan
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameState {
    SetupSettlements { first_player: PlayerID },
    SetupSettlementRoad { first_player: PlayerID, settlement: VertexID },
    SetupCities { first_player: PlayerID },
    SetupCityRoad { first_player: PlayerID, city: VertexID },
    Roll,
    Turn,
    DrawingDevCard,
    ResolvingDevCard(DevCard, u8),
    MovingRobber,
    StealingCards(PlayerID),
    GameOver,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum CatanMove {
    Series(Vec<CatanMove>),
    SetState(GameState),

    // Setup
    PlaceSettlement(VertexID),
    PlaceCity(VertexID),
    PlaceRoad(EdgeID),

    // Special
    Roll(u8),
    EndTurn,
    MoveRobber(HexCoord, PlayerID),
    Steal(Resource, PlayerID),

    // Building / Upgrading
    BuildRoad(EdgeID),
    BuildSettlement(VertexID),
    BuildCity(VertexID),

    // Development Cards
    BuyDevCard,
    DrawDevCard(DevCard),
    PlayDevCard(DevCard),
    ReceiveMonopoly(Resource),
    ReceiveYearOfPlenty(Resource, Resource),

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
    Convert3x {
        from: Resource,
        to: Resource,
    },
    Convert2x {
        from: Resource,
        to: Resource,
    },
}

lazy_static! {
    static ref ROLLS: Vec<(CatanMove, f64)> = vec![
        (CatanMove::Roll( 2), 1.0 / 36.0),
        (CatanMove::Roll( 3), 2.0 / 36.0),
        (CatanMove::Roll( 4), 3.0 / 36.0),
        (CatanMove::Roll( 5), 4.0 / 36.0),
        (CatanMove::Roll( 6), 5.0 / 36.0),
        (CatanMove::Roll( 7), 6.0 / 36.0),
        (CatanMove::Roll( 8), 5.0 / 36.0),
        (CatanMove::Roll( 9), 4.0 / 36.0),
        (CatanMove::Roll(10), 3.0 / 36.0),
        (CatanMove::Roll(11), 2.0 / 36.0),
        (CatanMove::Roll(12), 1.0 / 36.0),
    ];
    static ref DEVELOPMENT_CARDS: Vec<(CatanMove, f64)> = vec![
        (CatanMove::DrawDevCard(DevCard::Soldier),      14.0 / 25.0),
        (CatanMove::DrawDevCard(DevCard::YearOfPlenty),  2.0 / 25.0),
        (CatanMove::DrawDevCard(DevCard::Monopoly),      2.0 / 25.0),
        (CatanMove::DrawDevCard(DevCard::RoadBuilding),  2.0 / 25.0),
        (CatanMove::DrawDevCard(DevCard::VictoryPoint),  5.0 / 25.0),
    ];
}


#[derive(Clone)]
pub struct Catan {
    pub(in super) hexes:    HashMap<HexCoord, Hex>,
    pub(in super) edges:    HashMap<EdgeID,   Edge>,
    pub(in super) vertices: HashMap<VertexID, Vertex>,
    pub(in super) players:  HashMap<PlayerID, RefCell<Player>>,

    cur_player: PlayerID,
    state: GameState,

    robber_pos: HexCoord,
    largest_army_owner: Option<PlayerID>,
    longest_road: Option<(PlayerID, u8)>,

    new_dev_cards: Vec<DevCard>, // cards the player has drawn but can't use yet
}

impl Catan {
    pub fn new(mut builder: BoardBuilder) -> Self {
        let mut robber_pos = HexCoord::new(0, 0);
        for (coord, hex) in builder.hexes.iter() {
            if hex.typ == HexType::Desert {
                robber_pos = *coord;
                break;
            }
        }

        let mut catan = Catan {
            hexes:    HashMap::new(),
            edges:    HashMap::new(),
            vertices: HashMap::new(),
            players:  HashMap::new(),

            cur_player: builder.first_player,
            state: GameState::SetupSettlements { first_player: builder.first_player },

            robber_pos,
            largest_army_owner: None,
            longest_road: None,

            new_dev_cards: Vec::new(),
        };

        for (pos, hex) in builder.hexes.drain() {
            /* println!("Hex {:?}:\n  Edges: {:?}\n  Vertices: {:?}",
                pos, hex.edges, hex.vertices
            ); */
            catan.hexes.insert(pos, Hex::new(hex));
        }

        for (id, edge) in builder.edges.drain() {
            /* println!("Edge {}:\n  Hexes: {:?}\n  Vertices: {:?}",
                id, edge.hexes, edge.vertices
            ); */
            catan.edges.insert(id, Edge::new(edge));
        }

        for (id, vertex) in builder.vertices.drain() {
            /* println!("Vertex {}:\n  Hexes: {:?}\n  Edges: {:?}",
                id, vertex.hexes, vertex.edges
            ); */
            catan.vertices.insert(id, Vertex::new(vertex));
        }

        for (id, player) in builder.players.drain() {
            catan.players.insert(id, RefCell::new(Player::new(player)));
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

    fn is_water(&self, hex_coord: HexCoord) -> bool {
        let hex = self.hexes.get(&hex_coord).unwrap();
        match hex.static_data.typ {
            HexType::Water | HexType::Port2to1(_, _) | HexType::Port3to1(_) => true,
            HexType::Desert | HexType::Land(_) => false,
        }
    }

    fn is_bridge(&self, edge_id: EdgeID) -> bool {
        let edge = self.edges.get(&edge_id).unwrap();
        match (edge.static_data.hexes[0], edge.static_data.hexes[1]) {
            (None, None) => true,
            (None, Some(hex)) if self.is_water(hex) => true,
            (Some(hex), None) if self.is_water(hex) => true,
            (Some(hex1), Some(hex2)) if self.is_water(hex1) && self.is_water(hex2) => true,
            _ => false
        }
    }

    fn settlement_location_is_valid(&self, vertex_id: VertexID) -> bool {
        let vertex = self.vertices.get(&vertex_id).unwrap();

        let mut has_land = false;
        for hex_opt in vertex.static_data.hexes.iter() {
            if let &Some(hex) = hex_opt {
                if !self.is_water(hex) {
                    has_land = true;
                    break;
                }
            }
        }
        if !has_land { return false; }

        for edge_id_opt in vertex.static_data.edges.iter() {
            if let &Some(edge_id) = edge_id_opt {
                if self.is_bridge(edge_id) {
                    // settlements can be directly ajacent if they are seperated by a bridge
                    continue;
                }

                let edge = self.edges.get(&edge_id).unwrap();

                let vertex_id_2 = if edge.static_data.vertices[0] == vertex_id {
                    edge.static_data.vertices[1]
                } else {
                    edge.static_data.vertices[0]
                };
                let vertex_2 = self.vertices.get(&vertex_id_2).unwrap();
                if vertex_2.structure.is_some() {
                    return false;
                }
            }
        }
        true
    }

    fn check_for_port(&self, vertex_id: VertexID, player_id: PlayerID) {
        let vertex = self.vertices.get(&vertex_id).unwrap();
        let mut player = self.players.get(&player_id).unwrap().borrow_mut();
        for hex_opt in vertex.static_data.hexes.iter() {
            if let &Some(hex_id) = hex_opt {
                let hex = self.hexes.get(&hex_id).unwrap();
                if let HexType::Port2to1(resource, side) = hex.static_data.typ {
                    if hex.static_data.vertices[side as usize] == vertex_id || hex.static_data.vertices[(side as usize + 1) % 6] == vertex_id {
                        player.ports.insert(resource);
                    }
                } else if let HexType::Port3to1(side) = hex.static_data.typ {
                    if hex.static_data.vertices[side as usize] == vertex_id || hex.static_data.vertices[(side as usize + 1) % 6] == vertex_id {
                        player.has_3to1_port = true;
                    }
                }
            }
        }
    }

    fn check_longest_road(&mut self, new_road: EdgeID) -> Option<GameState> {
        let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
        let road_len = player.get_longest_road(self, new_road);

        if road_len >= 5 {
            if let Some((old_owner, old_road_len)) = self.longest_road {
                if old_owner != self.cur_player {
                    self.longest_road = Some((self.cur_player, road_len));
                    self.players.get(&old_owner).unwrap().borrow_mut().victory_points -= 2;

                    player.victory_points += 2;
                    if player.victory_points >= 10 {
                        return Some(GameState::GameOver);
                    }
                }
            } else {
                self.longest_road = Some((self.cur_player, road_len));

                player.victory_points += 2;
                if player.victory_points >= 10 {
                    return Some(GameState::GameOver);
                }
            }
        }
        None
    }

    pub fn draw(&self, ctx: &Context, w: f64, h: f64) {
        ctx.translate(w / 2.0, h / 2.0);

        for (pos, hex) in self.hexes.iter() {
            hex.draw(ctx, pos);
        }

        for (id, edge) in self.edges.iter() {
            edge.draw(ctx, self, *id);
        }

        for (id, vertex) in self.vertices.iter() {
            vertex.draw(ctx, self, *id);
        }

        let (offx, offy) = self.robber_pos.to_point();
        ctx.arc(offx + HEX_SCALE / 3.0, offy + HEX_SCALE / 3.0, HEX_SCALE / 5.0, 0.0, 2.0*PI);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.fill();
    }

    pub fn get_player_color(&self, player: PlayerID) -> [f64; 3] {
        self.players.get(&player).unwrap().borrow().static_data.color
    }
}

impl Hash for Catan {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        /* for hex in self.hexes.values() {
            hex.hash(state);
        } */
        for edge in self.edges.values() {
            edge.hash(state);
        }
        for vertex in self.vertices.values() {
            vertex.hash(state);
        }
        for player in self.players.values() {
            player.borrow().hash(state);
        }

        self.cur_player.hash(state);
        self.state.hash(state);
        self.robber_pos.hash(state);
        
        for card in self.new_dev_cards.iter() {
            card.hash(state);
        }
    }
}

impl Game for Catan {
    type Move = CatanMove;
    type Player = PlayerID;

    fn available_moves(&self) -> MoveList<Self::Move> {
        let player = self.players.get(&self.cur_player).expect("Current player does not exist").borrow();
        match self.state {
            GameState::SetupSettlements { first_player } => {
                let mut mvs = Vec::new();
                for (vertex_id, vertex) in self.vertices.iter() {
                    if vertex.structure.is_none() && self.settlement_location_is_valid(*vertex_id) {
                        mvs.push(CatanMove::PlaceSettlement(*vertex_id));
                    }
                }
                MoveList::Choice(mvs)
            }
            GameState::SetupSettlementRoad { first_player, settlement } => {
                let vertex = self.vertices.get(&settlement).unwrap();
                MoveList::Choice(vertex.static_data.edges.iter().filter_map(|edge_opt| {
                    edge_opt.and_then(|edge_id| {
                        let edge = self.edges.get(&edge_id).unwrap();
                        if edge.road.is_none() {
                            Some(CatanMove::PlaceRoad(edge_id))
                        } else {
                            None
                        }
                    })
                }).collect())
            }
            GameState::SetupCities { first_player } => {
                let mut mvs = Vec::new();
                for (vertex_id, vertex) in self.vertices.iter() {
                    if vertex.structure.is_none() && self.settlement_location_is_valid(*vertex_id) {
                        mvs.push(CatanMove::PlaceCity(*vertex_id));
                    }
                }
                MoveList::Choice(mvs)
            }
            GameState::SetupCityRoad { first_player, city } => {
                let vertex = self.vertices.get(&city).unwrap();
                MoveList::Choice(vertex.static_data.edges.iter().filter_map(|edge_opt| {
                    edge_opt.and_then(|edge_id| {
                        let edge = self.edges.get(&edge_id).unwrap();
                        if edge.road.is_none() {
                            Some(CatanMove::PlaceRoad(edge_id))
                        } else {
                            None
                        }
                    })
                }).collect())
            }
            GameState::Roll => {
                MoveList::Random((*ROLLS).clone())
            }
            GameState::MovingRobber => {
                let mut mvs = Vec::new();
                for (pos, hex) in self.hexes.iter() {
                    if hex.static_data.typ != HexType::Water {
                        for vertex_id in hex.static_data.vertices.iter() {
                            let vertex = self.vertices.get(&vertex_id).unwrap();

                            if let Some((structure, player)) = vertex.structure {
                                if player != self.cur_player {
                                    match structure {
                                        Structure::City | Structure::Settlement => {
                                            mvs.push(CatanMove::MoveRobber(*pos, player))
                                        }
                                        _ => { }
                                    }
                                }
                            }
                        }
                    }
                }
                MoveList::Choice(mvs)
            }
            GameState::StealingCards(from_id) => {
                let from = self.players.get(&from_id).unwrap().borrow();
                let mvs: Vec<(CatanMove, f64)> = from.cards.iter().filter_map(|pair| {
                    let (resource, num) = pair;
                    if *num > 0 {
                        Some((CatanMove::Steal(*resource, from_id), *num as f64))
                    } else {
                        None
                    }
                }).collect();
                if mvs.len() > 0 {
                    MoveList::Random(mvs)
                } else {
                    MoveList::Choice(vec![CatanMove::SetState(GameState::Turn)])
                }
            }
            GameState::Turn => {
                let mut moves = Vec::new();
                moves.push(CatanMove::EndTurn);

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
                        if self.settlement_location_is_valid(*vertex) {
                            moves.push(CatanMove::BuildSettlement(*vertex));
                        }
                    }
                }

                if rock >= 3 && wheat >= 2 {
                    // city

                    for vertex in player.settlements.iter() {
                        moves.push(CatanMove::BuildCity(*vertex));
                    }
                }

                if sheep >= 1 && wheat >=1 && rock >= 1 {
                    // development card

                    moves.push(CatanMove::BuyDevCard);
                }

                for (resource, num) in player.cards.iter() {
                    if *num >= 4 && !player.has_3to1_port {
                        for other_resource in ALL_RESOURCES.iter() {
                            if *other_resource != *resource {
                                moves.push(CatanMove::Convert4x {
                                    from: *resource,
                                    to: *other_resource
                                });
                            }
                        }
                    }

                    if *num >= 3 && player.has_3to1_port {
                        for other_resource in ALL_RESOURCES.iter() {
                            if *other_resource != *resource {
                                moves.push(CatanMove::Convert3x {
                                    from: *resource,
                                    to: *other_resource
                                });
                            }
                        }
                    }

                    if *num >= 2 && player.ports.contains(resource) {
                        for other_resource in ALL_RESOURCES.iter() {
                            if *other_resource != *resource {
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

                MoveList::Choice(moves)
            }
            GameState::DrawingDevCard => {
                MoveList::Random((*DEVELOPMENT_CARDS).clone())
            }
            GameState::ResolvingDevCard(card, step) => {
                match card {
                    DevCard::YearOfPlenty => {
                        let mut mvs = Vec::new();
                        for (ind, a) in ALL_RESOURCES.iter().enumerate() {
                            for b in ALL_RESOURCES.iter().skip(ind) {
                                mvs.push(CatanMove::ReceiveYearOfPlenty(*a, *b));
                            }
                        }
                        MoveList::Choice(mvs)
                    }
                    DevCard::Monopoly => {
                        MoveList::Choice(ALL_RESOURCES.iter().map(|res| {
                            CatanMove::ReceiveMonopoly(*res)
                        }).collect())
                    }
                    DevCard::RoadBuilding => {
                        let (buildable_edges, _) = player.get_buildable_spaces(&self);
                        let next_state = if step == 0 {
                            GameState::ResolvingDevCard(DevCard::RoadBuilding, 1)
                        } else {
                            GameState::Turn
                        };
                        MoveList::Choice(buildable_edges.iter().map(|id| {
                            CatanMove::Series(vec![CatanMove::PlaceRoad(*id), CatanMove::SetState(next_state)])
                        }).collect())
                    }
                    _ => panic!()
                }
            }
            GameState::GameOver => {
                MoveList::Choice(Vec::new())
            }
        }
    }
    fn make_move(&mut self, m: &Self::Move) {
        let mut state_change = None;
        match m {
            &CatanMove::SetState(state) => {
                state_change = Some(state)
            }
            &CatanMove::Series(ref actions) => {
                for mv in actions.iter() {
                    self.make_move(mv);
                }
            }
            // Setup
            &CatanMove::PlaceSettlement(vertex_id) => {
                self.check_for_port(vertex_id, self.cur_player);
                let mut vertex = self.vertices.get_mut(&vertex_id).unwrap();
                vertex.structure = Some((Structure::Settlement, self.cur_player));
                if let GameState::SetupSettlements { first_player } = self.state {
                    state_change = Some(GameState::SetupSettlementRoad { first_player, settlement: vertex_id });
                }
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.settlements.insert(vertex_id);
            }
            &CatanMove::PlaceCity(vertex_id) => {
                self.check_for_port(vertex_id, self.cur_player);
                let mut vertex = self.vertices.get_mut(&vertex_id).unwrap();
                vertex.structure = Some((Structure::City, self.cur_player));
                if let GameState::SetupCities { first_player } = self.state {
                    state_change = Some(GameState::SetupCityRoad { first_player, city: vertex_id });
                }
            }
            &CatanMove::PlaceRoad(edge_id) => {
                {
                    let mut edge = self.edges.get_mut(&edge_id).unwrap();
                    let mut player = self.players.get_mut(&self.cur_player).unwrap().get_mut();
                    edge.road = Some(self.cur_player);
                    player.roads.insert(edge_id);

                    if let GameState::SetupSettlementRoad { first_player, settlement } = self.state {
                        if player.static_data.next_player == first_player {
                            state_change = Some(GameState::SetupCities { first_player });
                        } else {
                            self.cur_player = player.static_data.next_player;
                            state_change = Some(GameState::SetupSettlements { first_player });
                        }
                    } else if let GameState::SetupCityRoad { first_player, city } = self.state {
                        if self.cur_player == first_player {
                            state_change = Some(GameState::Roll);
                        } else {
                            self.cur_player = player.static_data.prev_player;
                            state_change = Some(GameState::SetupCities { first_player });
                        }
                    }
                }

                match self.state {
                    GameState::SetupCityRoad { .. } | GameState::SetupSettlementRoad { .. } => { }
                    _ => {
                        state_change = self.check_longest_road(edge_id).or(state_change);
                    }
                }
            }
            &CatanMove::Roll(roll) => {
                if roll == 7 {
                    state_change = Some(GameState::MovingRobber)
                } else {

                    for (hex_pos, hex) in self.hexes.iter() {
                        if hex.static_data.roll == roll && *hex_pos != self.robber_pos {
                            if let HexType::Land(resource) = hex.static_data.typ {
                                for vertex_id in hex.static_data.vertices.iter() {
                                    let vertex = self.vertices.get(vertex_id).unwrap();
                                    if let Some((structure, player_id)) = vertex.structure {
                                        match structure {
                                            Structure::Settlement => {
                                                let mut player = self.players.get(&player_id).unwrap().borrow_mut();
                                                player.give_resource(resource, 1);
                                            },
                                            Structure::City => {
                                                let mut player = self.players.get(&player_id).unwrap().borrow_mut();
                                                player.give_resource(resource, 2);
                                            },
                                            _ => { }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    state_change = Some(GameState::Turn);
                }
            }
            &CatanMove::MoveRobber(hex_pos, player_id) => {
                self.robber_pos = hex_pos;

                state_change = Some(GameState::StealingCards(player_id));
            }
            &CatanMove::Steal(resource, from_id) => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                let mut from = self.players.get(&from_id).unwrap().borrow_mut();

                if from.get_resource(resource) == 0 {
                    println!("!!!");
                } else {
                    from.consume_resource(resource, 1);
                }

                player.give_resource(resource, 1);

                state_change = Some(GameState::Turn);
            }
            &CatanMove::EndTurn => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                for card in self.new_dev_cards.drain(0..) {
                    player.dev_cards.push(card);
                }
                self.cur_player = player.static_data.next_player;
                state_change = Some(GameState::Roll);
            }

            // Building / Upgrading
            &CatanMove::BuildRoad(edge) => {
                {
                    let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                    player.consume_resource(Resource::Wood, 1);
                    player.consume_resource(Resource::Brick, 1);
                    self.edges.get_mut(&edge).unwrap().road = Some(self.cur_player);
                    player.roads.insert(edge);
                }

                state_change = self.check_longest_road(edge);
            }
            &CatanMove::BuildSettlement(vertex) => {
                self.check_for_port(vertex, self.cur_player);
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(Resource::Wood, 1);
                player.consume_resource(Resource::Brick, 1);
                player.consume_resource(Resource::Sheep, 1);
                player.consume_resource(Resource::Wheat, 1);
                self.vertices.get_mut(&vertex).unwrap().structure = Some((Structure::Settlement, self.cur_player));
                player.settlements.insert(vertex);

                player.victory_points += 1;
                if player.victory_points >= 10 {
                    state_change = Some(GameState::GameOver);
                }
            }
            &CatanMove::BuildCity(vertex) => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(Resource::Wheat, 2);
                player.consume_resource(Resource::Rock, 3);
                self.vertices.get_mut(&vertex).unwrap().structure = Some((Structure::City, self.cur_player));
                player.settlements.remove(&vertex);

                player.victory_points += 1;
                if player.victory_points >= 10 {
                    state_change = Some(GameState::GameOver);
                }
            }

            // Development Cards
            &CatanMove::BuyDevCard => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(Resource::Sheep, 1);
                player.consume_resource(Resource::Wheat, 1);
                player.consume_resource(Resource::Rock, 1);
                state_change = Some(GameState::DrawingDevCard);
            }
            &CatanMove::DrawDevCard(card) => {
                if card == DevCard::VictoryPoint {
                    let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                    player.victory_points += 1;
                    if player.victory_points >= 10 {
                        state_change = Some(GameState::GameOver);
                    }
                } else {
                    self.new_dev_cards.push(card);
                }
                state_change = Some(GameState::Turn);
            }
            &CatanMove::PlayDevCard(card) => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.dev_cards.remove_item(&card);
                match card {
                    DevCard::Soldier => {
                        state_change = Some(GameState::MovingRobber);
                        player.soldiers += 1;
                        if let Some(largest_army_owner) = self.largest_army_owner {
                            if player.soldiers >= 3 && largest_army_owner != self.cur_player {
                                let mut other_player = self.players.get(&largest_army_owner).unwrap().borrow_mut();
                                if player.soldiers > other_player.soldiers {
                                    other_player.victory_points -= 2;
                                    player.victory_points += 2;
                                    self.largest_army_owner = Some(self.cur_player);
                                }
                            }
                        } else if player.soldiers >= 3 {
                            player.victory_points += 2;
                            self.largest_army_owner = Some(self.cur_player);
                        }
                        
                    }
                    DevCard::YearOfPlenty | DevCard::Monopoly | DevCard::RoadBuilding => {
                        state_change = Some(GameState::ResolvingDevCard(card, 0));
                    }
                    // VP cards should have been played immediately
                    DevCard::VictoryPoint => { unreachable!() }
                }
            }
            &CatanMove::ReceiveMonopoly(resource) => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                for (player_id, player_cell) in self.players.iter() {
                    if *player_id != self.cur_player {
                        let mut other_player = player_cell.borrow_mut();
                        if let Some(count) = other_player.cards.remove(&resource) {
                            player.give_resource(resource, count);
                        }
                    }
                }
                state_change = Some(GameState::Turn);
            }
            &CatanMove::ReceiveYearOfPlenty(res1, res2) => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.give_resource(res1, 1);
                player.give_resource(res2, 1);
                state_change = Some(GameState::Turn);
            }

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
            }

            &CatanMove::Convert4x {from, to}  => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(from, 4);
                player.give_resource(to, 1);
            }
            &CatanMove::Convert3x {from, to}  => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(from, 3);
                player.give_resource(to, 1);
            }
            &CatanMove::Convert2x {from, to}  => {
                let mut player = self.players.get(&self.cur_player).unwrap().borrow_mut();
                player.consume_resource(from, 2);
                player.give_resource(to, 1);
            }
        }
        if let Some(state) = state_change {
            self.state = state;
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