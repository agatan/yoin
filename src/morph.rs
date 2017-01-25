use std::fmt;
use std::io::{self, Write};
use std::convert::AsRef;

use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Morph<S>
    where S: AsRef<str>
{
    pub surface: S,
    pub left_id: i16,
    pub right_id: i16,
    pub weight: i16,
    pub contents: S,
}

impl<S: AsRef<str>> Morph<S> {
    pub fn encode<W: Write>(&self, mut w: W) -> io::Result<()> {
        let surface_bytes = self.surface.as_ref().as_bytes();
        w.write_u32::<LittleEndian>(surface_bytes.len() as u32)?;
        w.write_all(surface_bytes)?;
        w.write_i16::<LittleEndian>(self.left_id)?;
        w.write_i16::<LittleEndian>(self.right_id)?;
        w.write_i16::<LittleEndian>(self.weight)?;
        let contents_bytes = self.contents.as_ref().as_bytes();
        w.write_u32::<LittleEndian>(contents_bytes.len() as u32)?;
        w.write_all(contents_bytes)?;
        Ok(())
    }
}

impl<'a> Morph<&'a str> {
    pub unsafe fn decode(mut bs: &'a [u8]) -> Self {
        let surface_len = bs.read_u32::<LittleEndian>().unwrap();
        let surface = ::std::str::from_utf8_unchecked(&bs[..surface_len as usize]);
        bs = &bs[surface_len as usize..];
        let left_id = bs.read_i16::<LittleEndian>().unwrap();
        let right_id = bs.read_i16::<LittleEndian>().unwrap();
        let weight = bs.read_i16::<LittleEndian>().unwrap();
        let contents_len = bs.read_u32::<LittleEndian>().unwrap();
        let contents = ::std::str::from_utf8_unchecked(&bs[..contents_len as usize]);

        Morph {
            surface: surface,
            left_id: left_id,
            right_id: right_id,
            weight: weight,
            contents: contents,
        }
    }
}

impl<S: AsRef<str>> fmt::Display for Morph<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{},{},{},{},{}",
               self.surface.as_ref(),
               self.left_id,
               self.right_id,
               self.weight,
               self.contents.as_ref())
    }
}
