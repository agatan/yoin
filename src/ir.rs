use std::collections::HashMap;

use mast::*;

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

#[derive(Debug, Clone)]
pub struct StateIr {
    id: StateId,
    iseq: Vec<Ir>,
}

impl StateIr {
    fn new(state: &State) -> StateIr {
        unimplemented!()
    }
}

pub type IrTable = HashMap<StateId, StateIr>;

#[derive(Debug, Clone)]
pub struct IrBuilder {
    table: IrTable,
}

impl IrBuilder {
    pub fn new() -> Self {
        IrBuilder { table: HashMap::new() }
    }

    pub fn build(mut self, mast: &Mast) -> IrTable {
        for state in mast.states.iter() {
            let ir = StateIr::new(&*state.borrow());
            let id = state.borrow().id;
            self.table.insert(id, ir);
        }
        self.table
    }
}