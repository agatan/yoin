use std::convert::AsRef;
use std::iter::Iterator;

use fst::{Fst, FstIter};

use morph::Morph;

#[derive(Debug, Clone)]
pub struct Dict<T: AsRef<[u8]>> {
    morph_bytes: T,
    fst: Fst<T>,
}

impl<'a> Dict<&'a [u8]> {
    pub unsafe fn from_bytes(bytecodes: &'a [u8], morph_bytes: &'a [u8]) -> Self {
        Dict {
            morph_bytes: morph_bytes,
            fst: Fst::from_bytes(bytecodes),
        }
    }
}

impl<T: AsRef<[u8]>> Dict<T> {
    pub fn lookup_iter<'a>(&'a self, input: &'a [u8]) -> Iter<'a> {
        Iter {
            morph_bytes: self.morph_bytes.as_ref(),
            iter: self.fst.run_iter(input),
        }
    }

    pub fn lookup<'a>(&'a self, input: &'a [u8]) -> Result<Vec<Morph<&'a str>>, String> {
        self.lookup_iter(input).collect()
    }

    pub fn lookup_str<'a>(&'a self, input: &'a str) -> Iter<'a> {
        self.lookup_iter(input.as_bytes())
    }
}

pub struct Iter<'a> {
    morph_bytes: &'a [u8],
    iter: FstIter<'a>,
}

impl<'a> Iter<'a> {
    fn fetch_entry(&self, offset: usize) -> Morph<&'a str> {
        let entry_bytes = &self.morph_bytes[offset..];
        unsafe { Morph::decode(entry_bytes) }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Result<Morph<&'a str>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(r) = self.iter.next() {
            Some(r.map(|o| self.fetch_entry(o.value as usize)))
        } else {
            None
        }
    }
}
