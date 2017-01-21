use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Morph<'a> {
    data: &'a str,
}

impl<'a> Morph<'a> {
    pub fn new(data: &'a str) -> Self {
        Morph {
            data: data,
        }
    }
}

impl<'a> fmt::Display for Morph<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Morph({})", self.data)
    }
}