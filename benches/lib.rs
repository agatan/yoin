#![feature(test)]
extern crate test;

use test::Bencher;

extern crate yoin;

#[bench]
fn bench_lookup(b: &mut Bencher) {
    let words = include_str!("./words.txt");
    let words: Vec<_> = words.lines().collect();
    let bytes = include_bytes!("../data/mecab.dic");
    let fst = unsafe { yoin::fst::Fst::from_bytes(bytes) };
    b.iter(|| {
        for word in &words {
            fst.run(word.as_bytes()).unwrap();
        }
    });
}
