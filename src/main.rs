extern crate gtk;
extern crate rand;
extern crate num_cpus;

mod catan;
mod ai;

use catan::Catan;

fn main() {
    let catan = Catan::new();
}