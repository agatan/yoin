use std::iter::Iterator;
use std::str::Split;
use std::marker::PhantomData;
use std::fmt;

mod lattice;
use self::lattice::{Lattice, Node, NodeKind};
use dict::Dic;
use dict::unknown::UnknownDic;

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
            NodeKind::Unkown(surface, entry) => (surface, entry.contents),
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

#[derive(Debug)]
pub struct Tokenizer<'a, D: Dic<'a> + 'a, Unk: UnknownDic> {
    dic: D,
    unk_dic: Unk,
    _mark: PhantomData<&'a D>,
}

impl<'a, D: Dic<'a> + 'a, Unk: UnknownDic> Tokenizer<'a, D, Unk> {
    pub fn new_with_dic(dic: D, unk_dic: Unk) -> Self {
        Tokenizer {
            dic: dic,
            unk_dic: unk_dic,
            _mark: PhantomData,
        }
    }

    pub fn tokenize(&'a self, input: &'a str) -> Vec<Token<'a>> {
        let la = Lattice::build(input, &self.dic, &self.unk_dic);
        la.into_output().into_iter().map(|node| Token::new(node)).collect()
    }
}
