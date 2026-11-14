use super::{DecodeError, ResponseContext, index::Document, scalar};
use inventory_core::{
    catalog::{Shape, Target, TimestampFormat},
    record::{FieldId, KeyRef, RecordBuilder},
};
use std::borrow::Cow;

mod collections;
mod members;
mod payload;
mod schema;

pub(super) use payload::raw_payload;

enum Key<'a> {
    Root,
    Field(FieldId),
    Name(Cow<'a, str>),
    List,
}

struct Value<'a> {
    node: usize,
    target: Target,
    key: Key<'a>,
    timestamp: Option<TimestampFormat>,
}

struct Items<'a> {
    nodes: Vec<usize>,
    target: Target,
    key: Key<'a>,
    timestamp: Option<TimestampFormat>,
}

struct Entries<'a, 'd> {
    nodes: Vec<usize>,
    shape: Shape<'a>,
    key: Key<'d>,
    timestamp: Option<TimestampFormat>,
}

enum Task<'a, 'd> {
    End,
    Value(Value<'d>),
    ListItems(Items<'d>),
    MapEntries(Entries<'a, 'd>),
}

pub(super) struct Model<'a, 'd, 'b> {
    document: &'d Document<'a>,
    context: ResponseContext<'a>,
    builder: &'b mut RecordBuilder,
    tasks: Vec<Task<'a, 'd>>,
}

impl Key<'_> {
    fn reference(&self) -> Option<KeyRef<'_>> {
        match self {
            Self::Root | Self::List => None,
            Self::Field(id) => Some(KeyRef::Field(*id)),
            Self::Name(name) => Some(KeyRef::Name(name)),
        }
    }
}

impl<'a, 'd, 'b> Model<'a, 'd, 'b> {
    pub(super) fn new(
        document: &'d Document<'a>,
        context: ResponseContext<'a>,
        builder: &'b mut RecordBuilder,
    ) -> Self {
        Self {
            document,
            context,
            builder,
            tasks: Vec::new(),
        }
    }

    pub(super) fn run(&mut self, node: usize, target: Target) -> Result<(), DecodeError> {
        self.push(Task::Value(Value {
            node,
            target,
            key: Key::Root,
            timestamp: None,
        }))?;
        while let Some(task) = self.tasks.pop() {
            self.step(task)?;
        }
        Ok(())
    }

    fn step(&mut self, task: Task<'a, 'd>) -> Result<(), DecodeError> {
        match task {
            Task::End => self.builder.end().map_err(Into::into),
            Task::Value(value) => self.value(value),
            Task::ListItems(items) => self.list_nodes(items),
            Task::MapEntries(entries) => self.map_nodes(entries),
        }
    }

    fn push(&mut self, task: Task<'a, 'd>) -> Result<(), DecodeError> {
        if self.tasks.len() >= self.context.limits.nodes() {
            return Err(DecodeError::Limit("XML tasks"));
        }
        self.tasks.try_reserve(1)?;
        self.tasks.push(task);
        Ok(())
    }

    fn value(&mut self, value: Value<'d>) -> Result<(), DecodeError> {
        match value.target {
            Target::Shape(index) => {
                let shape = self
                    .context
                    .service
                    .shape(index)?
                    .ok_or(DecodeError::Invalid("missing XML shape"))?;
                self.named_value(shape, value)
            }
            Target::Prelude("unit") => {
                self.builder.begin_map(value.key.reference())?;
                self.builder.end().map_err(Into::into)
            }
            Target::Prelude(kind) => self.scalar(kind, value),
            _ => Err(DecodeError::Unsupported("XML target")),
        }
    }

    fn named_value(&mut self, shape: Shape<'a>, mut value: Value<'d>) -> Result<(), DecodeError> {
        match shape.kind()? {
            "structure" | "union" => self.structure(value.node, shape, value.key),
            "list" => self.list(shape, value),
            "map" => self.map(shape, value),
            kind => {
                value.timestamp = value.timestamp.or(shape.wire()?.timestamp_format()?);
                self.scalar(kind, value)
            }
        }
    }

    fn scalar(&mut self, kind: &str, value: Value<'d>) -> Result<(), DecodeError> {
        scalar::parse(
            kind,
            scalar::text(self.document, value.node)?,
            value.timestamp,
        )?
        .write(self.builder, value.key.reference())
    }
}
