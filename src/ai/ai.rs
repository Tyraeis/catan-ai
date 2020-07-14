use std::collections::{ HashMap, HashSet };
use std::cell::{ RefCell, Ref, RefMut };
use std::thread;
use std::sync::mpsc::{ channel, Sender, Receiver, TryRecvError };
use std::time::{ Duration, Instant };

use super::Game;
use super::tree::*;
use ai::montecarlo::montecarlo;
use ai::sim_thread_pool::SimThreadPool;

use rand::{ thread_rng, Rng };

#[derive(Debug)]
pub enum Request<G: Game> {
    Info,

    MakeMove(G::Move),
    //MakeRandomMove,
}

#[derive(Debug)]
pub enum Response<G: Game> {
    Info {
        best_move: Option<G::Move>,
        possible_moves: Vec<G::Move>,
        is_random: bool,
        confidence: f64,
        total_sims: u64,
        time_elapsed: Duration,
    },

    Ok,
}

pub(in super) type NodeID = u64;

pub(in super) struct NodeList<G: Game> {
    pub nodes: HashMap<NodeID, RefCell<MoveTreeNode<G>>>
}
impl<G: Game> NodeList<G> {
    pub fn new() -> Self {
        NodeList {
            nodes: HashMap::new()
        }
    }

    pub fn add(&mut self, node: MoveTreeNode<G>) -> NodeID {
        let id = node.game.get_hash();
        self.nodes.insert(id, RefCell::new(node));

        id
    }

    pub fn get(&self, node: NodeID) -> Ref<MoveTreeNode<G>> {
        self.nodes[&node].borrow()
    }

    pub fn get_mut(&self, node: NodeID) -> RefMut<MoveTreeNode<G>> {
        self.nodes[&node].borrow_mut()
    }

    pub fn collect_garbage(&mut self, root: NodeID) {
        let mut referenced = HashSet::new();
        let mut open_set = Vec::new();
        open_set.push(root);

        while let Some(node_id) = open_set.pop() {
            let node = self.nodes.get(&node_id).unwrap().borrow();
            for child_id in node.children.values() {
                open_set.push(*child_id);
            }
            referenced.insert(node_id);
        }

        let all_keys: HashSet<u64> = self.nodes.keys().cloned().collect();
        for node_id in all_keys.difference(&referenced) {
            self.nodes.remove(node_id);
        }
    }

    /* pub fn drop_node(&mut self, old: NodeID, except: NodeID) {
        // Note: it is up to the caller to ensure that the node being removed is not referenced
        //   by any other node (i.e. its parent)
        if let Some(node) = self.nodes.remove(&node_id) {
            for child_id in node.borrow().children.values() {
                if *child_id != except {
                    self.drop_node(*child_id, except);
                }
            }
        } 
    } */

    pub fn take(&mut self, node_id: NodeID) -> MoveTreeNode<G> {
        self.nodes.remove(&node_id).unwrap().into_inner()
    }

    pub fn restore(&mut self, node_id: NodeID, node: MoveTreeNode<G>) {
        self.nodes.insert(node_id, RefCell::new(node));
    }
}

fn best_move<G: Game>(nodes: &NodeList<G>, root: NodeID) -> Option<G::Move> {
    let rt = nodes.get(root);
    let opt_mv = rt.children.iter().map(|e| {
        let (k, child_id) = e;
        let child = nodes.get(*child_id);

        //let c = nodes.get(child);

        (child.simulations, k)
    }).max_by(|x, y| x.0.cmp(&y.0));

    opt_mv.map(|i| i.1.clone())
}

pub struct Ai<G: Game> {
    to_thread: Sender<Request<G>>,
    from_thread: Receiver<Response<G>>,
}

impl<G> Ai<G> where G: Game + 'static {
    pub fn new(game: G) -> Self {
        let (to_thread, from_outside) = channel();
        let (to_outside, from_thread) = channel();

        thread::spawn(move || {
            let start_time = Instant::now();

            let mut rand = thread_rng();

            let mut num_sims: u64 = 0;
            let mut nodes = NodeList::new();
            let mut root = nodes.add(MoveTreeNode::new(game));
            let thread_pool = SimThreadPool::new();

            loop {
                //println!("#nodes: {}", nodes.len());
                while let Ok(msg) = from_outside.try_recv() {
                    match msg {
                        Request::Info => {
                            let rt = nodes.get(root);
                            let is_random = rt.weights.is_some();
                            let possible_moves: Vec<G::Move> = rt.children.keys().cloned().collect();

                            let mv = if is_random {
                                None   
                            } else {
                                best_move(&nodes, root)
                            };

                            let confidence = mv.clone()
                                .and_then(|m| {
                                    rt.children.get(&m)
                                })
                                .map(|c_id| {
                                    let c = nodes.get(*c_id);
                                    c.wins as f64 / c.games as f64
                                })
                                .unwrap_or(0.0);

                            let stats = Response::Info {
                                best_move: mv,
                                is_random,
                                possible_moves,
                                confidence: confidence,
                                total_sims: num_sims,
                                time_elapsed: start_time.elapsed(),
                            };

                            to_outside.send(stats).expect("Send failed (Info)");
                        },

                        Request::MakeMove(mv) => {
                            println!("{:?}", mv);
                            root = {
                                let rt = nodes.take(root);
                                let new_root_id = {
                                    let id = rt.children.get(&mv)
                                        .map(|id| *id)
                                        .unwrap_or_else(|| {
                                            let mut game = rt.game.clone();
                                            game.make_move(&mv);
                                            nodes.add(MoveTreeNode::new(game))
                                        });
                                    id
                                };

                                nodes.collect_garbage(new_root_id);

                                let mut new_rt = nodes.get_mut(new_root_id);

                                new_root_id
                            };

                            to_outside.send(Response::Ok).expect("Send failed (Ok)");
                        },

                        /*Request::MakeRandomMove => {
                            let rt = nodes.take(root);
                            let mvs: Vec<G::Move> = rt.children.keys().cloned().collect();

                            if let Some(mv) = rand.choose(&*mvs) {
                                println!("{:?}", mv);
                                root = {
                                    let new_root_id = {
                                        let id = rt.children.get(&mv)
                                            .map(|id| *id)
                                            .unwrap_or_else(|| {
                                                let mut game = rt.game.clone();
                                                game.make_move(&mv);
                                                nodes.add(MoveTreeNode::new_root(game))
                                            });
                                        id
                                    };

                                    let mut new_rt = {
                                        nodes.take(new_root_id)
                                    };

                                    nodes.restore(root, rt);
                                    nodes.drop_node(root, new_root_id);
                                    new_rt.parent = None;

                                    nodes.restore(new_root_id, new_rt);

                                    new_root_id
                                };
                            }
                            to_outside.send(Response::Ok).expect("Send failed (Ok)");
                        }*/
                    }
                };

                num_sims += montecarlo(&mut nodes, root, &thread_pool) as u64;
            }
        });

        Ai {
            to_thread, from_thread,
        }
    }

    pub fn send(&self, req: Request<G>) {
        self.to_thread.send(req).unwrap();
    }

    pub fn recv(&self) -> Option<Response<G>> {
        match self.from_thread.try_recv() {
            Ok(res) => Some(res),
            Err(TryRecvError::Empty) => None,
            Err(other) => panic!("Error: {:?}", other),
        }
    }


    pub fn make_move(&self, mv: G::Move) {
        self.to_thread.send(Request::MakeMove(mv)).unwrap();
    }

    /*pub fn make_random_move(&self) {
        self.to_thread.send(Request::MakeRandomMove).unwrap();
    }*/
}