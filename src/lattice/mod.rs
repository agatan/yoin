use std::collections::HashMap;

use dict::Dict;
use morph::Morph;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind<'a> {
    Dummy,
    Known(Morph<&'a str>),
}

type NodeId = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct Node<'a> {
    id: NodeId,
    kind: NodeKind<'a>,
}

impl<'a> Node<'a> {
    pub fn surface(&self) -> &'a str {
        match self.kind {
            NodeKind::Dummy => "",
            NodeKind::Known(ref m) => m.surface,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeArena<'a>(Vec<Node<'a>>, Vec<Vec<NodeId>>);

impl<'a> NodeArena<'a> {
    fn new(size: usize) -> Self {
        NodeArena(Vec::new(), vec![Vec::new(); size])
    }

    fn alloc_node(&mut self, kind: NodeKind<'a>) -> NodeId {
        let id = self.0.len();
        let node = Node {
            id: id,
            kind: kind,
        };
        self.0.push(node);
        id
    }

    fn add_node(&mut self, char_pos: usize, kind: NodeKind<'a>) {
        let id = self.alloc_node(kind);
        (self.1)[char_pos].push(id);
    }

    fn get(&self, id: NodeId) -> &Node<'a> {
        &(self.0)[id]
    }

    fn get_on_pos(&self, char_pos: usize) -> &[NodeId] {
        (self.1)[char_pos].as_slice()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct LatticeState {
    prev_list: HashMap<NodeId, NodeId>,
    cost_table: HashMap<NodeId, i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lattice<'a>{
    nodes: NodeArena<'a>,
    prev_table: HashMap<NodeId, NodeId>,
    cost_table: HashMap<NodeId, i64>,
}

impl<'a>Lattice<'a> {
    pub fn build<D: Dict<'a>>(input: &'a str, dic: &'a D) -> Self {
        let char_count = input.chars().count();
        let mut nodes = NodeArena::new(char_count);

        nodes.add_node(0, NodeKind::Dummy);
        nodes.add_node(char_count + 1, NodeKind::Dummy);
        for (char_pos, (pos, _)) in input.char_indices().enumerate() {
            if nodes.get_on_pos(char_pos).is_empty() {
                // if char_pos is not end of any morphs, skip here.
                continue;
            }
            let token = &input[pos..];
            for morph in dic.lookup_str_iter(token) {
                nodes.add_node(char_pos, NodeKind::Known(morph));
            }
        }
        Lattice {
            nodes: nodes,
            prev_table: HashMap::new(),
            cost_table: HashMap::new(),
        }
    }
}
