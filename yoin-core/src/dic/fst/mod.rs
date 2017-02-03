use std::convert::AsRef;
use std::iter::IntoIterator;

mod mast;
mod op;

#[derive(Debug, Clone)]
pub struct Fst<T>
    where T: AsRef<[u8]>
{
    bytecode: T,
}

impl<'a> Fst<&'a [u8]> {
    pub unsafe fn from_bytes(bytes: &'a [u8]) -> Self {
        Fst { bytecode: bytes }
    }
}

impl<T: AsRef<[u8]>> Fst<T> {
    pub fn run_iter<'a>(&'a self, input: &'a [u8]) -> Iter<'a> {
        Iter::new(self.bytecode.as_ref(), input)
    }

    pub fn run<'a>(&'a self, input: &'a [u8]) -> Vec<Accept> {
        self.run_iter(input).collect()
    }

    pub fn bytecode<'a>(&'a self) -> &'a [u8] {
        self.bytecode.as_ref()
    }
}

impl Fst<Vec<u8>> {
    pub fn build<'a, I: IntoIterator<Item = (&'a [u8], u32)>>(inputs: I) -> Self {
        let m = mast::Mast::build(inputs);
        Fst { bytecode: op::build(m) }
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a> {
    pc: usize,
    iseq: &'a [u8],
    input: &'a [u8],
    len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Accept(pub u32);

impl<'a> Iter<'a> {
    pub fn new(iseq: &'a [u8], input: &'a [u8]) -> Self {
        Iter {
            pc: 0,
            iseq: iseq,
            input: input,
            len: 0,
        }
    }

    fn read_u16(&mut self) -> u16 {
        let from = self.iseq[self.pc..].as_ptr() as *const u16;
        self.pc += 2; // skip 16 bits
        unsafe { *from }
    }

    fn read_u32(&mut self) -> u32 {
        let from = self.iseq[self.pc..].as_ptr() as *const u32;
        self.pc += 4; // skip 32 bits
        unsafe { *from }
    }

    fn get_jump_offset(&mut self, jump_size: u8) -> usize {
        if jump_size == op::JUMP_SIZE_16 {
            self.read_u16() as usize
        } else {
            debug_assert!(jump_size == op::JUMP_SIZE_32, "invalid bytecode");
            self.read_u32() as usize
        }
    }

    fn run_jump(&mut self) {
        let op = op::Op(self.iseq[self.pc]);
        self.pc += 1;
        let cmp = self.iseq[self.pc];
        self.pc += 1;

        let jump = self.get_jump_offset(op.jump_bytes());
        if cmp != self.input[self.len] {
            return;
        }
        self.len += 1;
        self.pc += jump;
    }

    fn run_outjump(&mut self) -> Option<u32> {
        let op = op::Op(self.iseq[self.pc]);
        self.pc += 1;
        let cmp = self.iseq[self.pc];
        self.pc += 1;
        let jump = self.get_jump_offset(op.jump_bytes());
        if cmp != self.input[self.len] {
            self.pc += 4 as usize; // skip unused data bytes.
            return None;
        }
        self.len += 1;
        let n = self.read_u32();
        self.pc += jump - 4 as usize;
        Some(n)
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = Accept;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let op = op::Op(self.iseq[self.pc]);
            match op.code() {
                op::OPCODE_BREAK => return None,
                op::OPCODE_JUMP => {
                    if self.len >= self.input.len() {
                        return None;
                    }
                    self.run_jump();
                }
                op::OPCODE_OUTJUMP => {
                    if self.len >= self.input.len() {
                        return None;
                    }
                    match self.run_outjump() {
                        None => (),
                        Some(n) => return Some(Accept(n)),
                    }
                }
                op::OPCODE_ACCEPT_WITH => {
                    self.pc += 1; // skip op::OPCODE_ACCEPT_WITH
                    let n = self.read_u32();
                    let accept = Accept(n);
                    return Some(accept);
                }
                op => unreachable!("unknown operator in bytecode: {:?}", op),
            }
        }
    }
}

#[test]
fn test_run() {
    use std::collections::HashSet;

    let samples: Vec<(&[u8], u32)> = vec![(b"ab", 0xFF), (b"abc", 0), (b"abc", !0), (b"abd", 1)];
    let iseq = Fst::build(samples);
    let accs: HashSet<_> = iseq.run(b"abc").into_iter().collect();
    let expects: HashSet<_> = vec![Accept(0xFF), Accept(0), Accept(!0)]
        .into_iter()
        .collect();
    assert_eq!(accs, expects);
}

#[test]
fn test_op() {
    use std::collections::HashSet;
    let samples: Vec<(&[u8], u32)> = vec![(b"apr", 0),
                                          (b"aug", 1),
                                          (b"dec", 2),
                                          (b"feb", 3),
                                          (b"feb", 4),
                                          (b"feb'", 8),
                                          (b"jan", 5),
                                          (b"jul", 6),
                                          (b"jun", 7)];
    let iseq = Fst::build(samples);
    let expected =
        vec![Accept(3), Accept(4), Accept(8)]
            .into_iter()
            .collect();
    assert_eq!(iseq.run_iter(b"feb'").collect::<HashSet<_>>(), expected);
}
