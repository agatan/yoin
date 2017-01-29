use dict::{FstDict, Matrix};
use dict::unknown::CompiledUnkDict;
use tokenizer::Tokenizer;

pub const BYTECODE: &'static [u8] = include_bytes!("../../data/ipadic.dic");
pub const MORPHS: &'static [u8] = include_bytes!("../../data/ipadic.morph");
pub const MATRIX: &'static [u8] = include_bytes!("../../data/ipadic.matrix");
pub const UNKOWN: &'static [u8] = include_bytes!("../../data/ipadic.unk");

pub fn dictionary() -> FstDict<&'static [u8], &'static [i16]> {
    unsafe { FstDict::from_bytes(BYTECODE, MORPHS, MATRIX) }
}

pub fn matrix() -> Matrix<&'static [i16]> {
    unsafe { Matrix::decode(MATRIX) }
}

pub fn unkown_dic() -> CompiledUnkDict<'static> {
    unsafe { CompiledUnkDict::decode(UNKOWN) }
}

pub fn tokenizer
    ()
    -> Tokenizer<'static, FstDict<&'static [u8], &'static [i16]>, CompiledUnkDict<'static>>
{
    Tokenizer::new_with_dic(dictionary(), unkown_dic())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dict::unknown::{UnknownDict, CharCategorize, Category};

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
    }
}
