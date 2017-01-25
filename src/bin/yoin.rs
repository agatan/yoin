extern crate encoding;
extern crate byteorder;

use std::env;

extern crate yoin;

use yoin::dict;

fn main() {
    let bytecodes = include_bytes!("../../mecab.dic");
    let entries = include_bytes!("../../mecab.entries");
    let dict = dict::Dict::from_bytes(bytecodes, entries);
    let input = env::args().nth(1).unwrap();
    let morphs = dict.run(input.as_bytes()).unwrap();
    for morph in &morphs {
        println!("{}", morph);
    }
}
