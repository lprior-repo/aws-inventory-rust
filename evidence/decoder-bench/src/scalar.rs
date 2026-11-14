use crate::{
    error::{Error, Result, syntax},
    shape::Shape,
};
use base64::Engine;
use inventory_core::record::{Float, Time, Value};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Text,
    Number,
    Bool,
    Null,
    Xml,
}
pub fn convert(shape: Shape, text: &str, kind: Kind) -> Result<Value> {
    if shape == Shape::Skip {
        return Ok(Value::Null);
    }
    if shape == Shape::Document {
        return document(text, kind);
    }
    match (shape, kind) {
        (Shape::Text, Kind::Text | Kind::Xml) => Ok(Value::Text(text.to_owned())),
        (Shape::Int, Kind::Number | Kind::Xml) => Ok(Value::Int(text.parse().map_err(syntax)?)),
        (Shape::Float, Kind::Number | Kind::Xml) => {
            Ok(Value::Float(Float::new(text.parse().map_err(syntax)?)))
        }
        (Shape::Bool, Kind::Bool | Kind::Xml) => Ok(Value::Bool(text.parse().map_err(syntax)?)),
        (Shape::Null, Kind::Null | Kind::Xml) => Ok(Value::Null),
        (Shape::Time, Kind::Number | Kind::Xml) => timestamp(text, kind),
        (Shape::Blob, Kind::Text | Kind::Xml) => Ok(Value::Bytes(
            base64::engine::general_purpose::STANDARD
                .decode(text)
                .map_err(syntax)?,
        )),
        _ => Err(Error::Shape("scalar type")),
    }
}
fn timestamp(text: &str, kind: Kind) -> Result<Value> {
    let (seconds, nanos) = if kind == Kind::Xml {
        let time =
            time::OffsetDateTime::parse(text, &time::format_description::well_known::Rfc3339)
                .map_err(syntax)?;
        (time.unix_timestamp(), time.nanosecond())
    } else {
        let (s, n) = match text.split_once('.') {
            Some(pair) => pair,
            None => (text, ""),
        };
        if n.len() > 9 || !n.bytes().all(|b| b.is_ascii_digit()) {
            return Err(Error::Shape("epoch fraction"));
        }
        let nanos = if n.is_empty() {
            0
        } else {
            n.parse::<u32>().map_err(syntax)?
        };
        let power = u32::try_from(9usize.saturating_sub(n.len())).map_err(|_| Error::Bound)?;
        (
            s.parse().map_err(syntax)?,
            nanos.checked_mul(10u32.pow(power)).ok_or(Error::Bound)?,
        )
    };
    Ok(Value::Time(Time::new(seconds, nanos)?))
}
fn document(text: &str, kind: Kind) -> Result<Value> {
    match kind {
        Kind::Text | Kind::Xml => Ok(Value::Text(text.to_owned())),
        Kind::Bool => Ok(Value::Bool(text.parse().map_err(syntax)?)),
        Kind::Null => Ok(Value::Null),
        Kind::Number if text.contains(['.', 'e', 'E']) => {
            Ok(Value::Float(Float::new(text.parse().map_err(syntax)?)))
        }
        Kind::Number => Ok(Value::Int(text.parse().map_err(syntax)?)),
    }
}
