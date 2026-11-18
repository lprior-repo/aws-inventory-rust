use super::{DecodeError, ResponseContext, scalar};
mod array;
mod containers;
mod object;
mod payload;
mod schema;
use self::schema::{
    begin_list, begin_map, element_target, member_target, nested_depth, nodes_add, object_target,
    shape_kind, syntax,
};
use inventory_core::{
    catalog::{HttpBinding, Member, Shape, Target, TimestampFormat},
    record::{FieldId, RecordBuilder},
};
use jiter::{Jiter, Peek};
pub(super) use payload::{decode_payload, payload_member};

#[derive(Clone)]
pub(super) enum Key {
    Field(FieldId, Option<TimestampFormat>, bool),
    Name(String, Option<TimestampFormat>),
    List(Option<TimestampFormat>),
    Ignore,
}

#[derive(Clone, Copy)]
enum Position {
    First,
    Next,
}

#[derive(Clone, Copy)]
enum MapRole {
    Root,
    Nested,
}

enum Variants {
    Structure,
    EmptyUnion,
    SeenUnion,
}

struct Object {
    target: Option<Target>,
    depth: usize,
    position: Position,
    role: MapRole,
    variants: Variants,
    fields: Vec<FieldId>,
}

struct Array {
    target: Option<Target>,
    depth: usize,
    position: Position,
}

struct Value {
    target: Option<Target>,
    key: Key,
    depth: usize,
}

enum Task {
    Value(Value),
    Object(Object),
    Array(Array),
}

struct Engine<'a, 'b> {
    parser: Jiter<'a>,
    context: ResponseContext<'a>,
    builder: &'b mut RecordBuilder,
    tasks: Vec<Task>,
    nodes: usize,
}

pub(super) fn decode_object(
    context: ResponseContext<'_>,
    target: Target,
    builder: &mut RecordBuilder,
) -> Result<(), DecodeError> {
    if context.body.is_empty() {
        return crate::response::defaults::apply(
            builder,
            Some(target),
            &[],
            context.service,
            context.limits,
        );
    }
    let root = Object::new(Some(target), 0, MapRole::Root, context)?;
    Engine::new(context, builder, 1).run(Task::Object(root))
}

impl<'a, 'b> Engine<'a, 'b> {
    fn new(context: ResponseContext<'a>, builder: &'b mut RecordBuilder, nodes: usize) -> Self {
        Self {
            parser: Jiter::new(context.body),
            context,
            builder,
            tasks: Vec::new(),
            nodes,
        }
    }

    fn run(&mut self, root: Task) -> Result<(), DecodeError> {
        self.push(root)?;
        while let Some(task) = self.tasks.pop() {
            self.step(task)?;
        }
        self.parser.finish().map_err(syntax)
    }

    fn push(&mut self, task: Task) -> Result<(), DecodeError> {
        self.tasks.try_reserve(1)?;
        self.tasks.push(task);
        Ok(())
    }

    fn step(&mut self, task: Task) -> Result<(), DecodeError> {
        match task {
            Task::Object(object) => self.object(object),
            Task::Array(array) => self.array(array),
            Task::Value(value) => self.value(value),
        }
    }

    fn value(&mut self, value: Value) -> Result<(), DecodeError> {
        nodes_add(&mut self.nodes, self.context)?;
        match self.parser.peek().map_err(syntax)? {
            Peek::Object => self.open_object(value),
            Peek::Array => self.open_array(value),
            _ => scalar::write(
                &mut self.parser,
                value.target,
                value.key,
                self.context,
                self.builder,
            ),
        }
    }
}
