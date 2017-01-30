use dic::{FstDic, Matrix};
use dic::unknown::CompiledUnkDic;
use tokenizer::Tokenizer;

pub const BYTECODE: &'static [u8] = include_bytes!("../../data/ipadic.dic");
pub const MORPHS: &'static [u8] = include_bytes!("../../data/ipadic.morph");
pub const MATRIX: &'static [u8] = include_bytes!("../../data/ipadic.matrix");
pub const UNKOWN: &'static [u8] = include_bytes!("../../data/ipadic.unk");

pub fn dictionary() -> FstDic<&'static [u8], &'static [i16]> {
    unsafe { FstDic::from_bytes(BYTECODE, MORPHS, MATRIX) }
}

pub fn matrix() -> Matrix<&'static [i16]> {
    unsafe { Matrix::decode(MATRIX) }
}

pub fn unkown_dic() -> CompiledUnkDic<'static> {
    unsafe { CompiledUnkDic::decode(UNKOWN) }
}

pub fn tokenizer
    ()
    -> Tokenizer<'static, FstDic<&'static [u8], &'static [i16]>, CompiledUnkDic<'static>>
{
    Tokenizer::new_with_dic(dictionary(), unkown_dic())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dic::unknown::{UnknownDic, CharCategorize, Category};

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
