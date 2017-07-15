use std::sync::Arc;

pub struct PlayerStatic {
    color: [f64; 3],
}

#[derive(Clone)]
pub struct Player {
    static_data: Arc<PlayerStatic>,
}