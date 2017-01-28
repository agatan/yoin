extern crate encoding;
extern crate byteorder;

use std::env;

extern crate yoin;

use yoin::dict::Dict;
use yoin::ipadic;
use yoin::lattice::Lattice;

fn main() {
    let dict = ipadic::dictionary();
    let input = env::args().nth(1).unwrap();
    let morphs = dict.lookup_str(input.as_str());
    for morph in &morphs {
        println!("{}", morph);
    }

    let la = Lattice::build(input.as_str(), &dict);
    let out = la.output();
    for node in out {
        println!("{:?}", node);
    }
}
