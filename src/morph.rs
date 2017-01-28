use std::fmt;
use std::io::{self, Write};
use std::convert::AsRef;

use byteorder::{ByteOrder, NativeEndian, WriteBytesExt};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Morph<S>
    where S: AsRef<str>
{
    pub surface: S,
    pub left_id: u16,
    pub right_id: u16,
    pub weight: i16,
    pub contents: S,
}

impl<S: AsRef<str>> Morph<S> {
    pub fn encode<W: Write, O: ByteOrder>(&self, mut w: W) -> io::Result<()> {
        let surface_bytes = self.surface.as_ref().as_bytes();
        w.write_u32::<O>(surface_bytes.len() as u32)?;
        w.write_all(surface_bytes)?;
        w.write_u16::<O>(self.left_id)?;
        w.write_u16::<O>(self.right_id)?;
        w.write_i16::<O>(self.weight)?;
        let contents_bytes = self.contents.as_ref().as_bytes();
        w.write_u32::<O>(contents_bytes.len() as u32)?;
        w.write_all(contents_bytes)?;
        Ok(())
    }

    pub fn encode_native<W: Write>(&self, w: W) -> io::Result<()> {
        self.encode::<W, NativeEndian>(w)
    }
}

impl<'a> Morph<&'a str> {
    pub unsafe fn decode(mut bs: &'a [u8]) -> Self {
        let ptr = bs.as_ptr() as *const u32;
        let surface_len = *ptr;
        bs = &bs[::std::mem::size_of::<u32>()..];
        let surface = ::std::str::from_utf8_unchecked(&bs[..surface_len as usize]);
        bs = &bs[surface_len as usize..];

        let ptr = bs.as_ptr() as *const u16;
        let left_id = *ptr;
        let right_id = *ptr.offset(1);
        let weight = *(ptr.offset(2) as *const i16);
        bs = &bs[::std::mem::size_of::<i16>() * 3..];

        let ptr = bs.as_ptr() as *const u32;
        let contents_len = *ptr;
        bs = &bs[::std::mem::size_of::<u32>()..];
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

#[test]
fn test_encode_decode() {
    let m = Morph {
        surface: "見出し語",
        left_id: 1,
        right_id: 2,
        weight: 3,
        contents: "contents",
    };
    let mut buf = Vec::new();
    m.encode_native(&mut buf).unwrap();
    let m2 = unsafe { Morph::decode(&buf) };
    assert_eq!(m2, m);
}
