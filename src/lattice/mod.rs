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
pub struct Lattice<'a, D: Dict<'a>> {
    input: &'a str,
    dic: D,
    node_list: Vec<Vec<Node<'a>>>,
}

impl<'a, D: Dict<'a>> Lattice<'a, D> {
    pub fn new(input: &'a str, dic: D) -> Self {
        let char_count = input.chars().count();
        let node_list = vec![vec![]; char_count + 2]; // head and last
        let mut la = Lattice {
            input: input,
            dic: dic,
            node_list: node_list,
        };
        la.add_node(0, NodeKind::Dummy);
        la.add_node(char_count+1, NodeKind::Dummy);
        // TODO(agatan): build the lattice for the input and the dic
        la
    }

    fn add_node(&mut self, p: usize, kind: NodeKind<'a>) {
        let node = Node {
            kind: kind,
            cost: 0,
        };
        self.node_list[p].push(node);
    }
}
