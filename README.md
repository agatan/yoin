## yoin-core - A Japanese Morphological Analyzer

This is a core component of `yoin` Japanese morphological analyzer.

`yoin` main repository is [agatan/yoin](https://github.com/agatan/yoin).

*This is still under development...*

[![Build Status](https://travis-ci.org/agatan/yoin-core.svg?branch=master)](https://travis-ci.org/agatan/yoin-core)

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

## LICENSE

This software in under the MIT License and contains the MeCab-ipadic model.
See `LICENSE` and `NOTICE.txt` for more details.
