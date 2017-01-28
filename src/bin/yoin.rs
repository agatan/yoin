extern crate encoding;
extern crate byteorder;

use std::env;

extern crate yoin;

use yoin::dict::Dict;
use yoin::ipadic;

fn main() {
    let dict = ipadic::dictionary();
    let input = env::args().nth(1).unwrap();
    let morphs = dict.lookup_str(input.as_str()).unwrap();
    for morph in &morphs {
        println!("{}", morph);
    }
}
