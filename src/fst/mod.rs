use std::convert::AsRef;
use std::iter::IntoIterator;
use byteorder::{LittleEndian, ReadBytesExt};

mod mast;
mod op;

#[derive(Debug, Clone)]
pub struct Fst<T> where T: AsRef<[u8]> {
    bytecode: T,
}

impl<'a> Fst<&'a [u8]> {
    pub unsafe fn from_bytes(bytes: &'a [u8]) -> Self {
        Fst {
            bytecode: bytes,
        }
    }
}

impl<T: AsRef<[u8]>> Fst<T> {
    pub fn run_iter<'a>(&'a self, input: &'a [u8]) -> FstIter<'a> {
        FstIter::new(self.bytecode.as_ref(), input)
    }

    pub fn run<'a>(&'a self, input: &'a [u8]) -> Result<Vec<Accept>, String> {
        self.run_iter(input).collect()
    }

    pub fn bytecode<'a>(&'a self) -> &'a [u8] {
        self.bytecode.as_ref()
    }
}

impl Fst<Vec<u8>> {
    pub fn build<'a, I: IntoIterator<Item=(&'a [u8], i32)>>(inputs: I) -> Self {
        let m = mast::Mast::build(inputs);
        Fst {
            bytecode: op::build(m),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FstIter<'a> {
    pc: usize,
    iseq: &'a [u8],
    data: [u8; 4],
    data_len: u8,
    input: &'a [u8],
    len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Accept {
    pub len: usize,
    pub value: i32,
}

impl<'a> FstIter<'a> {
    pub fn new(iseq: &'a [u8], input: &'a [u8]) -> Self {
        FstIter {
            pc: 0,
            iseq: iseq,
            data: [0; 4],
            data_len: 0,
            input: input,
            len: 0,
        }
    }

    fn read_u16(&mut self) -> u16 {
        let mut from = &self.iseq[self.pc..];
        self.pc += 2; // skip 16 bits
        from.read_u16::<LittleEndian>().unwrap()
    }

    fn read_u32(&mut self) -> u32 {
        let mut from = &self.iseq[self.pc..];
        self.pc += 4; // skip 32 bits
        from.read_u32::<LittleEndian>().unwrap()
    }

    fn get_jump_offset(&mut self, jump_size: u8) -> Result<usize, String> {
        let jump = if jump_size == op::JUMP_SIZE_16 {
            self.read_u16() as usize
        } else if jump_size == op::JUMP_SIZE_32 {
            self.read_u32() as usize
        } else {
            return Err(format!("jump size is ill-formed: {}", jump_size));
        };
        Ok(jump)
    }

    fn run_jump(&mut self) -> Result<(), String> {
        let op = op::Op(self.iseq[self.pc]);
        self.pc += 1;
        let cmp = self.iseq[self.pc];
        self.pc += 1;

        let jump = self.get_jump_offset(op.jump_bytes())?;
        if cmp != self.input[self.len] {
            return Ok(());
        }
        self.len += 1;
        self.pc += jump;
        Ok(())
    }

    fn run_outjump(&mut self) -> Result<(), String> {
        let op = op::Op(self.iseq[self.pc]);
        self.pc += 1;
        let cmp = self.iseq[self.pc];
        self.pc += 1;
        let jump = self.get_jump_offset(op.jump_bytes())?;
        if cmp != self.input[self.len] {
            self.pc += op.data_bytes() as usize; // skip unused data bytes.
            return Ok(());
        }
        self.len += 1;
        for _ in 0..op.data_bytes() {
            debug_assert!(self.data_len < 4, "output data is not 4 bytes");
            self.data[self.data_len as usize] = self.iseq[self.pc];
            self.data_len += 1;
            self.pc += 1;
        }
        self.pc += jump - op.data_bytes() as usize;
        Ok(())
    }
}

impl<'a> Iterator for FstIter<'a> {
    type Item = Result<Accept, String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let op = op::Op(self.iseq[self.pc]);
            match op.code() {
                op::OPCODE_BREAK => return None,
                op::OPCODE_JUMP => {
                    if self.len >= self.input.len() {
                        return None;
                    }
                    match self.run_jump() {
                        Ok(()) => (),
                        Err(err) => return Some(Err(err)),
                    }
                }
                op::OPCODE_OUTJUMP => {
                    if self.len >= self.input.len() {
                        return None;
                    }
                    match self.run_outjump() {
                        Ok(()) => (),
                        Err(err) => return Some(Err(err)),
                    }
                }
                op::OPCODE_ACCEPT => {
                    self.pc += 1;
                    debug_assert!(self.data_len == 4);
                    let value = gen_data(&self.data);
                    let accept = Accept {
                        len: self.len,
                        value: value,
                    };
                    return Some(Ok(accept));
                }
                op::OPCODE_ACCEPT_WITH => {
                    let save = self.data_len;
                    self.pc += 1; // skip op::OPCODE_ACCEPT_WITH
                    for _ in 0..op.data_bytes() {
                        debug_assert!(self.data_len < 4);
                        self.data[self.data_len as usize] = self.iseq[self.pc];
                        self.pc += 1;
                        self.data_len += 1;
                    }
                    debug_assert!(self.data_len == 4);
                    let value = gen_data(&self.data);
                    let accept = Accept {
                        len: self.len,
                        value: value,
                    };
                    self.data_len = save;
                    return Some(Ok(accept));
                }
                op => return Some(Err(format!("unknown op code: {:08b}", op.0))),
            }
        }
    }
}

fn gen_data(data: &[u8; 4]) -> i32 {
    let mut from: &[u8] = data;
    from.read_i32::<LittleEndian>().unwrap()
}
