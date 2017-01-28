use std::convert::AsRef;
use std::iter::Iterator;

use fst::{Fst, FstIter};

use morph::Morph;

pub trait Dict<'a> {
    type Iterator: Iterator<Item=Morph<&'a str>>;
    fn lookup_iter(&'a self, input: &'a [u8]) -> Self::Iterator;
    fn lookup(&'a self, input: &'a [u8]) -> Vec<Morph<&'a str>> {
        self.lookup_iter(input).collect()
    }
    fn lookup_str_iter(&'a self, input: &'a str) -> Self::Iterator {
        self.lookup_iter(input.as_bytes())
    }
    fn lookup_str(&'a self, input: &'a str) -> Vec<Morph<&'a str>> {
        self.lookup_str_iter(input).collect()
    }
}

#[derive(Debug, Clone)]
pub struct FstDict<T: AsRef<[u8]>> {
    morph_bytes: T,
    fst: Fst<T>,
}

impl<'a> FstDict<&'a [u8]> {
    pub unsafe fn from_bytes(bytecodes: &'a [u8], morph_bytes: &'a [u8]) -> Self {
        FstDict {
            morph_bytes: morph_bytes,
            fst: Fst::from_bytes(bytecodes),
        }
    }
}

impl FstDict<Vec<u8>> {
    pub fn build<S: AsRef<str>>(morphs: &[Morph<S>]) -> Self {
        let mut morph_bytes = Vec::new();
        let mut fst_inputs = Vec::new();
        for morph in morphs {
            let offset = morph_bytes.len();
            let surface = morph.surface.as_ref().as_bytes();
            fst_inputs.push((surface, offset as u32));
            morph.encode_native(&mut morph_bytes).unwrap();
        }
        fst_inputs.sort();
        let fst = Fst::build(fst_inputs);
        FstDict {
            morph_bytes: morph_bytes,
            fst: fst,
        }
    }
}

impl<'a, T: AsRef<[u8]>> Dict<'a> for FstDict<T> {
    type Iterator = Iter<'a>;

    fn lookup_iter(&'a self, input: &'a [u8]) -> Iter<'a> {
        Iter {
            morph_bytes: self.morph_bytes.as_ref(),
            iter: self.fst.run_iter(input),
        }
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
    type Item = Morph<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|acc| self.fetch_entry(acc.value as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morph::Morph;

    #[test]
    fn test_build_lookup() {
        let morphs = vec![Morph {
                              surface: "す",
                              left_id: 1,
                              right_id: 1,
                              weight: 1,
                              contents: "contents 1",
                          },
                          Morph {
                              surface: "す",
                              left_id: 2,
                              right_id: 2,
                              weight: 2,
                              contents: "contents 2",
                          },
                          Morph {
                              surface: "すも",
                              left_id: 3,
                              right_id: 3,
                              weight: 3,
                              contents: "contents 3",
                          },
                          Morph {
                              surface: "すもも",
                              left_id: 4,
                              right_id: 4,
                              weight: 4,
                              contents: "contents 4",
                          }];
        let dict = FstDict::build(&morphs);
        let results = dict.lookup_str("すもも");
        assert_eq!(results.len(), morphs.len());
        // the order of lookup results is not fixed.
        for result in results {
            assert!(morphs.iter().any(|m| *m == result),
                    "invalid result: {:?}",
                    result);
        }
    }
}
