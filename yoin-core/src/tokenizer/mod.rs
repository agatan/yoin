use std::iter::Iterator;
use std::str::Split;
use std::fmt;

mod lattice;
use self::lattice::{Lattice, Node, NodeKind};
use sysdic::SysDic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
    start: usize,
    surface: &'a str,
    contents: &'a str,
}

impl<'a> Token<'a> {
    fn new(node: Node<'a>) -> Self {
        let Node { start, kind } = node;
        let (surface, contents) = match kind {
            NodeKind::BOS | NodeKind::EOS => unreachable!(),
            NodeKind::Known(morph) => (morph.surface, morph.contents),
            NodeKind::Unknown(surface, entry) => (surface, entry.contents),
        };

        Token {
            start: start,
            surface: surface,
            contents: contents,
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

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.surface, self.contents)
    }
}

pub struct FeatureIter<'a>(Split<'a, char>);

impl<'a> Iterator for FeatureIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct Tokenizer {
    sysdic: SysDic,
}

impl Tokenizer {
    pub fn new(sysdic: SysDic) -> Self {
        Tokenizer { sysdic: sysdic }
    }

    pub fn tokenize<'a>(&'a self, input: &'a str) -> Vec<Token<'a>> {
        let la = Lattice::build(input, &self.sysdic);
        la.into_output().into_iter().map(|node| Token::new(node)).collect()
    }
}
