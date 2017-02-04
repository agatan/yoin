use std::io::{self, Write};

use dic::{Dic, Morph};
use dic::unknown::{UnknownDic, Entry, CharCategorize};
use sysdic::SysDic;

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
    /// for debugging
    #[allow(unused)]
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

#[derive(Clone)]
pub struct Lattice<'a> {
    sdic: &'a SysDic,
    arena: NodeArena<'a>,
    end_nodes: Vec<Vec<NodeId>>,
    prev_table: Vec<NodeId>,
    cost_table: Vec<i64>,
    pointer: usize,
}

/// care about overflow...
const MAX_COST: i64 = ::std::i32::MAX as i64;

impl<'a> Lattice<'a> {
    fn new(char_size: usize, sdic: &'a SysDic) -> Self {
        let mut arena = NodeArena::new();
        let mut end_nodes = vec![Vec::new(); char_size + 2];
        let bos = arena.add(Node {
            start: 0,
            kind: NodeKind::BOS,
        });
        end_nodes[0].push(bos);
        Lattice {
            sdic: sdic,
            arena: arena,
            end_nodes: end_nodes,
            prev_table: vec![0],
            cost_table: vec![0],
            pointer: 0,
        }
    }

    fn add(&mut self, start: usize, kind: NodeKind<'a>) {
        let id = self.arena.add(Node {
            start: start,
            kind: kind,
        });
        let node = self.arena.get(id);
        let node_weight = node.kind.weight() as i64;
        let node_conn_row = self.sdic.matrix.row(node.kind.left_id());
        let mut node_prev = DUMMY_PREV_NODE;
        let mut node_cost = MAX_COST;

        for &enode_id in &self.end_nodes[self.pointer] {
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
        self.add(!0, NodeKind::EOS);
    }

    pub fn build(input: &'a str, sysdic: &'a SysDic) -> Self {
        let mut la = Lattice::new(input.chars().count(), sysdic);
        let mut input_chars = input.chars();
        let mut byte_pos = 0;

        while !input_chars.as_str().is_empty() {
            let mut is_matched = false;
            for m in sysdic.dic.lookup_str_iter(input_chars.as_str()) {
                is_matched = true;
                la.add(byte_pos, NodeKind::Known(m));
            }
            let ch = input_chars.clone().next().unwrap();
            let category = sysdic.unknown_dic.categorize(ch);
            let cid = sysdic.unknown_dic.category_id(ch);
            let input_str = input_chars.as_str();

            // if no morphs found or character category requires to invoke unknown search
            if !is_matched || category.invoke {
                let mut end = ch.len_utf8();
                let mut word_len = 1;
                let entries = sysdic.unknown_dic.fetch_entries(cid);
                if category.group {
                    while end < input_str.len() {
                        let c = match input_str[end..].chars().next() {
                            None => break,
                            Some(ch) => ch,
                        };
                        if cid != sysdic.unknown_dic.category_id(c) {
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
                        la.add(byte_pos, NodeKind::Unknown(surface, e.clone()));
                    }
                }
                if category.length > 0 {
                    let mut p = 0;
                    let mut cloned_chars = input_chars.clone();
                    for _ in 0..category.length {
                        match cloned_chars.next() {
                            None => break,
                            Some(c) => {
                                if sysdic.unknown_dic.category_id(c) != cid {
                                    break;
                                }
                                p += c.len_utf8();
                            }
                        }
                        let surface = &(input_chars.as_str())[..p];
                        for e in entries.iter() {
                            la.add(byte_pos, NodeKind::Unknown(surface, e.clone()));
                        }
                    }
                }
            }
            let cnt = la.forward();
            for _ in 0..cnt {
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

    /// for debugging
    #[allow(unused)]
    fn dump_dot<W: Write>(&self, mut w: W) -> io::Result<()> {
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
