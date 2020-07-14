use std::f64::INFINITY;
use std::collections::HashMap;

use rand::{ thread_rng, Rng };

use super::{ Game, MoveList };
use super::tree::*;
use super::ai::{ NodeID, NodeList };
use ai::sim_thread_pool::SimThreadPool;

const EXPLORATION_FACTOR: f64 = 1.4142135623730950488016887242097; // sqrt(2)

fn all_max<'a, G, K, I>(list: I, nodes: &mut NodeList<G>) -> (f64, Vec<(&'a K, &'a NodeID)> )
    where I: Iterator<Item=(&'a K, &'a NodeID)>,
          G: Game
{
    list.into_iter().fold((0.0, Vec::new()), |acc, entry| {
        let (max, mut items) = acc;
        let uct = nodes.get(*entry.1).score;

        if uct == max {
            items.push(entry);
            (max, items)
        } else if uct > max {
            (uct, vec!(entry))
        } else {
            (max, items)
        }
    })
}

fn random_choice<'a, G, I>(list: I, weights: &HashMap<G::Move, f64>, nodes: &mut NodeList<G>) -> (&'a G::Move, &'a NodeID)
    where I: Iterator<Item=(&'a G::Move, &'a NodeID)>,
          G: Game
{
    let n = thread_rng().next_f64();
    let mut sum = 0.0;
    for (mv, node_id) in list.into_iter() {
        let weight = weights.get(mv).unwrap();
        sum += *weight;
        if sum >= n {
            return (mv, node_id);
        }
    }
    unreachable!();
}

pub(in super) fn montecarlo<G: Game + 'static>(nodes: &mut NodeList<G>, root: NodeID, thread_pool: &SimThreadPool<G>) -> u32 {
    let mut rand = thread_rng();

    // Select
    let mut cur_node_id = root;
    let mut sim_this = false;
    let mut path = Vec::new();
    while !sim_this {
        let mut node = nodes.take(cur_node_id);
        let last_node_id = cur_node_id;

        if node.children.len() > 0 {
            if let Some(ref weights) = node.weights {
                let (mv, child) = random_choice(node.children.iter(), weights, nodes);
                path.push((cur_node_id, mv.clone()));

                cur_node_id = *child;

                // no need to simulate a random node because there is no choice to be made
            } else {
                let (max_val, max_list) = all_max(node.children.iter(), nodes);
                let (mv, child) = *rand.choose(&max_list).unwrap();
                path.push((cur_node_id, mv.clone()));

                cur_node_id = *child;

                if max_val == INFINITY {
                    // simulate this node
                    sim_this = true;
                }
                // otherwise, select again from this node
            }
        } else {
            // Expand
            match node.game.available_moves() {
                MoveList::Choice(mvs) => {
                    for mv in mvs {
                        let mut new_game = node.game.clone();
                        new_game.make_move(&mv);

                        let new_node = nodes.add(MoveTreeNode::new(new_game));
                        node.children.insert(mv, new_node);
                    }
                }
                MoveList::Random(mvs) => {
                    let total: f64 = mvs.iter().map(|i| i.1).sum();

                    let mut weights = HashMap::new();
                    for (mv, weight) in mvs {
                        let mut new_game = node.game.clone();
                        new_game.make_move(&mv);

                        let new_node = nodes.add(MoveTreeNode::new(new_game));
                        node.children.insert(mv.clone(), new_node);
                        weights.insert(mv, weight / total);
                    }
                    node.weights = Some(weights);
                }
            }

            if node.children.len() == 0 {
                sim_this = true;
            }

            // select a child from the current node
        }

        nodes.restore(last_node_id, node);
    }

    // Simulate
    let (num_sims, results) = {
        let cur_node = nodes.get_mut(cur_node_id);

        thread_pool.simulate(cur_node.game.clone(), 25)
    };

    // Backprop
    for (node_id, mv) in path {
        let mut cur_node = nodes.get_mut(node_id);
        //let player = &cur_node.player.clone();

        cur_node.games += num_sims;

        {
            if let Some(child_id) = cur_node.children.get(&mv) {
                let mut child = nodes.get_mut(*child_id);
                child.games += num_sims;
                child.simulations += 1;
                if let Some(&wins) = results.get(&child.player) {
                    child.wins += wins;
                }
            } else {
                // rare race condition; the node has been dropped so who cares
            }
        }

        if cur_node.weights.is_none() {
            let total_games = cur_node.games;
            for child_id in cur_node.children.values() {
                let mut child = nodes.get_mut(*child_id);
                if child.games != 0 {
                    // UCT
                    child.score = (child.wins as f64 / child.games as f64) + EXPLORATION_FACTOR * ((total_games as f64).ln() / (child.games as f64)).sqrt();
                }
            }
        }
    }

    num_sims

}