#![feature(test)]
extern crate test;
extern crate rand;

use test::Bencher;

extern crate yoin;
use yoin::op;

#[bench]
fn bench_lookup(b: &mut Bencher) {
    let mut rng = rand::thread_rng();
    let words = include_str!("./words.txt");
    let words: Vec<_> = words.lines().collect();
    let sample = rand::sample(&mut rng, words, 1000);
    let bytes = include_bytes!("../mecab.dic");
    let sample = &sample;
    b.iter(|| {
        for input in sample {
            op::run(bytes, input.as_bytes()).unwrap();
        }
    })
}