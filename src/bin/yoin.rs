extern crate clap;

use std::io;
use std::fs::File;

use clap::{Arg, App};

extern crate yoin;

use yoin::ipadic;

fn read_and_analyze_lines<R: io::BufRead>(r: R) -> io::Result<()> {
    let tokenizer = ipadic::tokenizer();
    for line in r.lines() {
        let line = line?;
        for node in tokenizer.tokenize(line.as_str()) {
            println!("{}", node);
        }
        println!("EOS");
    }
    Ok(())
}

fn main() {
    let matches = App::new("yoin")
        .version(yoin::VERSION)
        .about("Japanese Morphological Analyzer")
        .arg(Arg::with_name("file")
            .long("file")
            .value_name("FILE")
            .help("input file. if not specified, read from stdin")
            .takes_value(true))
        .get_matches();

    if let Some(file) = matches.value_of("file") {
        let file = File::open(file).unwrap();
        read_and_analyze_lines(io::BufReader::new(file)).unwrap();
    } else {
        let stdin = io::stdin();
        read_and_analyze_lines(stdin.lock()).unwrap();
    }
}
