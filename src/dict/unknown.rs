use std::io::{self, Write};

use byteorder::{ByteOrder, WriteBytesExt, NativeEndian};

pub type Category = u8;
const DEFAULT_CATEGORY: Category = 0;

pub trait CharCategory {
    fn char_category(&self, ch: char) -> Category;
    fn invoke(&self, c: Category) -> bool;
    fn group(&self, c: Category) -> bool;
    fn length(&self, c: Category) -> u8;
}

pub struct CharTable {
    pub invokes: Vec<bool>,
    pub groups: Vec<bool>,
    pub lengths: Vec<u8>,
    pub table: [Category; ::std::u16::MAX as usize],
}

impl CharCategory for CharTable {
    fn char_category(&self, ch: char) -> Category {
        let ch = ch as u32;
        if ch < ::std::u16::MAX as u32 {
            self.table[ch as usize]
        } else {
            DEFAULT_CATEGORY
        }
    }

    fn invoke(&self, c: Category) -> bool {
        self.invokes[c as usize]
    }

    fn group(&self, c: Category) -> bool {
        self.groups[c as usize]
    }

    fn length(&self, c: Category) -> u8 {
        self.lengths[c as usize]
    }
}

impl CharTable {
    pub fn encode<W: Write>(&self, mut w: W) -> io::Result<()> {
        let n = self.invokes.len() as u8;
        w.write_u8(n)?;
        for &b in self.invokes.iter() {
            w.write_u8(b as u8)?;
        }
        for &b in self.groups.iter() {
            w.write_u8(b as u8)?;
        }
        for &b in self.lengths.iter() {
            w.write_u8(b)?;
        }
        for &b in self.table.iter() {
            w.write_u8(b)?;
        }
        Ok(())
    }

    pub fn encode_native<W: Write>(&self, w: W) -> io::Result<()> {
        self.encode::<W>(w)
    }
}

pub struct CompiledCharTable<'a> {
    pub n_categories: u8,
    pub invokes: &'a [u8],
    pub groups: &'a [u8],
    pub lengths: &'a [u8],
    pub table: &'a [Category],
}

impl<'a> CharCategory for CompiledCharTable<'a> {
    fn char_category(&self, ch: char) -> Category {
        let ch = ch as u32;
        if ch < ::std::u16::MAX as u32 {
            self.table[ch as usize]
        } else {
            DEFAULT_CATEGORY
        }
    }

    fn invoke(&self, c: Category) -> bool {
        self.invokes[c as usize] != 0
    }

    fn group(&self, c: Category) -> bool {
        self.groups[c as usize] != 0
    }

    fn length(&self, c: Category) -> u8 {
        self.lengths[c as usize]
    }
}

impl<'a> CompiledCharTable<'a> {
    pub unsafe fn decode(bs: &'a [u8]) -> Self {
        let ptr = bs.as_ptr() as *const u8;
        let n = *ptr;
        let ptr = ptr.offset(1);
        let invokes = ::std::slice::from_raw_parts(ptr, n as usize);
        let ptr = ptr.offset(n as isize);
        let groups = ::std::slice::from_raw_parts(ptr, n as usize);
        let ptr = ptr.offset(n as isize);
        let lengths = ::std::slice::from_raw_parts(ptr, n as usize);
        let ptr = ptr.offset(n as isize);
        let table = ::std::slice::from_raw_parts(ptr, ::std::u16::MAX as usize);
        CompiledCharTable {
            n_categories: n,
            invokes: invokes,
            groups: groups,
            lengths: lengths,
            table: table,
        }
    }
}

#[test]
fn test_encode_decode() {
    let mut table = CharTable {
        invokes: vec![true, false, false],
        groups: vec![false, true, false],
        lengths: vec![0, 1, 2],
        table: [DEFAULT_CATEGORY; ::std::u16::MAX as usize],
    };
    table.table['あ' as usize] = 1;
    table.table['a' as usize] = 2;

    let mut buf = Vec::new();
    table.encode(&mut buf).unwrap();

    let compiled = unsafe { CompiledCharTable::decode(&buf) };

    let tests = vec![('0', (true, false, 0)), ('あ', (false, true, 1)), ('a', (false, false, 2))];

    for (ch, (i, g, l)) in tests {
        let category = compiled.char_category(ch);
        assert_eq!(category, table.char_category(ch));
        assert_eq!(compiled.invoke(category), i);
        assert_eq!(compiled.group(category), g);
        assert_eq!(compiled.length(category), l);
    }
}

#[derive(Debug)]
pub struct Entry<'a> {
    pub left_id: u16,
    pub right_id: u16,
    pub weight: i16,
    pub contents: &'a str,
}

impl<'a> Entry<'a> {
    pub fn encode<W: Write, O: ByteOrder>(&self, mut w: W) -> io::Result<()> {
        w.write_u16::<O>(self.left_id)?;
        w.write_u16::<O>(self.right_id)?;
        w.write_i16::<O>(self.weight)?;
        w.write_u32::<O>(self.contents.len() as u32)?;
        for &b in self.contents.as_bytes() {
            w.write_u8(b)?;
        }
        Ok(())
    }

    pub fn encode_native<W: Write>(&self, w: W) -> io::Result<()> {
        self.encode::<_, NativeEndian>(w)
    }

    pub unsafe fn decode(bs: &'a [u8]) -> Self {
        let ptr = bs.as_ptr() as *const u16;
        let left_id = *ptr;
        let right_id = *ptr.offset(1);
        let ptr = ptr.offset(2) as *const i16;
        let weight = *ptr;
        let ptr = ptr.offset(1) as *const u32;
        let len = *ptr;
        let ptr = ptr.offset(1) as *const u8;
        let buf = ::std::slice::from_raw_parts(ptr, len as usize);
        let contents = ::std::str::from_utf8_unchecked(buf);
        Entry {
            left_id: left_id,
            right_id: right_id,
            weight: weight,
            contents: contents,
        }
    }
}

pub trait UnknownDict: CharCategory {
    fn fetch_entries<'a>(&'a self, cate: Category) -> Vec<Entry<'a>>;
}

pub struct UnkDict {
    pub indices: Vec<u32>, // Category -> Index of initial entry
    pub counts: Vec<u32>, // Category -> Counts of entries
    pub entry_offsets: Vec<u32>,
    pub entries: Vec<u8>,
    pub categories: CharTable,
}

impl CharCategory for UnkDict {
    fn char_category(&self, ch: char) -> Category {
        self.categories.char_category(ch)
    }

    fn invoke(&self, cate: Category) -> bool {
        self.categories.invoke(cate)
    }

    fn group(&self, cate: Category) -> bool {
        self.categories.group(cate)
    }

    fn length(&self, cate: Category) -> u8 {
        self.categories.length(cate)
    }
}

impl UnknownDict for UnkDict {
    fn fetch_entries<'a>(&'a self, cate: Category) -> Vec<Entry<'a>> {
        let count = self.counts[cate as usize] as usize;
        let index = self.indices[cate as usize] as usize;
        let offsets = &self.entry_offsets[index..index + count];
        let mut results = Vec::with_capacity(count);
        for &offset in offsets {
            results.push(unsafe { Entry::decode(&self.entries[offset as usize..]) });
        }
        results
    }
}

impl UnkDict {
    pub fn encode<W: Write, O: ByteOrder>(&self, mut w: W) -> io::Result<()> {
        w.write_u32::<O>(self.indices.len() as u32)?;
        for i in &self.indices {
            w.write_u32::<O>(*i)?;
        }
        w.write_u32::<O>(self.counts.len() as u32)?;
        for i in &self.counts {
            w.write_u32::<O>(*i)?;
        }
        w.write_u32::<O>(self.entry_offsets.len() as u32)?;
        for i in &self.entry_offsets {
            w.write_u32::<O>(*i)?;
        }
        w.write_u32::<O>(self.entries.len() as u32)?;
        for b in &self.entries {
            w.write_u8(*b)?;
        }
        self.categories.encode(w)
    }

    pub fn encode_native<W: Write>(&self, w: W) -> io::Result<()> {
        self.encode::<_, NativeEndian>(w)
    }
}

pub struct CompiledUnkDict<'a> {
    indices: &'a [u32],
    counts: &'a [u32],
    entry_offsets: &'a [u32],
    entries: &'a [u8],
    categories: CompiledCharTable<'a>,
}

impl<'a> CharCategory for CompiledUnkDict<'a> {
    fn char_category(&self, ch: char) -> Category {
        self.categories.char_category(ch)
    }

    fn invoke(&self, cate: Category) -> bool {
        self.categories.invoke(cate)
    }

    fn group(&self, cate: Category) -> bool {
        self.categories.group(cate)
    }

    fn length(&self, cate: Category) -> u8 {
        self.categories.length(cate)
    }
}

impl<'a> UnknownDict for CompiledUnkDict<'a> {
    fn fetch_entries<'b>(&'b self, cate: Category) -> Vec<Entry<'b>> {
        let count = self.counts[cate as usize] as usize;
        let index = self.indices[cate as usize] as usize;
        let offsets = &self.entry_offsets[index..index + count];
        let mut results = Vec::with_capacity(count);
        for &offset in offsets {
            results.push(unsafe { Entry::decode(&self.entries[offset as usize..]) });
        }
        results
    }
}

impl<'a> CompiledUnkDict<'a> {
    pub unsafe fn decode(bs: &'a [u8]) -> Self {
        let ptr = bs.as_ptr() as *const u32;
        let ind_len = *ptr;
        let ptr = ptr.offset(1) as *const u32;
        let indices = ::std::slice::from_raw_parts(ptr, ind_len as usize);
        let ptr = ptr.offset(ind_len as isize);
        let counts_len = *ptr;
        let ptr = ptr.offset(1) as *const u32;
        let counts = ::std::slice::from_raw_parts(ptr, counts_len as usize);
        let ptr = ptr.offset(counts_len as isize);
        let entry_offsets_len = *ptr;
        let ptr = ptr.offset(1) as *const u32;
        let entry_offsets = ::std::slice::from_raw_parts(ptr, entry_offsets_len as usize);
        let ptr = ptr.offset(entry_offsets_len as isize);
        let entries_len = *ptr;
        let ptr = ptr.offset(1) as *const u8;
        let entries = ::std::slice::from_raw_parts(ptr, entries_len as usize);
        let ptr = ptr.offset(entries_len as isize);
        let ptr_diff = ptr as usize - bs.as_ptr() as usize;
        let bs = &bs[ptr_diff..];
        let categories = CompiledCharTable::decode(bs);

        CompiledUnkDict {
            indices: indices,
            counts: counts,
            entry_offsets: entry_offsets,
            entries: entries,
            categories: categories,
        }
    }
}
