use std::hash::Hash;

pub trait Game: Clone + Send {
	type Move: Hash + Eq + Clone + Send;
	type Player: Hash + Eq + Clone + Send;

	fn available_moves(&self) -> Vec<Self::Move>;
    fn make_move_mut(&mut self, m: &Self::Move) -> bool;
    fn get_cur_player(&self) -> Self::Player;
    fn get_winner(&self) -> Option<Self::Player>;

	fn make_move(&self, m: &Self::Move) -> Option<Box<Self>> {
        let mut c = self.clone();
        if c.make_move_mut(m) {
            Some(Box::new(c))
        } else {
            None
        }
    }
	fn to_str(&self) -> String { String::new() }
}