use crate::{
    error::{Error, Result, syntax},
    scalar::Kind,
    sink::Sink,
};
use json_event_parser::{JsonEvent, SliceJsonParser};
pub fn parse(input: &[u8], sink: &mut Sink) -> Result<()> {
    let mut parser = SliceJsonParser::new(input);
    let mut done = false;
    std::iter::from_fn(|| {
        if done {
            return None;
        }
        Some(match parser.parse_next().map_err(syntax) {
            Ok(JsonEvent::Eof) => {
                done = true;
                Ok(())
            }
            Ok(event) => emit(event, sink),
            Err(e) => Err(e),
        })
    })
    .take(input.len().saturating_add(1))
    .try_for_each(|r| r)?;
    if !done {
        return Err(Error::Bound);
    }
    Ok(())
}
fn emit(event: JsonEvent<'_>, sink: &mut Sink) -> Result<()> {
    match event {
        JsonEvent::StartObject => sink.begin(true),
        JsonEvent::EndObject => sink.end(true),
        JsonEvent::StartArray => sink.begin(false),
        JsonEvent::EndArray => sink.end(false),
        JsonEvent::ObjectKey(key) => sink.key(&key),
        JsonEvent::String(value) => sink.scalar(&value, Kind::Text),
        JsonEvent::Number(value) => sink.scalar(&value, Kind::Number),
        JsonEvent::Boolean(value) => sink.scalar(if value { "true" } else { "false" }, Kind::Bool),
        JsonEvent::Null => sink.scalar("", Kind::Null),
        JsonEvent::Eof => Ok(()),
    }
}
pub fn parse_reader(input: impl std::io::Read, sink: &mut Sink, bound: usize) -> Result<()> {
    let mut parser = json_event_parser::ReaderJsonParser::new(input);
    let mut done = false;
    std::iter::from_fn(|| {
        if done {
            return None;
        }
        Some(match parser.parse_next().map_err(syntax) {
            Ok(JsonEvent::Eof) => {
                done = true;
                Ok(())
            }
            Ok(event) => emit(event, sink),
            Err(e) => Err(e),
        })
    })
    .take(bound.saturating_add(1))
    .try_for_each(|r| r)?;
    if !done {
        return Err(Error::Bound);
    }
    Ok(())
}
