extern crate encoding;
extern crate byteorder;

use std::env;

extern crate yoin;

use yoin::ipadic;

fn main() {
    let input = env::args().nth(1).unwrap();
    let tokenizer = ipadic::tokenizer();
    for node in tokenizer.tokenize(input.as_str()) {
        println!("{}", node);
    }
    println!("EOS");
}
