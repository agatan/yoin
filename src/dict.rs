use byteorder::{LittleEndian, ReadBytesExt};

use fst::Fst;

use morph::Morph;

#[derive(Debug, Clone)]
pub struct Dict<'a> {
    entries: &'a [u8],
    fst: Fst<&'a [u8]>,
}

impl<'a> Dict<'a> {
    pub unsafe fn from_bytes(bytecodes: &'a [u8], entries: &'a [u8]) -> Dict<'a> {
        Dict {
            entries: entries,
            fst: Fst::from_bytes(bytecodes),
        }
    }

    pub fn run(&self, input: &[u8]) -> Result<Vec<Morph<'a>>, String> {
        self.fst.run_iter(input)
            .map(|result| result.map(|acc| self.fetch_entry(acc.value as usize)))
            .collect()
    }

    pub fn fetch_entry(&self, offset: usize) -> Morph<'a> {
        let mut entry_bytes = &self.entries[offset..];
        let size = entry_bytes.read_u32::<LittleEndian>().unwrap();
        let entry = unsafe { ::std::str::from_utf8_unchecked(&entry_bytes[..size as usize]) };
        Morph::new(entry)
    }
}
