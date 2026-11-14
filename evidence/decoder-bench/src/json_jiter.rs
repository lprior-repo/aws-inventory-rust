use crate::{
    error::{Error, Result, syntax},
    scalar::Kind,
    sink::Sink,
};
use jiter::{Jiter, Peek};
#[derive(Clone, Copy)]
enum Task {
    Value,
    Object,
    Array,
}
pub fn parse(input: &[u8], sink: &mut Sink) -> Result<()> {
    let mut parser = Jiter::new(input);
    let mut tasks = vec![Task::Value];
    std::iter::from_fn(|| {
        tasks.pop().map(|task| match task {
            Task::Value => value(&mut parser, sink, &mut tasks),
            Task::Object => object(&mut parser, sink, &mut tasks, false),
            Task::Array => array(&mut parser, sink, &mut tasks, false),
        })
    })
    .take(input.len().saturating_mul(2).saturating_add(1))
    .try_for_each(|result| result)?;
    if !tasks.is_empty() {
        return Err(Error::Bound);
    }
    parser.finish().map_err(syntax)
}
fn value(p: &mut Jiter<'_>, s: &mut Sink, tasks: &mut Vec<Task>) -> Result<()> {
    match p.peek().map_err(syntax)? {
        Peek::Object => {
            s.begin(true)?;
            object(p, s, tasks, true)
        }
        Peek::Array => {
            s.begin(false)?;
            array(p, s, tasks, true)
        }
        Peek::String => s.scalar(p.next_str().map_err(syntax)?, Kind::Text),
        Peek::Null => {
            p.next_null().map_err(syntax)?;
            s.scalar("", Kind::Null)
        }
        Peek::True | Peek::False => s.scalar(
            if p.next_bool().map_err(syntax)? {
                "true"
            } else {
                "false"
            },
            Kind::Bool,
        ),
        _ => s.scalar(
            std::str::from_utf8(p.next_number_bytes().map_err(syntax)?).map_err(syntax)?,
            Kind::Number,
        ),
    }
}
fn object(p: &mut Jiter<'_>, s: &mut Sink, tasks: &mut Vec<Task>, first: bool) -> Result<()> {
    let key = if first { p.next_object() } else { p.next_key() }.map_err(syntax)?;
    if let Some(key) = key {
        s.key(key)?;
        tasks.extend([Task::Object, Task::Value]);
        Ok(())
    } else {
        s.end(true)
    }
}
fn array(p: &mut Jiter<'_>, s: &mut Sink, tasks: &mut Vec<Task>, first: bool) -> Result<()> {
    let next = if first {
        p.next_array()
    } else {
        p.array_step()
    }
    .map_err(syntax)?;
    if next.is_some() {
        tasks.extend([Task::Array, Task::Value]);
        Ok(())
    } else {
        s.end(false)
    }
}
