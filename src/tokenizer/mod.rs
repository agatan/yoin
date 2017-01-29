use std::iter::Iterator;
use std::str::Split;
use std::marker::PhantomData;
use std::fmt;

mod lattice;
use self::lattice::{Lattice, Node, NodeKind};
use dict::Dict;

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

#[derive(Debug)]
pub struct Tokenizer<'a, D: Dict<'a> + 'a> {
    dic: D,
    _mark: PhantomData<&'a D>,
}

impl<'a, D: Dict<'a> + 'a> Tokenizer<'a, D> {
    pub fn new_with_dic(dic: D) -> Self {
        Tokenizer {
            dic: dic,
            _mark: PhantomData,
        }
    }

    pub fn tokenize(&'a self, input: &'a str) -> Vec<Token<'a>> {
        let la = Lattice::build(input, &self.dic);
        la.into_output().into_iter().map(|node| Token::new(node)).collect()
    }
}
