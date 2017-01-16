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

pub fn build(mast: &Mast) -> IrTable {
    let mut table = IrTable::new();
    for state in mast.states.iter() {
        let ir = StateIr::new(&*state.borrow());
        let id = state.borrow().id;
        table.insert(id, ir);
    }
    table
}

pub fn run(mast: &Mast, input: &[u8]) -> Result<Vec<i32>, String> {
    fn bytes_to_i32(bytes: &[u8]) -> Result<i32, String> {
        if bytes.len() != 4 {
            Err(format!("output byte length is not 4, got {}", bytes.len()))
        } else {
            let ptr: *const i32 = unsafe { ::std::mem::transmute(bytes.as_ptr()) };
            let i = unsafe { *ptr };
            Ok(i)
        }
    }
    let ir_table = build(mast);
    let mut state_ir = &ir_table[&mast.initial_state_id()];
    let mut data = Vec::new();
    let mut i = 0;
    loop {
        for ir in state_ir.iseq.iter() {
            match *ir {
                Ir::Accept => return bytes_to_i32(&data).map(|i| vec![i]),
                Ir::AcceptWith(ref tails) => {
                    return tails.iter()
                        .map(|tail| {
                            let mut buf = data.clone();
                            buf.extend_from_slice(tail);
                            bytes_to_i32(&buf)
                        })
                        .collect()
                }
                Ir::Break => return Err("input does not match".to_string()),
                Ir::Jump { ch, ref state_id } => {
                    if ch == input[i] {
                        i += 1;
                        state_ir = &ir_table[state_id];
                        break;
                    }
                }
                Ir::Output { ch, ref state_id, ref bytes } => {
                    if ch == input[i] {
                        i += 1;
                        data.extend_from_slice(bytes);
                        state_ir = &ir_table[state_id];
                        break;
                    }
                }
            }
        }
    }
}

#[test]
fn test_run() {
    use std::collections::HashSet;

    let samples: Vec<(&[u8], [u8; 4])> = vec![(b"apr", [0, 0, 3, 0]),
                                              (b"aug", [0, 0, 3, 1]),
                                              (b"dec", [0, 0, 3, 1]),
                                              (b"feb", [0, 0, 2, 8]),
                                              (b"feb", [0, 0, 2, 9]),
                                              (b"feba", [0, 0, 3, 1]),
                                              (b"jul", [0, 0, 3, 0]),
                                              (b"jun", [0, 0, 3, 1])];
    let samples = samples.into_iter()
        .map(|(x, bytes)| {
            let out: i32 = unsafe { ::std::mem::transmute(bytes) };
            (x, out)
        });
    let m = Mast::build(samples);

    let tests: Vec<(&[u8], _)> = vec![(b"feb", vec![[0, 0, 2, 8], [0, 0, 2, 9]]),
                                      (b"feba", vec![[0, 0, 3, 1]])];

    for (input, expected) in tests {
        let out: HashSet<_> = run(&m, input)
            .unwrap()
            .into_iter()
            .map(|out| unsafe { ::std::mem::transmute::<i32, [u8; 4]>(out) })
            .collect();
        let expected_set = expected.into_iter().collect();
        assert_eq!(out, expected_set);
    }
}