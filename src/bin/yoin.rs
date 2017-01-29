extern crate encoding;
extern crate byteorder;

use std::io::prelude::*;
use std::io;

extern crate yoin;

use yoin::ipadic;

fn main() {
    let tokenizer = ipadic::tokenizer();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        for node in tokenizer.tokenize(line.as_str()) {
            println!("{}", node);
        }
        println!("EOS");
    }
}
