use std::hash::{ Hash, Hasher };
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;

pub enum MoveList<T> {
    Random(Vec<(T, f64)>),
    Choice(Vec<T>),
}

pub trait Game: Clone + Hash + Send {
	type Move: Hash + Eq + Clone + Send + Debug;
	type Player: Hash + Eq + Clone + Send + Debug;

	fn available_moves(&self) -> MoveList<Self::Move>;
    fn make_move(&mut self, m: &Self::Move);
    fn get_cur_player(&self) -> Self::Player;
    fn get_winner(&self) -> Option<Self::Player>;

    fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
    
	fn to_str(&self) -> String { String::new() }
}