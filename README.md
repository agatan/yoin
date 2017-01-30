## Yoin - A Japanese Morphological Analyzer

*This is still under development...*

[![Build Status](https://travis-ci.org/agatan/yoin.svg?branch=master)](https://travis-ci.org/agatan/yoin)

`yoin` is a Japanese morphological analyze engine written in pure Rust.

[mecab-ipadic](https://taku910.github.io/mecab/) is embedded in `yoin`.

```sh
:) $ yoin
すもももももももものうち
すもも	名詞,一般,*,*,*,*,すもも,スモモ,スモモ
も	助詞,係助詞,*,*,*,*,も,モ,モ
もも	名詞,一般,*,*,*,*,もも,モモ,モモ
も	助詞,係助詞,*,*,*,*,も,モ,モ
もも	名詞,一般,*,*,*,*,もも,モモ,モモ
の	助詞,連体化,*,*,*,*,の,ノ,ノ
うち	名詞,非自立,副詞可能,*,*,*,うち,ウチ,ウチ
EOS
```

## Build & Install

*`yoin` is unavailable on [crates.io](https://crates.io), because dictionary data is too big...*

### CLI

```sh
:) $ git clone https://github.com/agatan/yoin
:) $ cd yoin && cargo install
```

### Library

yoin can be included in your Cargo project like this:

```toml
[dependencies]
yoin = { git = "https://github.com/agatan/yoin" }
```

and write your code like this:

```rust
extern crate yoin;
```

## Usage - CLI

By default, `yoin` reads lines from stdin, analyzes each line, and outputs results.

```sh
:) $ yoin
すもももももももものうち
すもも	名詞,一般,*,*,*,*,すもも,スモモ,スモモ
も	助詞,係助詞,*,*,*,*,も,モ,モ
もも	名詞,一般,*,*,*,*,もも,モモ,モモ
も	助詞,係助詞,*,*,*,*,も,モ,モ
もも	名詞,一般,*,*,*,*,もも,モモ,モモ
の	助詞,連体化,*,*,*,*,の,ノ,ノ
うち	名詞,非自立,副詞可能,*,*,*,うち,ウチ,ウチ
EOS
そこではなしは終わりになった
そこで	接続詞,*,*,*,*,*,そこで,ソコデ,ソコデ
はなし	名詞,一般,*,*,*,*,はなし,ハナシ,ハナシ
は	助詞,係助詞,*,*,*,*,は,ハ,ワ
終わり	動詞,自立,*,*,五段・ラ行,連用形,終わる,オワリ,オワリ
に	助詞,格助詞,一般,*,*,*,に,ニ,ニ
なっ	動詞,自立,*,*,五段・ラ行,連用タ接続,なる,ナッ,ナッ
た	助動詞,*,*,*,特殊・タ,基本形,た,タ,タ
EOS
```

Or, reads from file.

```sh
:) $ cat input.txt
すもももももももものうち
:) $ yoin --file input.txt
すもも	名詞,一般,*,*,*,*,すもも,スモモ,スモモ
も	助詞,係助詞,*,*,*,*,も,モ,モ
もも	名詞,一般,*,*,*,*,もも,モモ,モモ
も	助詞,係助詞,*,*,*,*,も,モ,モ
もも	名詞,一般,*,*,*,*,もも,モモ,モモ
の	助詞,連体化,*,*,*,*,の,ノ,ノ
うち	名詞,非自立,副詞可能,*,*,*,うち,ウチ,ウチ
EOS
```

## LICENSE

This software in under the MIT License and contains the MeCab-ipadic model.
See `LICENSE` and `NOTICE.txt` for more details.
