use std::io::{self, Write};
use std::collections::HashMap;

use byteorder::{ByteOrder, WriteBytesExt, NativeEndian};

pub type CategoryId = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Category {
    pub invoke: bool,
    pub group: bool,
    pub length: u8,
}

pub trait CharCategorize {
    fn categorize(&self, ch: char) -> Category;
    fn category_id(&self, ch: char) -> CategoryId;
}

pub struct CharTable {
    pub default_id: CategoryId,
    pub categories: Vec<Category>,
    pub table: [CategoryId; ::std::u16::MAX as usize],
}

impl CharTable {
    pub fn new(default_id: CategoryId, categories: Vec<Category>) -> CharTable {
        CharTable {
            default_id: default_id,
            categories: categories,
            table: [default_id; ::std::u16::MAX as usize],
        }
    }

    pub fn set(&mut self, ch: usize, id: CategoryId) {
        if ch < self.table.len() {
            self.table[ch] = id;
        }
    }
}

impl CharCategorize for CharTable {
    fn categorize(&self, ch: char) -> Category {
        let id = self.category_id(ch);
        self.categories[id as usize]
    }

    fn category_id(&self, ch: char) -> CategoryId {
        let ch = ch as u32;
        if ch < ::std::u16::MAX as u32 {
            self.table[ch as usize]
        } else {
            self.default_id
        }
    }
}

impl CharTable {
    pub fn encode<W: Write>(&self, mut w: W) -> io::Result<()> {
        let n = self.categories.len() as u8;
        w.write_u8(n)?;
        w.write_u8(self.default_id)?;
        for c in self.categories.iter() {
            w.write_u8(c.invoke as u8)?;
        }
        for c in self.categories.iter() {
            w.write_u8(c.group as u8)?;
        }
        for c in self.categories.iter() {
            w.write_u8(c.length as u8)?;
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
    pub default_id: u8,
    pub invokes: &'a [u8],
    pub groups: &'a [u8],
    pub lengths: &'a [u8],
    pub table: &'a [CategoryId],
}

impl<'a> CharCategorize for CompiledCharTable<'a> {
    fn categorize(&self, ch: char) -> Category {
        let id = self.category_id(ch) as usize;
        Category {
            invoke: self.invokes[id] != 0,
            group: self.groups[id] != 0,
            length: self.lengths[id],
        }
    }

    fn category_id(&self, ch: char) -> CategoryId {
        let ch = ch as u32;
        if ch < ::std::u16::MAX as u32 {
            self.table[ch as usize]
        } else {
            self.default_id
        }
    }
}

impl<'a> CompiledCharTable<'a> {
    pub unsafe fn decode(bs: &'a [u8]) -> Self {
        let ptr = bs.as_ptr() as *const u8;
        let n = *ptr;
        let default_id = *ptr.offset(1);
        let ptr = ptr.offset(2);
        let invokes = ::std::slice::from_raw_parts(ptr, n as usize);
        let ptr = ptr.offset(n as isize);
        let groups = ::std::slice::from_raw_parts(ptr, n as usize);
        let ptr = ptr.offset(n as isize);
        let lengths = ::std::slice::from_raw_parts(ptr, n as usize);
        let ptr = ptr.offset(n as isize);
        let table = ::std::slice::from_raw_parts(ptr, ::std::u16::MAX as usize);
        CompiledCharTable {
            n_categories: n,
            default_id: default_id,
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
        default_id: 0,
        categories: vec![Category {
                             invoke: true,
                             group: false,
                             length: 0,
                         },
                         Category {
                             invoke: false,
                             group: true,
                             length: 1,
                         },
                         Category {
                             invoke: true,
                             group: false,
                             length: 2,
                         }],
        table: [0; ::std::u16::MAX as usize],
    };
    table.table['あ' as usize] = 1;
    table.table['a' as usize] = 2;

    let mut buf = Vec::new();
    table.encode(&mut buf).unwrap();

    let compiled = unsafe { CompiledCharTable::decode(&buf) };

    let tests = vec!['0', 'あ', 'a'];

    for ch in tests {
        let category = compiled.categorize(ch);
        assert_eq!(category, table.categorize(ch));
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[test]
fn test_entry_encode() {
    let e = Entry {
        left_id: 0,
        right_id: 1,
        weight: -1,
        contents: "てすと",
    };
    let mut buf = Vec::new();
    e.encode_native(&mut buf).unwrap();
    let actual = unsafe { Entry::decode(&buf) };
    assert_eq!(actual, e);
}

pub trait UnknownDic: CharCategorize {
    fn fetch_entries<'a>(&'a self, cate: CategoryId) -> Vec<Entry<'a>>;
}

pub struct UnkDic {
    pub indices: Vec<u32>, // Category -> Index of initial entry
    pub counts: Vec<u32>, // Category -> Counts of entries
    pub entry_offsets: Vec<u32>,
    pub entries: Vec<u8>,
    pub categories: CharTable,
}

impl CharCategorize for UnkDic {
    fn categorize(&self, ch: char) -> Category {
        self.categories.categorize(ch)
    }

    fn category_id(&self, ch: char) -> CategoryId {
        self.categories.category_id(ch)
    }
}

impl UnknownDic for UnkDic {
    fn fetch_entries<'a>(&'a self, cate: CategoryId) -> Vec<Entry<'a>> {
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

impl UnkDic {
    pub fn build<'a>(entries: HashMap<CategoryId, Vec<Entry<'a>>>, char_table: CharTable) -> Self {
        let n_cates = entries.len();
        let mut indices = vec![0; n_cates];
        let mut counts = vec![0; n_cates];
        let mut offsets = Vec::new();
        let mut entry_buf = Vec::new();
        let mut index = 0;
        for (id, entries) in entries {
            indices[id as usize] = index;
            counts[id as usize] = entries.len() as u32;
            for entry in entries {
                let offset = entry_buf.len() as u32;
                offsets.push(offset);
                index += 1;
                entry.encode_native(&mut entry_buf).unwrap();
            }
        }
        UnkDic {
            indices: indices,
            counts: counts,
            entry_offsets: offsets,
            entries: entry_buf,
            categories: char_table,
        }
    }

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

#[test]
fn test_unk_dic() {
    let stub_char_table = CharTable::new(0, Vec::new());
    let mut entries = HashMap::new();
    let es = vec!["a", "b"]
        .into_iter()
        .map(|s| {
            Entry {
                left_id: 0,
                right_id: 1,
                weight: -1,
                contents: s,
            }
        })
        .collect::<Vec<_>>();
    entries.insert(0, es.clone());
    entries.insert(1, es.clone());
    let dic = UnkDic::build(entries, stub_char_table);
    assert_eq!(dic.fetch_entries(0), es);
    assert_eq!(dic.fetch_entries(1), es);
}

pub struct CompiledUnkDic<'a> {
    indices: &'a [u32],
    counts: &'a [u32],
    entry_offsets: &'a [u32],
    entries: &'a [u8],
    categories: CompiledCharTable<'a>,
}

impl<'a> CharCategorize for CompiledUnkDic<'a> {
    fn categorize(&self, ch: char) -> Category {
        self.categories.categorize(ch)
    }

    fn category_id(&self, ch: char) -> CategoryId {
        self.categories.category_id(ch)
    }
}

impl<'a> UnknownDic for CompiledUnkDic<'a> {
    fn fetch_entries<'b>(&'b self, cate: CategoryId) -> Vec<Entry<'b>> {
        let count = self.counts[cate as usize] as usize;
        let index = self.indices[cate as usize] as usize;
        let offsets = &self.entry_offsets[index..index + count];
        let mut results = Vec::with_capacity(count);
        for &offset in offsets {
            let e = unsafe { Entry::decode(&self.entries[offset as usize..]) };
            results.push(e);
        }
        results
    }
}

impl<'a> CompiledUnkDic<'a> {
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

        CompiledUnkDic {
            indices: indices,
            counts: counts,
            entry_offsets: entry_offsets,
            entries: entries,
            categories: categories,
        }
    }
}

#[test]
fn test_unk_dic_encode() {
    let stub_char_table = CharTable::new(0, Vec::new());
    let mut entries = HashMap::new();
    let es = vec!["a", "b"]
        .into_iter()
        .map(|s| {
            Entry {
                left_id: 0,
                right_id: 1,
                weight: -1,
                contents: s,
            }
        })
        .collect::<Vec<_>>();
    entries.insert(0, es.clone());
    entries.insert(1, es.clone());
    entries.insert(2, es.clone());
    let dic = UnkDic::build(entries, stub_char_table);
    let mut buf = Vec::new();
    dic.encode_native(&mut buf).unwrap();
    let compiled = unsafe { CompiledUnkDic::decode(&buf) };
    assert_eq!(dic.fetch_entries(0), compiled.fetch_entries(0));
    assert_eq!(dic.fetch_entries(1), compiled.fetch_entries(1));
    assert_eq!(dic.fetch_entries(2), compiled.fetch_entries(2));
}
