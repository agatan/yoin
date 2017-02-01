use dic::{FstDic, Matrix};
use dic::unknown::CompiledUnkDic;

pub struct SysDic {
    pub dic: FstDic<&'static [u8]>,
    pub matrix: Matrix<&'static [i16]>,
    pub unknown_dic: CompiledUnkDic<'static>,
}
