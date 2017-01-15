use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StateHash(i32);

#[derive(Debug, Clone, PartialEq, Eq)]
struct State {
    is_final: bool,
    trans: HashMap<u8, Rc<RefCell<State>>>,
    output: HashMap<u8, Vec<u8>>,
    state_output: HashSet<Vec<u8>>,
    hash_code: StateHash,
}

impl State {
    fn new() -> State {
        State {
            is_final: false,
            trans: HashMap::new(),
            output: HashMap::new(),
            state_output: HashSet::new(),
            hash_code: StateHash(0),
        }
    }
    fn transition(&self, c: u8) -> Option<Rc<RefCell<State>>> {
        self.trans.get(&c).cloned()
    }

    fn set_transition(&mut self, c: u8, to: Rc<RefCell<State>>) {
        self.trans.insert(c, to);
    }

    fn output(&self, c: u8) -> Option<&[u8]> {
        self.output.get(&c).map(|x| x.as_slice())
    }

    fn set_output(&mut self, c: u8, out: Vec<u8>) {
        self.output.insert(c, out);
    }

    fn set_state_output(&mut self, outs: HashSet<Vec<u8>>) {
        self.state_output = outs;
    }

    fn clear(&mut self) {
        self.is_final = false;
        self.trans.clear();
        self.output.clear();
    }
}

struct StateTable {
    table: HashMap<StateHash, Vec<Rc<RefCell<State>>>>,
}

impl StateTable {
    fn new() -> Self {
        StateTable { table: HashMap::new() }
    }

    fn get(&self, state: &Rc<RefCell<State>>) -> Option<Rc<RefCell<State>>> {
        let h = state.borrow().hash_code;
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
        let h = state.borrow().hash_code;
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
                let r = Rc::new(RefCell::new(state.borrow().clone()));
                self.insert(r.clone());
                r
            }
        }
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

fn create_minimal_transducer(pairs: Vec<(&[u8], i32)>) {
    let mut table = StateTable::new();
    let mut buf: Vec<Rc<RefCell<State>>> = Vec::new();
    let mut prev_word: &[u8] = b"";
    let mut chars = HashSet::new();
    let mut last_input: &[u8] = b"";
    for (input, output) in pairs {
        // hold the last input.
        last_input = input;
        // setup
        let current_output: [u8; 4] = unsafe { ::std::mem::transmute(output) };
        let mut current_output: &[u8] = &current_output;
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
            s.set_transition(prev_word[i-1], table.find_minimized(target));
        }
        for i in (prefix_len+1)..input.len() {
            buf[i].borrow_mut().clear();
            buf[i - 1].borrow_mut().set_transition(input[i], buf[i].clone());
        }

        if input != prev_word {
            buf[input.len()].borrow_mut().is_final = true;
            let mut outs = HashSet::new();
            outs.insert(Vec::new());
            buf[input.len()].borrow_mut().set_state_output(outs);
        }

        for j in 1..(prefix_len+1) {
            let output = match buf[j - 1].borrow().output(input[j-1]) {
                Some(output) => Vec::from(output),
                None => Vec::new(),
            };
            let mut common_prefix = Vec::new();
            for (current_out, out) in current_output.iter().zip(output.iter()) {
                if current_out != out {
                    break;
                }
                common_prefix.push(*current_out);
            }
            let common_prefix_len = common_prefix.len();
            let word_suffix = &output[common_prefix_len..];
            buf[j - 1].borrow_mut().set_output(input[j-1], common_prefix);

            for &c in chars.iter() {
                if buf[j].borrow().transition(c).is_some() {
                    let mut new_output = Vec::from(word_suffix);
                    if let Some(os) = buf[j].borrow().output(c) {
                        new_output.extend_from_slice(os);
                    }
                    buf[j].borrow_mut().set_output(c, new_output);
                }
            }

            if buf[j].borrow().is_final {
                let mut temp_set = HashSet::new();
                for temp_str in buf[j].borrow().state_output.iter() {
                    let mut new_output = Vec::from(word_suffix);
                    new_output.extend_from_slice(temp_str);
                    temp_set.insert(new_output);
                }
                buf[j].borrow_mut().set_state_output(temp_set);
            }

            current_output = &current_output[common_prefix_len..];
        }

        if input == prev_word {
            buf[input.len()].borrow_mut().state_output.insert(Vec::from(current_output));
        } else {
            buf[prefix_len].borrow_mut().set_output(input[prefix_len], Vec::from(current_output));
        }
        prev_word = input;
    }

    // minimize the last word
    for i in (0..last_input.len()).map(|x| x + 1).rev() {
        let target = buf[i].clone();
        let mut s = buf[i - 1].borrow_mut();
        s.set_transition(prev_word[i-1], table.find_minimized(target));
    }

    let initial_state = table.find_minimized(buf[0].clone());
}

fn main() {
    let samples: Vec<(&[u8], _)> = vec![
        (b"hello", 111),
        (b"hello", 112),
        (b"hallo", 222),
    ];
    create_minimal_transducer(samples);
}