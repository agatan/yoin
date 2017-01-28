extern crate encoding;
extern crate byteorder;

use std::env;

extern crate yoin;

use yoin::dict::Dict;
use yoin::ipadic;
use yoin::lattice::{Lattice, NodeKind};

fn main() {
    let dict = ipadic::dictionary();
    let input = env::args().nth(1).unwrap();
    let morphs = dict.lookup_str(input.as_str());
    for morph in &morphs {
        println!("{}", morph);
    }

    let mut la = Lattice::new(input.chars().count(), &dict);
    let mut input_chars = input.chars();
    while !input_chars.as_str().is_empty() {
        for m in dict.lookup_str_iter(input_chars.as_str()) {
            la.add(NodeKind::Known(m));
        }
        let cnt = la.forward();
        for _ in 0..cnt {
            input_chars.next();
        }
    }
    la.end();
    println!("backward");
    let out = la.backward();
    println!("FINISH");
    for id in out {
        let node = la.arena.get(id);
        println!("{:?}", node);
    }
}
