use super::{DecodeError, ResponseContext};
use quick_xml::events::{BytesCData, BytesStart, BytesText};

mod namespace;
mod parser;
mod validation;

pub(super) enum Token<'a> {
    Start(BytesStart<'a>),
    Empty(BytesStart<'a>),
    Text(BytesText<'a>),
    CData(BytesCData<'a>),
    Reference(char),
    Other,
}

pub(super) struct Document<'a> {
    pub tokens: Vec<Token<'a>>,
    pub parents: Vec<Option<usize>>,
    pub mates: Vec<usize>,
    pub root: usize,
    pub has_root: bool,
}

impl<'a> Document<'a> {
    pub fn parse(context: ResponseContext<'a>) -> Result<Self, DecodeError> {
        parser::parse(context)
    }

    pub fn next_child(&self, parent: usize, after: usize) -> Option<usize> {
        let mut index = after;
        let end = self.mates.get(parent).copied()?;
        while index < end {
            if self.parents.get(index).copied().flatten() == Some(parent)
                && matches!(
                    self.tokens.get(index),
                    Some(Token::Start(_) | Token::Empty(_))
                )
            {
                return Some(index);
            }
            index = self.mates.get(index).copied()?.saturating_add(1);
        }
        None
    }
}
