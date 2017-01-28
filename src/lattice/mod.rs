use std::collections::HashMap;

use dict::Dict;
use morph::Morph;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind<'a> {
    BOS,
    Known(Morph<&'a str>),
}

impl<'a> NodeKind<'a> {
    fn left_id(&mut self) -> u16 {
        match *self {
            NodeKind::BOS => 0,
            NodeKind::Known(ref morph) => morph.left_id,
        }
    }

    fn right_id(&mut self) -> u16 {
        match *self {
            NodeKind::BOS => 0,
            NodeKind::Known(ref morph) => morph.right_id,
        }
    }

    fn weight(&mut self) -> i16 {
        match *self {
            NodeKind::BOS => 0,
            NodeKind::Known(ref morph) => morph.weight,
        }
    }
}

type NodeId = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct Node<'a> {
    id: NodeId,
    start: usize,
    kind: NodeKind<'a>,
}

impl<'a> Node<'a> {
    pub fn surface(&self) -> &'a str {
        match self.kind {
            NodeKind::BOS => "",
            NodeKind::Known(ref m) => m.surface,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeArena<'a>(Vec<Node<'a>>);

impl<'a> NodeArena<'a> {
    fn new() -> Self {
        NodeArena(Vec::new())
    }

    fn add_with_id<F: FnOnce(NodeId) -> Node<'a>>(&mut self, f: F) -> NodeId {
        let id = self.0.len();
        let node = f(id);
        self.0.push(node);
        id
    }

    fn get(&self, id: NodeId) -> &Node<'a> {
        &(self.0)[id]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lattice<'a, D: Dict<'a> + 'a>{
    dic: &'a D,
    arena: NodeArena<'a>,
    end_nodes: Vec<Vec<NodeId>>,
    prev_table: HashMap<NodeId, NodeId>,
    cost_table: HashMap<NodeId, i64>,
    pointer: usize,
}

impl<'a, D: Dict<'a> + 'a>Lattice<'a, D> {
    pub fn new(char_size: usize, dic: &'a D) -> Self {
        let mut arena = NodeArena::new();
        let mut end_nodes = vec![Vec::new(); char_size + 2];
        let bos = arena.add_with_id(|id| Node { id: id, start: 0, kind: NodeKind::BOS });
        end_nodes[0].push(bos);
        Lattice {
            dic: dic,
            arena: arena,
            end_nodes: end_nodes,
            prev_table: HashMap::new(),
            cost_table: HashMap::new(),
            pointer: 0,
        }
    }

    pub fn add(&mut self, start: usize, kind: NodeKind<'a>) {
        for enode_id in &self.end_nodes[self.pointer] {
        }
    }
}
