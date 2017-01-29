use std::iter::Iterator;
use std::str::Split;
use lattice::{Node, NodeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
    start: usize,
    surface: &'a str,
    contents: &'a str,
}

impl<'a> Token<'a> {
    fn new(node: Node<'a>) -> Self {
        let Node { start, kind } = node;
        let morph = match kind {
            NodeKind::BOS | NodeKind::EOS => unreachable!(),
            NodeKind::Known(morph) => morph,
        };

        Token {
            start: start,
            surface: morph.surface,
            contents: morph.contents,
        }
    }

    pub fn surface(&self) -> &str {
        self.surface
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.start + self.surface().len()
    }

    pub fn features(&self) -> FeatureIter {
        FeatureIter(self.contents.split(','))
    }
}

pub struct FeatureIter<'a>(Split<'a, char>);

impl<'a> Iterator for FeatureIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
