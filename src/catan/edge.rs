use catan::PlayerID;

#[derive(Clone)]
pub struct Edge {
    pub road: Option<PlayerID>,
}
impl Edge {
    pub fn new() -> Self {
        Edge {
            road: None
        }
    }
}