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
    /// add offset to pc if ch matches input.
    Jump { ch: u8, state_id: StateId },
    /// accept and stop execution
    Accept,
    /// accept with tail output set
    AcceptWith(Vec<Vec<u8>>),
    /// interrupt execution and return
    Break,
}

impl Ir {
    fn from_transition(from: &State, ch: u8, to: &State) -> Self {
        let to_id = to.id;
        match from.output(ch) {
            Some(out) if !out.is_empty() => {
                let out = Vec::from(out);
                Ir::Output {
                    ch: ch,
                    state_id: to_id,
                    bytes: out,
                }
            }
            _ => {
                Ir::Jump {
                    ch: ch,
                    state_id: to_id,
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateIr {
    id: StateId,
    iseq: Vec<Ir>,
}

impl StateIr {
    fn new(state: &State) -> StateIr {
        let mut iseq = Vec::new();
        if state.is_final {
            if state.state_output.is_empty() {
                iseq.push(Ir::Accept);
            } else {
                let outputs = state.state_output.iter().cloned().collect();
                iseq.push(Ir::AcceptWith(outputs));
            }
        } else {
            for (&ch, to) in state.trans.iter() {
                iseq.push(Ir::from_transition(state, ch, &*to.borrow()));
            }
            iseq.push(Ir::Break);
        }
        StateIr {
            id: state.id,
            iseq: iseq,
        }
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