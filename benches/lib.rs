#![feature(test)]
extern crate test;

use test::Bencher;

extern crate yoin;
use yoin::op;

#[bench]
fn bench_lookup(b: &mut Bencher) {
    let words = include_str!("./words.txt");
    let words: Vec<_> = words.lines().collect();
    let bytes = include_bytes!("../mecab.dic");
    b.iter(|| {
        for word in &words {
            op::run(bytes, word.as_bytes()).unwrap();
        }
    });
}