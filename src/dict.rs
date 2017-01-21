use byteorder::{LittleEndian, ReadBytesExt};

use morph::Morph;
use op;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dict<'a> {
    entries: &'a [u8],
    bytecodes: &'a [u8],
}

impl<'a> Dict<'a> {
    pub fn from_bytes(bytecodes: &'a [u8], entries: &'a [u8]) -> Dict<'a> {
        Dict {
            entries: entries,
            bytecodes: bytecodes,
        }
    }

    pub fn run(&self, input: &[u8]) -> Result<Vec<Morph<'a>>, String> {
        op::run_iter(self.bytecodes, input)
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
