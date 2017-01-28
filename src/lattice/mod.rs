use std::collections::HashMap;

use dict::Dict;
use morph::Morph;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind<'a> {
    Dummy,
    Known(Morph<&'a str>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node<'a> {
    kind: NodeKind<'a>,
    cost: i32,
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
pub struct NodeFolder<'a>(HashMap<usize, Vec<Node<'a>>>);

impl<'a> NodeFolder<'a> {
    fn new() -> Self {
        NodeFolder(HashMap::new())
    }

    fn add_node(&mut self, p: usize, kind: NodeKind<'a>) {
        let node = Node {
            kind: kind,
            cost: 0,
        };
        self.0.entry(p).or_insert(Vec::new()).push(node);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lattice<'a>{
    nodes: NodeFolder<'a>,
}

impl<'a>Lattice<'a> {
    pub fn new<D: Dict<'a>>(input: &'a str, dic: &'a D) -> Self {
        let mut nodes = NodeFolder::new();
        nodes.add_node(0, NodeKind::Dummy);
        let char_count = input.chars().count();
        nodes.add_node(char_count + 1, NodeKind::Dummy);
        // TODO(agatan): build the lattice for the input and the dic
        for (pos, _) in input.char_indices() {
            let token = &input[pos..];
            for morph in dic.lookup_str_iter(token) {
                nodes.add_node(pos, NodeKind::Known(morph));
            }
        }
        Lattice {
            nodes: nodes,
        }
    }
}
