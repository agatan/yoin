use std::convert::AsRef;
use std::iter::Iterator;

use fst::{Fst, FstIter};

use morph::Morph;

pub trait Dict<'a> {
    type Str: AsRef<str>;
    type Iterator: Iterator<Item=Result<Morph<Self::Str>, String>>;

    fn lookup_iter(&'a self, input: &'a [u8]) -> Self::Iterator;
    fn lookup(&'a self, input: &'a [u8]) -> Result<Vec<Morph<Self::Str>>, String> {
        self.lookup_iter(input).collect()
    }
    fn lookup_str(&'a self, input: &'a str) -> Self::Iterator {
        self.lookup_iter(input.as_bytes())
    }
}

#[derive(Debug, Clone)]
pub struct DecodedDict<'a> {
    entries: &'a [u8],
    fst: Fst<&'a [u8]>,
}

impl<'a> DecodedDict<'a> {
    pub unsafe fn from_bytes(bytecodes: &'a [u8], entries: &'a [u8]) -> Self {
        DecodedDict {
            entries: entries,
            fst: Fst::from_bytes(bytecodes),
        }
    }
}

pub struct DecodedMorphIter<'a> {
    entries: &'a [u8],
    iter: FstIter<'a>,
}

impl<'a> DecodedMorphIter<'a> {
    fn fetch_entry(&self, offset: usize) -> Morph<&'a str> {
        let entry_bytes = &self.entries[offset..];
        unsafe { Morph::decode(entry_bytes) }
    }
}

impl<'a> Iterator for DecodedMorphIter<'a> {
    type Item = Result<Morph<&'a str>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(r) = self.iter.next() {
            Some(r.map(|o| self.fetch_entry(o.value as usize)))
        } else {
            None
        }
    }
}

impl<'a> Dict<'a> for DecodedDict<'a> {
    type Str = &'a str;
    type Iterator = DecodedMorphIter<'a>;

    fn lookup_iter(&'a self, input: &'a [u8]) -> Self::Iterator {
        DecodedMorphIter{
            entries: self.entries,
            iter: self.fst.run_iter(input),
        }
    }
}

