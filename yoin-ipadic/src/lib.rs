extern crate yoin_core as core;

use core::dic::{FstDic, Matrix};
use core::dic::unknown::CompiledUnkDic;
use core::sysdic::SysDic;
use core::tokenizer::Tokenizer;

pub const BYTECODE: &'static [u8] = include_bytes!("../data/ipadic.dic");
pub const MORPHS: &'static [u8] = include_bytes!("../data/ipadic.morph");
pub const MATRIX: &'static [u8] = include_bytes!("../data/ipadic.matrix");
pub const UNKOWN: &'static [u8] = include_bytes!("../data/ipadic.unk");

pub fn dictionary() -> FstDic<&'static [u8]> {
    unsafe { FstDic::from_bytes(BYTECODE, MORPHS) }
}

pub fn matrix() -> Matrix<&'static [i16]> {
    unsafe { Matrix::decode(MATRIX) }
}

pub fn unkown_dic() -> CompiledUnkDic<'static> {
    unsafe { CompiledUnkDic::decode(UNKOWN) }
}

pub fn tokenizer() -> Tokenizer {
    let sysdic = SysDic {
        dic: dictionary(),
        matrix: matrix(),
        unknown_dic: unkown_dic(),
    };
    Tokenizer::new(sysdic)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::dic::unknown::{UnknownDic, CharCategorize, Category};

    #[test]
    fn test_unknown_dic() {
        let dic = unkown_dic();
        let cate = dic.categorize('ビ');
        assert_eq!(cate,
                   Category {
                       invoke: true,
                       group: true,
                       length: 2,
                   });
        let id = dic.category_id('ビ');
        for e in dic.fetch_entries(id) {
            assert!(e.contents.contains("名詞") || e.contents.contains("感動詞"),
                    "KATAKANA entry should be either '名詞' or '感動詞', got: {:?}",
                    e);
        }
        let numerics = (0..10).map(|i| {
            let c = ::std::char::from_digit(i, 10).unwrap();
            dic.category_id(c)
        });
        for n in numerics {
            assert_eq!(n, dic.category_id('0'));
        }
    }

    #[test]
    fn test_tokenize() {
        let input = "すもももももももものうち";
        let expected = vec!["すもも", "も", "もも", "も", "もも", "の", "うち"];

        let tokenizer = tokenizer();
        let tokens = tokenizer.tokenize(input);

        for (tok, e) in tokens.iter().zip(expected) {
            assert_eq!(tok.surface(), e);
            assert_eq!(&input[tok.start()..tok.end()], e);
        }
    }
}
