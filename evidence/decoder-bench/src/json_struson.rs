use crate::{
    error::{Error, Result, syntax},
    scalar::Kind,
    sink::Sink,
};
use struson::reader::{JsonReader, JsonStreamReader, ValueType};
#[derive(Clone, Copy)]
enum Task {
    Value,
    Next(bool),
}
pub fn parse(input: &[u8], sink: &mut Sink) -> Result<()> {
    let mut reader = JsonStreamReader::new(input);
    let mut tasks = vec![Task::Value];
    std::iter::from_fn(|| {
        tasks.pop().map(|task| match task {
            Task::Value => value(&mut reader, sink, &mut tasks),
            Task::Next(object) => next(&mut reader, sink, &mut tasks, object),
        })
    })
    .take(input.len().saturating_mul(2).saturating_add(1))
    .try_for_each(|result| result)?;
    if !tasks.is_empty() {
        return Err(Error::Bound);
    }
    reader.consume_trailing_whitespace().map_err(syntax)
}
fn next(r: &mut impl JsonReader, s: &mut Sink, tasks: &mut Vec<Task>, object: bool) -> Result<()> {
    if r.has_next().map_err(syntax)? {
        if object {
            s.key(r.next_name().map_err(syntax)?)?;
        }
        tasks.extend([Task::Next(object), Task::Value]);
        return Ok(());
    }
    if object {
        r.end_object()
    } else {
        r.end_array()
    }
    .map_err(syntax)?;
    s.end(object)
}
fn value(r: &mut impl JsonReader, s: &mut Sink, tasks: &mut Vec<Task>) -> Result<()> {
    match r.peek().map_err(syntax)? {
        ValueType::Object => {
            r.begin_object().map_err(syntax)?;
            s.begin(true)?;
            tasks.push(Task::Next(true));
            Ok(())
        }
        ValueType::Array => {
            r.begin_array().map_err(syntax)?;
            s.begin(false)?;
            tasks.push(Task::Next(false));
            Ok(())
        }
        ValueType::String => s.scalar(r.next_str().map_err(syntax)?, Kind::Text),
        ValueType::Number => s.scalar(r.next_number_as_str().map_err(syntax)?, Kind::Number),
        ValueType::Boolean => s.scalar(
            if r.next_bool().map_err(syntax)? {
                "true"
            } else {
                "false"
            },
            Kind::Bool,
        ),
        ValueType::Null => {
            r.next_null().map_err(syntax)?;
            s.scalar("", Kind::Null)
        }
    }
}
