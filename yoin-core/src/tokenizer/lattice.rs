use std::convert::AsRef;
use std::io::{self, Write};

use dic::{Dic, Morph, Matrix};
use dic::unknown::{UnknownDic, Entry};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind<'a> {
    BOS,
    EOS,
    Known(Morph<&'a str>),
    Unknown(&'a str, Entry<'a>),
}

impl<'a> NodeKind<'a> {
    fn left_id(&self) -> u16 {
        match *self {
            NodeKind::BOS | NodeKind::EOS => 0,
            NodeKind::Known(ref morph) => morph.left_id,
            NodeKind::Unknown(_, ref e) => e.left_id,
        }
    }

    fn right_id(&self) -> u16 {
        match *self {
            NodeKind::BOS | NodeKind::EOS => 0,
            NodeKind::Known(ref morph) => morph.right_id,
            NodeKind::Unknown(_, ref e) => e.right_id,
        }
    }

    fn weight(&self) -> i16 {
        match *self {
            NodeKind::BOS | NodeKind::EOS => 0,
            NodeKind::Known(ref morph) => morph.weight,
            NodeKind::Unknown(_, ref e) => e.weight,
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
    fn surface(&self) -> &str {
        match self.kind {
            NodeKind::BOS => "BOS",
            NodeKind::EOS => "EOS",
            NodeKind::Known(ref m) => m.surface,
            NodeKind::Unknown(surface, _) => surface,
        }
    }

    fn surface_len(&self) -> usize {
        match self.kind {
            NodeKind::BOS => 0,
            NodeKind::EOS => 1,
            NodeKind::Known(ref m) => m.surface.chars().count(),
            NodeKind::Unknown(s, _) => s.chars().count(),
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

const DUMMY_PREV_NODE: NodeId = !0;

#[derive(Debug, Clone, PartialEq)]
pub struct Lattice<'a, D: Dic<'a> + 'a, Unk: UnknownDic + 'a, T: AsRef<[i16]> + 'a> {
    dic: &'a D,
    unk_dic: &'a Unk,
    matrix: &'a Matrix<T>,
    arena: NodeArena<'a>,
    end_nodes: Vec<Vec<NodeId>>,
    prev_table: Vec<NodeId>,
    cost_table: Vec<i64>,
}

/// care about overflow...
const MAX_COST: i64 = ::std::i32::MAX as i64;

impl<'a, D: Dic<'a> + 'a, Unk: UnknownDic + 'a, T: AsRef<[i16]>> Lattice<'a, D, Unk, T> {
    fn new(char_size: usize, dic: &'a D, unk_dic: &'a Unk, matrix: &'a Matrix<T>) -> Self {
        let mut arena = NodeArena::new();
        let mut end_nodes = vec![Vec::new(); char_size + 2];
        let bos = arena.add(Node {
            start: 0,
            kind: NodeKind::BOS,
        });
        end_nodes[0].push(bos);
        Lattice {
            dic: dic,
            unk_dic: unk_dic,
            matrix: matrix,
            arena: arena,
            end_nodes: end_nodes,
            prev_table: vec![0],
            cost_table: vec![0],
        }
    }

    fn add(&mut self, char_pos: usize, start_byte: usize, kind: NodeKind<'a>) {
        let id = self.arena.add(Node {
            start: start_byte,
            kind: kind,
        });
        let node = self.arena.get(id);
        let node_weight = node.kind.weight() as i64;
        let node_conn_row = self.matrix.row(node.kind.left_id());
        let mut node_prev = DUMMY_PREV_NODE;
        let mut node_cost = MAX_COST;

        for &enode_id in &self.end_nodes[char_pos] {
            let enode = self.arena.get(enode_id);
            let cost = node_conn_row[enode.kind.right_id() as usize] as i64 + node_weight;
            let total_cost = self.cost_table[enode_id] + cost;
            if total_cost < node_cost {
                node_cost = total_cost;
                node_prev = enode_id;
            }
        }

        self.prev_table.push(node_prev);
        self.cost_table.push(node_cost);
        self.end_nodes[char_pos + node.surface_len()].push(id);
    }

    fn next_char_pos(&mut self, mut char_pos: usize) -> usize {
        char_pos += 1;
        while self.end_nodes[char_pos].is_empty() {
            char_pos += 1;
        }
        char_pos
    }

    fn end(&mut self) {
        let char_size = self.end_nodes.len() - 2;
        self.add(char_size, !0, NodeKind::EOS);
    }

    pub fn build(input: &'a str, dic: &'a D, unk_dic: &'a Unk, matrix: &'a Matrix<T>) -> Self {
        let mut la = Lattice::new(input.chars().count(), dic, unk_dic, matrix);
        let mut input_chars = input.chars();
        let mut char_pos = 0;
        let mut byte_pos = 0;

        while !input_chars.as_str().is_empty() {
            let mut is_matched = false;
            for m in dic.lookup_str_iter(input_chars.as_str()) {
                is_matched = true;
                la.add(char_pos, byte_pos, NodeKind::Known(m));
            }
            let ch = input_chars.clone().next().unwrap();
            let category = unk_dic.categorize(ch);
            let cid = unk_dic.category_id(ch);
            let input_str = input_chars.as_str();

            // if no morphs found or character category requires to invoke unknown search
            if !is_matched || category.invoke {
                let mut end = ch.len_utf8();
                let mut word_len = 1;
                let entries = unk_dic.fetch_entries(cid);
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
                    let surface = &input_str[..end];
                    for e in entries.iter() {
                        la.add(char_pos, byte_pos, NodeKind::Unknown(surface, e.clone()));
                    }
                }
                if category.length > 0 {
                    let mut p = 0;
                    let mut cloned_chars = input_chars.clone();
                    for _ in 0..category.length {
                        match cloned_chars.next() {
                            None => break,
                            Some(c) => {
                                if unk_dic.category_id(c) != cid {
                                    break;
                                }
                                p += c.len_utf8();
                            }
                        }
                        let surface = &(input_chars.as_str())[..p];
                        for e in entries.iter() {
                            la.add(char_pos, byte_pos, NodeKind::Unknown(surface, e.clone()));
                        }
                    }
                }
            }
            let old_char_pos = char_pos;
            char_pos = la.next_char_pos(char_pos);
            for _ in 0..(char_pos - old_char_pos) {
                if let Some(c) = input_chars.next() {
                    byte_pos += c.len_utf8();
                }
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
            p = self.prev_table[p];
            if p == DUMMY_PREV_NODE {
                return Vec::new();
            }
            while p != 0 {
                path.push(p);
                p = self.prev_table[p];
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

    pub fn dump_dot<W: Write>(&self, mut w: W) -> io::Result<()> {
        writeln!(w, "digraph lattice {{")?;
        writeln!(w, "\trankdir=LR;")?;
        writeln!(w, "\tnode [shape=circle]")?;
        for (id, &prev_id) in self.prev_table.iter().enumerate() {
            if prev_id != DUMMY_PREV_NODE {
                writeln!(w,
                         "\t\"{}\"[label=\"{}\"];",
                         id,
                         self.arena.get(id).surface())?;
                let prev_node = self.arena.get(prev_id);
                writeln!(w, "\t\"{}\"[label=\"{}\"];", prev_id, prev_node.surface())?;
                writeln!(w, "\t\"{}\" -> \"{}\";", prev_id, id)?;
            }
        }
        writeln!(w, "}}")
    }
}
