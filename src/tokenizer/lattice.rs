use std::collections::HashMap;

use dict::{Dic, Morph};
use dict::unknown::{UnknownDic, Entry};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind<'a> {
    BOS,
    EOS,
    Known(Morph<&'a str>),
    Unkown(&'a str, Entry<'a>),
}

impl<'a> NodeKind<'a> {
    fn left_id(&self) -> u16 {
        match *self {
            NodeKind::BOS | NodeKind::EOS => 0,
            NodeKind::Known(ref morph) => morph.left_id,
            NodeKind::Unkown(_, ref e) => e.left_id,
        }
    }

    fn right_id(&self) -> u16 {
        match *self {
            NodeKind::BOS | NodeKind::EOS => 0,
            NodeKind::Known(ref morph) => morph.right_id,
            NodeKind::Unkown(_, ref e) => e.right_id,
        }
    }

    fn weight(&self) -> i16 {
        match *self {
            NodeKind::BOS | NodeKind::EOS => 0,
            NodeKind::Known(ref morph) => morph.weight,
            NodeKind::Unkown(_, ref e) => e.weight,
        }
    }
}

type NodeId = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct Node<'a> {
    pub start: usize,
    pub kind: NodeKind<'a>,
}

impl<'a> Node<'a> {
    fn surface_len(&self) -> usize {
        match self.kind {
            NodeKind::BOS => 0,
            NodeKind::EOS => 1,
            NodeKind::Known(ref m) => m.surface.chars().count(),
            NodeKind::Unkown(s, _) => s.chars().count(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct NodeArena<'a>(Vec<Node<'a>>);

impl<'a> NodeArena<'a> {
    fn new() -> Self {
        NodeArena(Vec::new())
    }

    fn add(&mut self, node: Node<'a>) -> NodeId {
        let id = self.0.len();
        self.0.push(node);
        id
    }

    fn get(&self, id: NodeId) -> &Node<'a> {
        &(self.0)[id]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lattice<'a, D: Dic<'a> + 'a, Unk: UnknownDic + 'a> {
    dic: &'a D,
    unk_dic: &'a Unk,
    arena: NodeArena<'a>,
    end_nodes: Vec<Vec<NodeId>>,
    prev_table: HashMap<NodeId, NodeId>,
    cost_table: HashMap<NodeId, i64>,
    pointer: usize,
}

/// care about overflow...
const MAX_COST: i64 = ::std::i32::MAX as i64;

impl<'a, D: Dic<'a> + 'a, Unk: UnknownDic + 'a> Lattice<'a, D, Unk> {
    fn new(char_size: usize, dic: &'a D, unk_dic: &'a Unk) -> Self {
        let mut arena = NodeArena::new();
        let mut end_nodes = vec![Vec::new(); char_size + 2];
        let bos = arena.add(Node {
            start: 0,
            kind: NodeKind::BOS,
        });
        end_nodes[0].push(bos);
        let mut cost_table = HashMap::new();
        cost_table.insert(bos, 0);
        Lattice {
            dic: dic,
            unk_dic: unk_dic,
            arena: arena,
            end_nodes: end_nodes,
            prev_table: HashMap::new(),
            cost_table: cost_table,
            pointer: 0,
        }
    }

    fn add(&mut self, kind: NodeKind<'a>) {
        let start = self.pointer;
        let id = self.arena.add(Node {
            start: start,
            kind: kind,
        });
        let node = self.arena.get(id);
        for &enode_id in &self.end_nodes[self.pointer] {
            let enode = self.arena.get(enode_id);
            let cost = self.dic.connection_cost(enode.kind.right_id(), node.kind.left_id()) as i64 +
                       node.kind.weight() as i64;
            let total_cost = self.min_cost(enode_id) + cost;
            if total_cost < self.min_cost(id) {
                self.cost_table.insert(id, total_cost);
                self.prev_table.insert(id, enode_id);
            }
        }
        self.end_nodes[self.pointer + node.surface_len()].push(id);
    }

    fn forward(&mut self) -> usize {
        let old = self.pointer;
        self.pointer += 1;
        while self.end_nodes[self.pointer].is_empty() {
            self.pointer += 1;
        }
        self.pointer - old
    }

    fn end(&mut self) {
        self.add(NodeKind::EOS);
    }

    pub fn build(input: &'a str, dic: &'a D, unk_dic: &'a Unk) -> Self {
        let mut la = Lattice::new(input.chars().count(), dic, unk_dic);
        let mut input_chars = input.chars();

        while !input_chars.as_str().is_empty() {
            let mut is_matched = false;
            for m in dic.lookup_str_iter(input_chars.as_str()) {
                is_matched = true;
                la.add(NodeKind::Known(m));
            }
            let ch = input_chars.clone().next().unwrap();
            let category = unk_dic.categorize(ch);
            let cid = unk_dic.category_id(ch);
            let input_str = input_chars.as_str();
            if !is_matched || category.invoke {
                // if no morphs found or character category requires to invoke unknown search
                let mut end = ch.len_utf8();
                let mut word_len = 1;
                if category.group {
                    while end < input_str.len() {
                        let c = match input_str[end..].chars().next() {
                            None => break,
                            Some(ch) => ch,
                        };
                        if cid != unk_dic.category_id(c) {
                            break;
                        }
                        end += c.len_utf8();
                        word_len += 1;
                        const MAX_UNKOWN_WORD_LEN: usize = 1024;
                        if word_len > MAX_UNKOWN_WORD_LEN {
                            break;
                        }
                    }
                }
                let mut p = 0;
                let mut cloned_chars =  input_chars.clone();
                let entries = unk_dic.fetch_entries(cid);
                for _ in 0..word_len {
                    match cloned_chars.next() {
                        None => break,
                        Some(c) => p += c.len_utf8(),
                    }
                    let surface = &(input_chars.as_str())[..p];
                    for e in entries.iter() {
                        la.add(NodeKind::Unkown(surface, e.clone()));
                    }
                }
            }
            let cnt = la.forward();
            for _ in 0..cnt {
                input_chars.next();
            }
        }

        la.end();
        la
    }

    fn rev_output_path(&self) -> Vec<NodeId> {
        if let Some(ref ps) = self.end_nodes.last() {
            let mut path = Vec::new();
            let mut p = ps[0];
            debug_assert!(self.arena.get(p).kind == NodeKind::EOS);
            // skip EOS node.
            match self.prev_table.get(&p) {
                Some(&prev) => p = prev,
                None => return Vec::new(),
            }
            while let Some(prev) = self.prev_table.get(&p).cloned() {
                path.push(p);
                p = prev;
            }
            debug_assert!(self.arena.get(p).kind == NodeKind::BOS);
            path
        } else {
            Vec::new()
        }
    }

    pub fn into_output(self) -> Vec<Node<'a>> {
        let path = self.rev_output_path();
        let NodeArena(mut nodes) = self.arena;
        let mut results = Vec::new();
        for p in path {
            results.push(nodes.swap_remove(p));
        }
        results.reverse();
        results
    }

    fn min_cost(&self, id: NodeId) -> i64 {
        match self.cost_table.get(&id) {
            Some(&cost) => cost,
            None => MAX_COST,
        }
    }
}
