use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StateId(usize);

impl StateId {
    fn get(self, arena: &StateArena) -> &State {
        &arena[self]
    }

    fn get_mut(self, arena: &mut StateArena) -> &mut State {
        &mut arena[self]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StateHash(i64);

#[derive(Debug, Clone)]
struct State {
    id: StateId,
    trans: HashMap<u8, StateId>,
    is_final: bool,
    hash_code: StateHash,
}

impl State {
    fn new(id: StateId) -> Self {
        State {
            id: id,
            trans: HashMap::new(),
            is_final: false,
            hash_code: StateHash(0),
        }
    }
}

impl ::std::cmp::PartialEq<State> for State {
    fn eq(&self, other: &State) -> bool {
        self.trans == other.trans && self.is_final == other.is_final
    }
}

#[derive(Debug, Clone)]
struct StateArena(Vec<State>);

impl StateArena {
    fn new() -> Self {
        StateArena(Vec::new())
    }

    fn alloc(&mut self) -> StateId {
        let id = StateId(self.0.len());
        let state = State::new(id);
        self.0.push(state);
        id
    }
}

impl ::std::ops::Index<StateId> for StateArena {
    type Output = State;
    fn index(&self, index: StateId) -> &State {
        &self.0[index.0]
    }
}

impl ::std::ops::IndexMut<StateId> for StateArena {
    fn index_mut(&mut self, index: StateId) -> &mut State {
        &mut self.0[index.0]
    }
}

#[derive(Debug, Clone)]
struct Mast {
    arena: StateArena,
    states: Vec<StateId>,
    final_states: Vec<StateId>,
}

impl Mast {
    fn new() -> Mast {
        Mast {
            arena: StateArena::new(),
            states: Vec::new(),
            final_states: Vec::new(),
        }
    }

    fn add_state(&mut self, id: StateId) {
        self.states.push(id);
    }

    fn build<'a, I: IntoIterator<Item = (&'a [u8], i32)>>(sorted_iter: I) -> Self {
        let mut arena = StateArena::new();
        let mut state_dict: HashMap<StateHash, Vec<StateId>> = HashMap::new();
        let mut prev = b"";
        let mut buf = Vec::new();
        for (input, output) in sorted_iter.into_iter() {
            while buf.len() < input.len() {
                buf.push(arena.alloc());
            }
            let prefix_len = common_prefix_len(prev, input);
            for i in (prefix_len..input.len()).map(|x| x + 1).rev() {
                let state = match state_dict.get(&arena[buf[i]].hash_code) {
                    Some(ss) => ss.iter().find(|&id| id.get(&arena) == buf[i].get(&arena)),
                    None => None,
                };
                let state_id = match state {
                    Some(id) => *id,
                    None => {
                        let sid = arena.alloc();
                        let state = sid.get_mut(&mut arena);

                        sid
                    }
                };
            }
        }
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct MastBuilder {

}

fn common_prefix_len<'a>(a: &'a [u8], b: &'a [u8]) -> usize {
    let mut len = 0;
    for (a, b) in a.iter().zip(b.iter()) {
        if a != b {
            break;
        }
        len += 1;
    }
    len
}