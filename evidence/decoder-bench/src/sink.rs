use crate::{
    error::{Error, Result},
    shape::{self, Shape},
};
use inventory_core::record::{FieldId, Key, Record, Value};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Tokens,
    Records,
}
struct Frame {
    shape: Shape,
    object: bool,
    key: Option<Key>,
    next: Shape,
    map: BTreeMap<Key, Value>,
    list: Vec<Value>,
    seen: u64,
}
pub struct Sink {
    stack: Vec<Frame>,
    pub records: Vec<Vec<u8>>,
    pub items: usize,
    pub tokens: usize,
    mode: Mode,
    finished: bool,
}
impl Sink {
    pub fn new(mode: Mode) -> Self {
        Self {
            stack: Vec::with_capacity(96),
            records: Vec::new(),
            items: 0,
            tokens: 0,
            mode,
            finished: false,
        }
    }
    pub fn expected(&self) -> Shape {
        self.stack.last().map_or(Shape::Page, |f| {
            if f.object {
                f.next
            } else {
                shape::child(f.shape, "")
            }
        })
    }
    pub fn begin(&mut self, object: bool) -> Result<()> {
        let shape = self.expected();
        if self.finished || self.stack.len() >= 96 {
            return Err(Error::Bound);
        }
        if !matches!(shape, Shape::Skip | Shape::Document) && object != shape.object() {
            return Err(Error::Shape("container type"));
        }
        self.stack.push(Frame {
            shape,
            object,
            key: None,
            next: Shape::Skip,
            map: BTreeMap::new(),
            list: Vec::new(),
            seen: 0,
        });
        self.tokens = self.tokens.saturating_add(1);
        Ok(())
    }
    pub fn key(&mut self, name: &str) -> Result<()> {
        let f = self
            .stack
            .last_mut()
            .ok_or(Error::Shape("key outside object"))?;
        if !f.object || f.key.is_some() {
            return Err(Error::Shape("unexpected key"));
        }
        let m = shape::member(f.shape, name);
        f.next = shape::child(f.shape, name);
        if let Some(m) = m {
            let mask = 1u64.checked_shl(m.id).ok_or(Error::Bound)?;
            if f.seen & mask != 0 {
                return Err(Error::Shape("duplicate known key"));
            }
            f.seen |= mask;
            f.key = Some(Key::Field(FieldId::new(m.id)?));
        } else if matches!(f.shape, Shape::Tags | Shape::Document) {
            f.key = Some(Key::Name(name.to_owned()));
        }
        self.tokens = self.tokens.saturating_add(1);
        Ok(())
    }
    pub fn scalar(&mut self, text: &str, kind: crate::scalar::Kind) -> Result<()> {
        self.tokens = self.tokens.saturating_add(1);
        let shape = self.expected();
        if self.mode == Mode::Tokens {
            return self.attach(Value::Null);
        }
        let value = crate::scalar::convert(shape, text, kind)?;
        self.attach(value)
    }
    fn attach(&mut self, value: Value) -> Result<()> {
        let f = self.stack.last_mut().ok_or(Error::Shape("scalar root"))?;
        if f.object {
            if let Some(key) = f.key.take()
                && self.mode == Mode::Records
                && f.map.insert(key, value).is_some()
            {
                return Err(Error::Shape("duplicate map key"));
            }
            f.next = Shape::Skip;
        } else if self.mode == Mode::Records && f.shape != Shape::Items && f.shape != Shape::Skip {
            f.list.try_reserve(1).map_err(|_| Error::Bound)?;
            f.list.push(value);
        }
        Ok(())
    }
    pub fn end(&mut self, object: bool) -> Result<()> {
        let f = self.stack.pop().ok_or(Error::Shape("extra close"))?;
        if f.object != object || f.key.is_some() {
            return Err(Error::Shape("unmatched close"));
        }
        self.tokens = self.tokens.saturating_add(1);
        let value = if object {
            Value::Map(f.map)
        } else {
            Value::List(f.list)
        };
        if f.shape == Shape::Item {
            self.emit(value)?;
        } else if !self.stack.is_empty() {
            self.attach(value)?;
        }
        if self.stack.is_empty() {
            self.finished = true;
        }
        Ok(())
    }
    fn emit(&mut self, value: Value) -> Result<()> {
        self.items = self.items.saturating_add(1);
        if self.mode == Mode::Records {
            self.records.try_reserve(1).map_err(|_| Error::Bound)?;
            self.records.push(Record::encode(&value)?.into_bytes());
        }
        Ok(())
    }
    pub fn finish(self) -> Result<Self> {
        if !self.finished || !self.stack.is_empty() {
            return Err(Error::Shape("truncated document"));
        }
        Ok(self)
    }
}
