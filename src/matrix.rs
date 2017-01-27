use std::convert::AsRef;
use std::io::{self, Write};

use byteorder::{NativeEndian, WriteBytesExt, ByteOrder};

#[derive(Debug, Clone, PartialEq)]
pub struct Matrix<T: AsRef<[i16]>> {
    width: u32,
    height: u32,
    table: T,
}

impl<T: AsRef<[i16]>> Matrix<T> {
    pub fn encode<W: Write, O: ByteOrder>(&self, mut w: W) -> io::Result<()> {
        w.write_u32::<O>(self.width)?;
        w.write_u32::<O>(self.height)?;
        for &byte in self.table.as_ref() {
            w.write_i16::<O>(byte)?;
        }
        Ok(())
    }

    pub fn encode_native<W: Write>(&self, w: W) -> io::Result<()> {
        self.encode::<W, NativeEndian>(w)
    }
}

impl<'a> Matrix<&'a [i16]> {
    pub unsafe fn decode(bs: &'a [u8]) -> Self {
        let ptr = bs.as_ptr() as *const u32;
        let width = *ptr;
        let height = *ptr.offset(1);
        let ptr = ptr.offset(2) as *const i16;
        let table = ::std::slice::from_raw_parts(ptr, (width * height) as usize);
        Matrix {
            width: width,
            height: height,
            table: table,
        }
    }
}

#[test]
fn test_matrix_encode_decode() {
    let table: &[i16] = &[-3, -2, -1, 0, 1, 2];
    let matrix = Matrix {
        width: 2,
        height: 3,
        table: table,
    };
    let mut buf = Vec::new();
    matrix.encode_native(&mut buf).unwrap();
    let decoded = unsafe { Matrix::decode(&buf) };
    assert_eq!(decoded, matrix);
}
