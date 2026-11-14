use crate::{
    error::{Error, Result},
    scalar::Kind,
    shape::{self, Shape},
    sink::Sink,
};
#[derive(Clone, Copy, PartialEq, Eq)]
enum Node {
    Root,
    Wrapper,
    Value(Shape),
    Entry,
    Key,
    MapValue,
    Skip,
}
struct Frame {
    name: String,
    node: Node,
    text: String,
    key: Option<String>,
}
pub struct Bridge<'a> {
    stack: Vec<Frame>,
    pub sink: &'a mut Sink,
    roots: usize,
}
impl<'a> Bridge<'a> {
    pub fn new(sink: &'a mut Sink) -> Self {
        Self {
            stack: Vec::with_capacity(96),
            sink,
            roots: 0,
        }
    }
    pub fn depth(&self) -> usize {
        self.stack.len()
    }
    pub fn scalar(&self) -> bool {
        self.stack.last().is_some_and(|f| {
            matches!(f.node, Node::Key | Node::MapValue)
                || matches!(f.node, Node::Value(s) if !s.object() && !s.list())
        })
    }
    pub fn start(&mut self, name: &str) -> Result<()> {
        if self.stack.len() >= 96 {
            return Err(Error::Bound);
        }
        let node = self.node(name)?;
        self.open(node, name)?;
        self.stack.push(Frame {
            name: name.to_owned(),
            node,
            text: String::new(),
            key: None,
        });
        Ok(())
    }
    fn node(&self, name: &str) -> Result<Node> {
        let parent = self.stack.last().map(|f| f.node);
        match (parent, name) {
            (None, "DescribeInstancesResponse") if self.roots == 0 => Ok(Node::Root),
            (None, _) => Err(Error::Shape("XML root")),
            (Some(Node::Root), "reservationSet") | (Some(Node::Wrapper), "item") => {
                Ok(Node::Wrapper)
            }
            (Some(Node::Wrapper), "instancesSet") => Ok(Node::Value(Shape::Items)),
            (Some(Node::Value(Shape::Items)), "item") => Ok(Node::Value(Shape::Item)),
            (Some(Node::Value(Shape::Groups)), "item") => Ok(Node::Value(Shape::Group)),
            (Some(Node::Value(Shape::Tags)), "item") => Ok(Node::Entry),
            (Some(Node::Entry), "key") => Ok(Node::Key),
            (Some(Node::Entry), "value") => Ok(Node::MapValue),
            (Some(Node::Value(s)), n) => {
                Ok(shape::member(s, n).map_or(Node::Skip, |m| Node::Value(m.shape)))
            }
            _ => Ok(Node::Skip),
        }
    }
    fn open(&mut self, node: Node, name: &str) -> Result<()> {
        match node {
            Node::Root => {
                self.roots = self.roots.saturating_add(1);
                self.sink.begin(true)
            }
            Node::Value(s) => {
                if s == Shape::Items {
                    self.sink.key("Items")?;
                } else if let Some(Node::Value(parent)) = self.stack.last().map(|f| f.node)
                    && let Some(m) = shape::member(parent, name)
                {
                    self.sink.key(m.json)?;
                }
                if s.object() || s.list() {
                    self.sink.begin(s.object())?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub fn text(&mut self, text: &str) -> Result<()> {
        if self.scalar() {
            let f = self
                .stack
                .last_mut()
                .ok_or(Error::Shape("text outside XML"))?;
            f.text.try_reserve(text.len()).map_err(|_| Error::Bound)?;
            f.text.push_str(text);
        } else if self.stack.is_empty() && !text.trim().is_empty() {
            return Err(Error::Shape("text after XML"));
        }
        Ok(())
    }
    pub fn end(&mut self, name: &str) -> Result<()> {
        if !self.stack.last().is_some_and(|f| f.name == name) {
            return Err(Error::Shape("XML mismatched end"));
        }
        self.close()
    }
    pub fn close(&mut self) -> Result<()> {
        let f = self.stack.pop().ok_or(Error::Shape("XML extra close"))?;
        match f.node {
            Node::Root => self.sink.end(true),
            Node::Value(s) if s.object() || s.list() => self.sink.end(s.object()),
            Node::Value(_) => self.sink.scalar(&f.text, Kind::Xml),
            Node::Key => {
                let entry = self.stack.last_mut().ok_or(Error::Shape("map entry"))?;
                if entry.key.replace(f.text).is_some() {
                    return Err(Error::Shape("duplicate XML map key"));
                }
                Ok(())
            }
            Node::MapValue => {
                let key = self
                    .stack
                    .last_mut()
                    .and_then(|f| f.key.take())
                    .ok_or(Error::Shape("map key absent"))?;
                self.sink.key(&key)?;
                self.sink.scalar(&f.text, Kind::Xml)
            }
            _ => Ok(()),
        }
    }
    pub fn finish(&self) -> Result<()> {
        if !self.stack.is_empty() || self.roots != 1 {
            return Err(Error::Shape("XML truncated root"));
        }
        Ok(())
    }
}
