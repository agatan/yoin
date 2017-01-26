extern crate encoding;
extern crate byteorder;

use std::env;

extern crate yoin;

use yoin::dict;
use yoin::ipadic;

fn main() {
    let bytecodes = ipadic::BYTECODE;
    let entries = ipadic::ENTRIES;
    let dict = unsafe { dict::Dict::from_bytes(bytecodes, entries) };
    let input = env::args().nth(1).unwrap();
    let morphs = dict.run(input.as_bytes()).unwrap();
    for morph in &morphs {
        println!("{}", morph);
    }
}
