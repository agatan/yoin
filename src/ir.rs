use std::collections::HashMap;

use mast::*;

#[derive(Debug, Clone)]
pub struct StateIr {
    id: StateId,
    iseq: Vec<Ir>,
}

pub type IrTable = HashMap<StateId, StateIr>;

#[derive(Debug, Clone)]
pub enum Ir {
    /// output bytes and jump to state_id if ch matches input.
    Output {
        ch: u8,
        state_id: StateId,
        bytes: Vec<u8>,
    },
    /// same as `Output`, but break if not matched.
    OutBreak { ch: u8 },
    /// add offset to pc if ch matches input.
    Jump { ch: u8, state_id: StateId },
    /// accept and stop execution
    Accept,
    /// accept with tail output set
    AcceptWith(Vec<Vec<u8>>),
}