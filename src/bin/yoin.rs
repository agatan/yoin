extern crate encoding;
extern crate byteorder;

use std::env;

extern crate yoin;

use yoin::dict::Dict;
use yoin::ipadic;

fn main() {
    let bytecodes = ipadic::BYTECODE;
    let morphs = ipadic::MORPHS;
    let dict = unsafe { Dict::from_bytes(bytecodes, morphs) };
    let input = env::args().nth(1).unwrap();
    let morphs = dict.lookup(input.as_bytes()).unwrap();
    for morph in &morphs {
        println!("{}", morph);
    }
}
