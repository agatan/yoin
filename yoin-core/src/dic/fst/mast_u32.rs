use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StateHash(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StateId(usize);

pub const DUMMY_STATE_ID: StateId = StateId(!0);

const TRANS_MAGIC: i32 = 16381;
const OUTPUT_MAGIC: i32 = 8191;

#[derive(Debug, Clone, Eq)]
pub struct State {
    pub id: StateId,
    pub is_final: bool,
    pub trans: HashMap<u8, Rc<RefCell<State>>>,
    pub output: HashMap<u8, u32>,
    pub state_output: Vec<u32>,
    hash_code_mem: Cell<Option<StateHash>>,
}

impl ::std::cmp::PartialEq<State> for State {
    fn eq(&self, other: &State) -> bool {
        self.is_final == other.is_final && self.trans == other.trans &&
        self.output == other.output && self.state_output == other.state_output &&
        self.hash_code() == other.hash_code()
    }
}

impl State {
    fn new() -> State {
        State {
            id: DUMMY_STATE_ID,
            is_final: false,
            trans: HashMap::new(),
            output: HashMap::new(),
            state_output: Vec::new(),
            hash_code_mem: Cell::new(None),
        }
    }
    fn transition(&self, c: u8) -> Option<Rc<RefCell<State>>> {
        self.trans.get(&c).cloned()
    }

    fn set_transition(&mut self, c: u8, to: Rc<RefCell<State>>) {
        self.trans.insert(c, to);
    }

    pub fn output(&self, c: u8) -> Option<u32> {
        self.output.get(&c).cloned()
    }

    fn set_output(&mut self, c: u8, out: u32) {
        self.output.insert(c, out);
    }

    fn add_state_output(&mut self, out: u32) {
        self.state_output.push(out);
    }

    fn set_state_output(&mut self, outs: Vec<u32>) {
        self.state_output = outs;
    }

    fn hash_code(&self) -> StateHash {
        match self.hash_code_mem.get() {
            Some(s) => s,
            None => {
                let mut code = 0i32;
                for (&c, &b) in &self.output {
                    code = code.wrapping_add((c as i32).wrapping_mul(OUTPUT_MAGIC));
                    code = code.wrapping_add((b as i32).wrapping_mul(OUTPUT_MAGIC));
                }
                for (&c, to) in &self.trans {
                    code = code.wrapping_add((c as i32).wrapping_mul(TRANS_MAGIC));
                    code = code.wrapping_add((to.borrow().id.0 as i32).wrapping_mul(TRANS_MAGIC));
                }
                self.hash_code_mem.set(Some(StateHash(code)));
                StateHash(code)
            }
        }
    }
}

struct StateTable {
    size: usize,
    table: HashMap<StateHash, Vec<Rc<RefCell<State>>>>,
}

impl StateTable {
    fn new() -> Self {
        StateTable {
            size: 0,
            table: HashMap::new(),
        }
    }

    fn get(&self, state: &Rc<RefCell<State>>) -> Option<Rc<RefCell<State>>> {
        let h = state.borrow().hash_code();
        match self.table.get(&h) {
            Some(ss) => {
                for s in ss {
                    if s == state {
                        return Some(s.clone());
                    }
                }
                None
            }
            None => None,
        }
    }

    fn insert(&mut self, state: Rc<RefCell<State>>) {
        self.size += 1;
        let h = state.borrow().hash_code();
        match self.table.entry(h) {
            Entry::Occupied(o) => {
                o.into_mut().push(state);
            }
            Entry::Vacant(v) => {
                v.insert(vec![state]);
            }
        }
    }

    fn find_minimized(&mut self, state: Rc<RefCell<State>>) -> Rc<RefCell<State>> {
        match self.get(&state) {
            Some(s) => s,
            None => {
                let mut s = state.borrow().clone();
                s.id = StateId(self.size);
                let r = Rc::new(RefCell::new(s));
                self.insert(r.clone());
                r
            }
        }
    }

    fn states(&self) -> Vec<Rc<RefCell<State>>> {
        let mut states = Vec::new();
        for (_, ss) in &self.table {
            for s in ss {
                states.push(s.clone());
            }
        }
        states
    }
}

fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
    let mut i = 0;
    let len = ::std::cmp::min(a.len(), b.len());
    while i < len && a[i] == b[i] {
        i += 1;
    }
    i
}

#[derive(Debug, Clone)]
pub struct Mast {
    pub initial: Rc<RefCell<State>>,
    pub states: Vec<Rc<RefCell<State>>>,
}


impl Mast {
    pub fn build<'a, I: IntoIterator<Item = (&'a [u8], u32)>>(pairs: I) -> Mast {
        let mut table = StateTable::new();
        let mut buf: Vec<Rc<RefCell<State>>> = Vec::new();
        let mut prev_word: &[u8] = b"";
        let mut chars = HashSet::new();
        let mut last_input: &[u8] = b"";

        for (input, current_output) in pairs {
            debug_assert!(input >= prev_word);
            // hold the last input.
            last_input = input;
            // setup
            while buf.len() <= input.len() {
                buf.push(Rc::new(RefCell::new(State::new())));
            }
            for c in input {
                chars.insert(*c);
            }

            let prefix_len = common_prefix_len(prev_word, input);
            for i in (prefix_len..prev_word.len()).map(|x| x + 1).rev() {
                let target = buf[i].clone();
                let mut s = buf[i - 1].borrow_mut();
                s.set_transition(prev_word[i - 1], table.find_minimized(target));
            }
            for i in (prefix_len + 1)..(input.len() + 1) {
                // buf[i].borrow_mut().clear();
                buf[i] = Rc::new(RefCell::new(State::new()));
                buf[i - 1].borrow_mut().set_transition(input[i - 1], buf[i].clone());
            }

            if input != prev_word {
                buf[input.len()].borrow_mut().is_final = true;
                buf[input.len()].borrow_mut().set_state_output(Vec::new());
            }

            let mut is_new_output = true;
            for j in 1..(prefix_len + 1) {
                let output = match buf[j - 1].borrow().output(input[j - 1]) {
                    Some(output) => output,
                    None => continue,
                };
                if current_output == output {
                    is_new_output = false;
                    break;
                }
                // common_prefix_len == 4 or 0
                // if cpl == 0 { remove output  }
                buf[j - 1].borrow_mut().output.remove(&input[j - 1]);

                for &c in chars.iter() {
                    if buf[j].borrow().transition(c).is_some() {
                        // if cpl == 0 { set output to output }
                        buf[j].borrow_mut().set_output(c, output);
                    }
                }

                if buf[j].borrow().is_final {
                    // if cpl == 0 { set state output }
                    buf[j].borrow_mut().add_state_output(output);
                }
            }

            if is_new_output {
                if input == prev_word {
                    buf[input.len()].borrow_mut().add_state_output(current_output);
                } else {
                    buf[prefix_len]
                        .borrow_mut()
                        .set_output(input[prefix_len], current_output);
                }
            }
            prev_word = input;
        }

        // minimize the last word
        for i in (0..last_input.len()).map(|x| x + 1).rev() {
            let target = buf[i].clone();
            let mut s = buf[i - 1].borrow_mut();
            s.set_transition(prev_word[i - 1], table.find_minimized(target));
        }

        let initial_state = table.find_minimized(buf[0].clone());

        Mast {
            initial: initial_state,
            states: table.states(),
        }
    }

    /// Print MAST as dot. (for debugging)
    #[allow(unused)]
    pub fn print_dot(&self) {
        let initial = &self.initial;
        let states = &self.states;
        println!("digraph G {{");
        println!("\trankdir=LR;");
        println!("\tnode [shape=circle]");
        for s in states {
            if s.borrow().is_final {
                println!("\t{:?} [peripheries = 2];", s.borrow().id.0);
            }
        }
        println!("\t{:?} [peripheries = 3];", initial.borrow().id.0);

        let mut stack = Vec::new();
        let mut done = StateTable::new();
        stack.push(initial.clone());
        while let Some(s) = stack.pop() {
            done.insert(s.clone());
            let state = s.borrow();
            for (c, to) in &state.trans {
                print!("\t{:?} -> {:?} [label=\"{}/{:?}",
                       state.id.0,
                       to.borrow().id.0,
                       *c as char,
                       state.output(*c));
                if !to.borrow().state_output.is_empty() {
                    print!(" {:?}", to.borrow().state_output);
                }
                println!("\"]");
                if done.get(&to).is_none() {
                    stack.push(to.clone());
                }
            }
        }
        println!("}}");
    }

    /// Run MAST directly. (for debugging)
    #[allow(unused)]
    pub fn run(&self, input: &[u8]) -> Result<Vec<u32>, String> {
        let mut state = self.initial.clone();
        let mut results = Vec::new();
        let mut buf = None;
        for &c in input {
            if let Some(o) = state.borrow().output(c) {
                debug_assert!(buf.is_none());
                buf = Some(o);
                results.push(o);
            }
            if state.borrow().is_final {
                for o in state.borrow().state_output.iter().cloned() {
                    results.push(o);
                }
            }
            let new_state = match state.borrow().transition(c) {
                Some(s) => s,
                None => return Err(format!("transition for {} not found", c as char)),
            };
            state = new_state;
        }
        if state.borrow().is_final {
            for o in state.borrow().state_output.iter().cloned() {
                results.push(o);
            }
        }
        Ok(results)
    }
}

#[test]
fn test_run() {
    use std::collections::HashSet;
    let samples: Vec<(&[u8], u32)> = vec![(b"a ", 0), (b"a ", 1), (b"a b ", 2), (b"a b c ", 3)];
    let m = Mast::build(samples);
    m.print_dot();
    let values = m.run("a b c ".as_bytes()).unwrap().into_iter().collect::<HashSet<_>>();
    assert_eq!(vec![0, 1, 2, 3].into_iter().collect::<HashSet<_>>(), values);
}
