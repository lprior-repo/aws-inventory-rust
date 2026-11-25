use crate::record::{Float, Key, Time};

pub const MAX_FIXED_DEPTH: usize = 8;
pub const MAX_FIXED_NODES: usize = 1_024;
pub const MAX_FIXED_COLLECTION: usize = 64;
pub const MAX_FIXED_STRING: usize = 16_384;
pub const MAX_FIXED_TEXT_BYTES: usize = 1_048_576;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Float(Float),
    Bytes(Box<[u8]>),
    Timestamp(Time),
    Strings(Box<[String]>),
    List(Box<[Self]>),
    Object(Box<[(Key, Self)]>),
}

impl BindingValue {
    pub const MAX_DEPTH: usize = MAX_FIXED_DEPTH;
    pub const MAX_NODES: usize = MAX_FIXED_NODES;
    pub const MAX_COLLECTION: usize = MAX_FIXED_COLLECTION;
    pub const MAX_STRING: usize = MAX_FIXED_STRING;
    pub const MAX_TEXT_BYTES: usize = MAX_FIXED_TEXT_BYTES;
}
