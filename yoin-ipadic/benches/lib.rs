#![feature(test)]
extern crate test;

use test::Bencher;

extern crate yoin_ipadic as ipadic;
extern crate yoin_core as core;
use core::dic::Dic;

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
