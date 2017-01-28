#![feature(test)]
extern crate test;

use test::Bencher;

extern crate yoin;
use yoin::dict::Dict;
use yoin::ipadic;

#[bench]
fn bench_lookup(b: &mut Bencher) {
    let words = include_str!("./words.txt");
    let words: Vec<_> = words.lines().collect();
    let dict = ipadic::dictionary();
    b.iter(|| {
        for word in &words {
            dict.lookup_str_iter(word).count();
        }
    });
}
