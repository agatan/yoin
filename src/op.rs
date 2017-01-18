use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashSet, HashMap};

use byteorder::{LittleEndian, WriteBytesExt};

use mast::{State, Mast, StateId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    /// Output variadic bytes. parameters: character, offset, size, data, data, ...
    Output,
    /// Jump if matched. parameter: character, offset.
    Jump,
    /// Stop execution
    Break,
    /// Accept curreny data.
    Accept,
    /// Accept with variadic data. parameters: size, data, data, ...
    AcceptWith,
}

impl Opcode {
    fn jump(rev_bytes: &mut Vec<u8>, ch: u8, to: u32) {
        let bytes_size = rev_bytes.len() as u32;
        let jump_offset = bytes_size - to;
        rev_bytes.write_u32::<LittleEndian>(jump_offset).unwrap();
        rev_bytes.push(ch);
        rev_bytes.push(Opcode::Jump as u8);
    }

    fn output(rev_bytes: &mut Vec<u8>, ch: u8, to: u32, data: &[u8]) {
        let size = data.len() as u32;
        for b in data.iter().rev() {
            rev_bytes.push(*b);
        }
        rev_bytes.write_u32::<LittleEndian>(size).unwrap();
        Self::jump(rev_bytes, ch, to);
    }

    fn accept_with(rev_bytes: &mut Vec<u8>, data: &[u8]) {
        let size = data.len() as u32;
        for b in data.iter().rev() {
            rev_bytes.push(*b);
        }
        rev_bytes.write_u32::<LittleEndian>(size).unwrap();
        rev_bytes.push(Opcode::AcceptWith as u8);
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

fn build_transition(rev_bytes: &mut Vec<u8>, jump_table: &HashMap<StateId, u32>, from: &State, ch: u8, to: &State) {
    let to_id = to.id;
    let to_pos = jump_table[&to_id];
    match from.output(ch) {
        Some(out) if !out.is_empty() => {
            Opcode::output(rev_bytes, ch, to_pos, out)
        },
        _ => Opcode::jump(rev_bytes, ch, to_pos),
    }
}

fn build_state(rev_bytes: &mut Vec<u8>, jump_table: &mut HashMap<StateId, u32>, state: &State) {
    rev_bytes.push(Opcode::Break as u8);
    for (&ch, to) in state.trans.iter() {
        build_transition(rev_bytes, jump_table, state, ch, &*to.borrow());
    }
    if state.is_final {
        if state.state_output.is_empty() {
            rev_bytes.push(Opcode::Accept as u8);
        } else {
            for output in state.state_output.iter() {
                Opcode::accept_with(rev_bytes, output);
            }
        }
    }
    jump_table.insert(state.id, rev_bytes.len() as u32 - 1);
}

pub fn build(mast: Mast) -> Vec<u8> {
    let mut rev_bytes = Vec::new();
    let mut jump_table = HashMap::<StateId, u32>::new();
    let rev_sorted = rev_topological_sort(&mast);

    for state in rev_sorted {
        build_state(&mut rev_bytes, &mut jump_table, &*state.borrow());
    }
    rev_bytes.reverse();
    rev_bytes
}