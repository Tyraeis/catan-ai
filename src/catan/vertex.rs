use catan::PlayerID;

#[derive(Clone)]
pub enum Structure {
    Settlement, City, Metropolis,
    KnightT1, KnightT2, KnightT3,
}

#[derive(Clone)]
pub struct Vertex {
    pub structure: Option<(Structure, PlayerID)>,
}

impl Vertex {
    pub fn new() -> Self {
        Vertex {
            structure: None
        }
    }
}