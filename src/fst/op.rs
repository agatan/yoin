use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashSet, HashMap};
use std::ops;
use std::iter::Iterator;

use byteorder::{BigEndian, WriteBytesExt};

use super::mast::{State, Mast, StateId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Op(pub u8);

const OPCODE_MASK: Op = Op(0b111_00000);

/// OUTJUMP: op | jump | data, ch, jump..., data...
pub const OPCODE_OUTJUMP: Op = Op(0b000_00000);
/// JUMP: op | jump, ch, jump...
pub const OPCODE_JUMP: Op = Op(0b001_00000);
pub const OPCODE_BREAK: Op = Op(0b010_00000);
pub const OPCODE_ACCEPT: Op = Op(0b011_00000);
pub const OPCODE_ACCEPT_WITH: Op = Op(0b100_00000);

pub const JUMP_SIZE_MASK: Op = Op(0b000_11_000);

pub const DATA_SIZE_MASK: Op = Op(0b000_00_111);

pub const JUMP_SIZE_OFFSET: u8 = 3;
pub const JUMP_SIZE_16: u8 = 0;
pub const JUMP_SIZE_32: u8 = 1;

impl ops::BitOr for Op {
    type Output = Self;
    fn bitor(self, rhs: Op) -> Self {
        Op(self.0 | rhs.0)
    }
}

impl ops::BitOrAssign for Op {
    fn bitor_assign(&mut self, rhs: Op) {
        self.0 |= rhs.0
    }
}

impl ops::BitAnd for Op {
    type Output = Self;
    fn bitand(self, rhs: Op) -> Self {
        Op(self.0 & rhs.0)
    }
}

impl ops::BitAndAssign for Op {
    fn bitand_assign(&mut self, rhs: Op) {
        self.0 &= rhs.0
    }
}

impl ops::BitOr<u8> for Op {
    type Output = Self;
    fn bitor(self, rhs: u8) -> Self {
        Op(self.0 | rhs)
    }
}

impl ops::BitOrAssign<u8> for Op {
    fn bitor_assign(&mut self, rhs: u8) {
        self.0 |= rhs
    }
}

impl Op {
    pub fn code(self) -> Op {
        self & OPCODE_MASK
    }

    pub fn jump_bytes(self) -> u8 {
        (self & JUMP_SIZE_MASK).0 >> JUMP_SIZE_OFFSET
    }

    fn with_jump_bytes(mut self, size: u8) -> Op {
        self.0 |= (size << JUMP_SIZE_OFFSET) & JUMP_SIZE_MASK.0;
        self
    }

    pub fn data_bytes(self) -> u8 {
        self.0 & DATA_SIZE_MASK.0
    }

    fn with_data_bytes(mut self, size: u8) -> Op {
        self.0 |= size & DATA_SIZE_MASK.0;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct Compiler {
    rev_bytes: Vec<u8>,
    jump_table: HashMap<StateId, usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler::default()
    }

    fn compile_jump_offset(&mut self, to: usize) -> u8 {
        // FIXME(agatan): is the offset correct?
        let jump = self.rev_bytes.len() - 1 - to;
        if jump < ::std::u16::MAX as usize {
            self.rev_bytes.write_u16::<BigEndian>(jump as u16).unwrap();
            JUMP_SIZE_16
        } else {
            self.rev_bytes.write_u32::<BigEndian>(jump as u32).unwrap();
            JUMP_SIZE_32
        }
    }

    fn compile_output(&mut self, data: &[u8]) -> u8 {
        let data_size = data.len();
        debug_assert!(data_size <= 4);
        for d in data.iter().rev() {
            self.rev_bytes.push(*d);
        }
        data_size as u8
    }

    fn compile_outjump(&mut self, ch: u8, to: usize, data: &[u8]) {
        let data_size = self.compile_output(data);
        let jump_size = self.compile_jump_offset(to);
        let op = OPCODE_OUTJUMP.with_data_bytes(data_size).with_jump_bytes(jump_size);
        self.rev_bytes.push(ch);
        self.rev_bytes.push(op.0);
    }

    fn compile_jump(&mut self, ch: u8, to: usize) {
        let jump_size = self.compile_jump_offset(to);
        let op = OPCODE_JUMP.with_jump_bytes(jump_size);
        self.rev_bytes.push(ch);
        self.rev_bytes.push(op.0);
    }

    fn compile_accept_with(&mut self, data: &[u8]) {
        debug_assert!(data.len() <= 4);
        let op = OPCODE_ACCEPT_WITH.with_data_bytes(data.len() as u8);
        for d in data.iter().rev() {
            self.rev_bytes.push(*d);
        }
        self.rev_bytes.push(op.0);
    }

    fn compile_transition(&mut self, from: &State, ch: u8, to: &State) {
        let to_pos = self.jump_table[&to.id];
        match from.output(ch) {
            Some(out) if !out.is_empty() => self.compile_outjump(ch, to_pos, out),
            _ => self.compile_jump(ch, to_pos),
        }
    }

    fn compile_state(&mut self, state: &State) {
        self.rev_bytes.push(OPCODE_BREAK.0);
        for (&ch, to) in state.trans.iter() {
            self.compile_transition(state, ch, &*to.borrow());
        }
        if state.is_final {
            for output in state.state_output.iter() {
                if output.is_empty() {
                    self.rev_bytes.push(OPCODE_ACCEPT.0);
                } else {
                    self.compile_accept_with(output);
                }
            }
        }
        self.jump_table.insert(state.id, self.rev_bytes.len() - 1);
    }

    pub fn compile(&mut self, mast: Mast) {
        let rev_sorted = rev_topological_sort(&mast);

        for state in rev_sorted {
            self.compile_state(&*state.borrow());
        }
    }

    pub fn iseq(&self) -> Vec<u8> {
        self.rev_bytes.iter().rev().cloned().collect()
    }
}

fn rev_topological_sort(mast: &Mast) -> Vec<Rc<RefCell<State>>> {
    fn visit(visited: &mut HashSet<StateId>,
             rev_results: &mut Vec<Rc<RefCell<State>>>,
             state: &Rc<RefCell<State>>) {
        if visited.contains(&state.borrow().id) {
            return;
        }
        visited.insert(state.borrow().id);
        for to in state.borrow().trans.values() {
            visit(visited, rev_results, to);
        }
        rev_results.push(state.clone());
    }

    let mut rev_results = Vec::new();
    let mut visited = HashSet::new();
    for state in mast.states.iter() {
        visit(&mut visited, &mut rev_results, state);
    }
    rev_results
}

pub fn build(mast: Mast) -> Vec<u8> {
    let mut compiler = Compiler::new();
    compiler.compile(mast);
    compiler.iseq()
}
