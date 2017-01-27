use std::convert::AsRef;
use std::io::{self, Write};

use byteorder::{NativeEndian, WriteBytesExt, ByteOrder};

#[derive(Debug, Clone)]
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
