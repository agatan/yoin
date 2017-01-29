use std::collections::HashMap;

use dict::{Dict, Morph};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind<'a> {
    BOS,
    EOS,
    Known(Morph<&'a str>),
}

impl<'a> NodeKind<'a> {
    fn left_id(&self) -> u16 {
        match *self {
            NodeKind::BOS | NodeKind::EOS => 0,
            NodeKind::Known(ref morph) => morph.left_id,
        }
    }

    fn right_id(&self) -> u16 {
        match *self {
            NodeKind::BOS | NodeKind::EOS => 0,
            NodeKind::Known(ref morph) => morph.right_id,
        }
    }

    fn weight(&self) -> i16 {
        match *self {
            NodeKind::BOS | NodeKind::EOS => 0,
            NodeKind::Known(ref morph) => morph.weight,
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
    pub fn surface(&self) -> &'a str {
        match self.kind {
            NodeKind::BOS | NodeKind::EOS => "",
            NodeKind::Known(ref m) => m.surface,
        }
    }

    fn surface_len(&self) -> usize {
        match self.kind {
            NodeKind::BOS => 0,
            NodeKind::EOS => 1,
            NodeKind::Known(ref m) => m.surface.chars().count(),
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
pub struct Lattice<'a, D: Dict<'a> + 'a> {
    dic: &'a D,
    arena: NodeArena<'a>,
    end_nodes: Vec<Vec<NodeId>>,
    prev_table: HashMap<NodeId, NodeId>,
    cost_table: HashMap<NodeId, i64>,
    pointer: usize,
}

/// care about overflow...
const MAX_COST: i64 = ::std::i32::MAX as i64;

impl<'a, D: Dict<'a> + 'a> Lattice<'a, D> {
    fn new(char_size: usize, dic: &'a D) -> Self {
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

    pub fn build(input: &'a str, dic: &'a D) -> Self {
        let mut la = Lattice::new(input.chars().count(), dic);
        let mut input_chars = input.chars();
        while !input_chars.as_str().is_empty() {
            for m in dic.lookup_str_iter(input_chars.as_str()) {
                la.add(NodeKind::Known(m));
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

    pub fn output(&self) -> Vec<&Node<'a>> {
        let path = self.rev_output_path();
        path.into_iter().rev().map(|id| self.arena.get(id)).collect()
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
