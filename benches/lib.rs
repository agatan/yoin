#![feature(test)]
extern crate test;

use test::Bencher;


use yoin::ipadic;

#[bench]
fn bench_sumomo(b: &mut Bencher) {
    let input = "すもももももももものうち";
    let tokenizer = ipadic::tokenizer();
    b.iter(|| tokenizer.tokenize(input))
}

// from wikipedia (https://ja.wikipedia.org/wiki/形態素解析)
const LONG_TEXT: &'static str = r#"
形態素解析（けいたいそかいせき、Morphological Analysis）とは、文法的な情報の注記の無い自然言語のテキストデータ（文）から、対象言語の文法や、辞書と呼ばれる単語の品詞等の情報にもとづき、形態素（Morpheme, おおまかにいえば、言語で意味を持つ最小単位）の列に分割し、それぞれの形態素の品詞等を判別する作業である。
自然言語処理の分野における主要なテーマのひとつであり、機械翻訳やかな漢字変換など応用も多い（もちろん、かな漢字変換の場合は入力が通常の文と異なり全てひらがなであり、その先に続く文章もその時点では存在しないなどの理由で、内容は機械翻訳の場合とは異なったものになる）。
もっぱら言語学的な観点を主として言語学で研究されている文法にもとづく解析もあれば、コンピュータ上の自然言語処理としてコンピュータでの扱いやすさに主眼を置いた解析もある。以下は後者のためのツールを用いた例で、「お待ちしております」という文を形態素解析した例である （「茶筌」を使用した）。
    "#;

#[bench]
fn bench_long_text(b: &mut Bencher) {
    let tokenizer = ipadic::tokenizer();
    b.iter(|| {
        tokenizer.tokenize(LONG_TEXT);
    })
}

#[bench]
fn bench_5_times_long_text(b: &mut Bencher) {
    let tokenizer = ipadic::tokenizer();
    let mut input = String::new();
    for _ in 0..5 {
        input.push_str(LONG_TEXT);
    }
    b.iter(|| {
        tokenizer.tokenize(&input);
    })
}

#[bench]
fn bench_10_times_long_text(b: &mut Bencher) {
    let tokenizer = ipadic::tokenizer();
    let mut input = String::new();
    for _ in 0..10 {
        input.push_str(LONG_TEXT);
    }
    b.iter(|| {
        tokenizer.tokenize(&input);
    })
}
