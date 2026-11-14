use super::super::CompileError;
use base64::Engine;
use inventory_core::record::{Float, Key, Time, Value};
use num_traits::ToPrimitive;
use serde_json::{Number, Value as Json};

pub(super) fn normalize(value: &Json, kind: Option<&str>) -> Result<Value, CompileError> {
    if value.is_null() {
        return Ok(Value::Null);
    }
    match kind {
        Some("blob") => blob(value),
        Some("timestamp") => timestamp(value),
        Some("boolean") => boolean(value),
        Some("string") => string(value),
        Some("float" | "double") => float(value),
        _ => compound(value, kind),
    }
}

fn blob(value: &Json) -> Result<Value, CompileError> {
    value
        .as_str()
        .map(decode_base64)
        .transpose()?
        .map(Value::Bytes)
        .ok_or_else(|| invalid("blob default must be a base64 string"))
}

fn boolean(value: &Json) -> Result<Value, CompileError> {
    value
        .as_bool()
        .map(Value::Bool)
        .ok_or_else(|| invalid("boolean default must be boolean"))
}

fn string(value: &Json) -> Result<Value, CompileError> {
    value
        .as_str()
        .map(|value| Value::Text(value.to_owned()))
        .ok_or_else(|| invalid("string default must be a string"))
}

fn float(value: &Json) -> Result<Value, CompileError> {
    if let Some(text) = value.as_str() {
        return match text {
            "NaN" | "Infinity" | "-Infinity" => Ok(Value::Text(text.to_owned())),
            _ => Err(invalid("invalid non-finite float default")),
        };
    }
    value
        .as_f64()
        .filter(|value| value.is_finite())
        .map(|value| Value::Float(Float::new(value)))
        .ok_or_else(|| invalid("float default must be finite"))
}

enum Task<'a> {
    Visit(&'a Json),
    Array(usize),
    Object(Vec<&'a str>),
}

fn compound(value: &Json, kind: Option<&str>) -> Result<Value, CompileError> {
    if let Json::Number(value) = value {
        return number(value, kind);
    }
    let mut tasks = vec![Task::Visit(value)];
    let mut results = Vec::new();
    while let Some(task) = tasks.pop() {
        visit(task, &mut tasks, &mut results)?;
    }
    results
        .pop()
        .ok_or_else(|| invalid("empty default conversion"))
}

fn visit<'a>(
    task: Task<'a>,
    tasks: &mut Vec<Task<'a>>,
    results: &mut Vec<Value>,
) -> Result<(), CompileError> {
    match task {
        Task::Visit(Json::Null) => results.push(Value::Null),
        Task::Visit(Json::Bool(value)) => results.push(Value::Bool(*value)),
        Task::Visit(Json::Number(value)) => results.push(number(value, None)?),
        Task::Visit(Json::String(value)) => results.push(Value::Text(value.clone())),
        Task::Visit(Json::Array(values)) => {
            tasks.push(Task::Array(values.len()));
            tasks.extend(values.iter().rev().map(Task::Visit));
        }
        Task::Visit(Json::Object(values)) => {
            let keys = values.keys().map(String::as_str).collect::<Vec<_>>();
            tasks.push(Task::Object(keys));
            tasks.extend(values.values().rev().map(Task::Visit));
        }
        Task::Array(count) => array(count, results)?,
        Task::Object(keys) => object(keys, results)?,
    }
    Ok(())
}

fn array(count: usize, results: &mut Vec<Value>) -> Result<(), CompileError> {
    let start = results
        .len()
        .checked_sub(count)
        .ok_or_else(|| invalid("invalid array default"))?;
    let items = results.drain(start..).collect();
    results.push(Value::List(items));
    Ok(())
}

fn object(keys: Vec<&str>, results: &mut Vec<Value>) -> Result<(), CompileError> {
    let start = results
        .len()
        .checked_sub(keys.len())
        .ok_or_else(|| invalid("invalid object default"))?;
    let fields = keys
        .into_iter()
        .zip(results.drain(start..))
        .map(|(key, value)| (Key::Name(key.to_owned()), value))
        .collect();
    results.push(Value::Map(fields));
    Ok(())
}

fn number(value: &Number, kind: Option<&str>) -> Result<Value, CompileError> {
    if kind == Some("bigDecimal") {
        return Ok(Value::Text(value.to_string()));
    }
    if matches!(
        kind,
        Some("byte" | "short" | "integer" | "long" | "bigInteger")
    ) {
        return integer(value, kind);
    }
    if let Some(value) = value.as_i64() {
        return Ok(Value::Int(value));
    }
    if let Some(value) = value.as_u64() {
        return Ok(i64::try_from(value)
            .map(Value::Int)
            .unwrap_or_else(|_| Value::Text(value.to_string())));
    }
    finite_or_text(value)
}

fn integer(value: &Number, kind: Option<&str>) -> Result<Value, CompileError> {
    let Some(value) = value.as_i64() else {
        return if kind == Some("bigInteger") {
            Ok(Value::Text(value.to_string()))
        } else {
            Err(invalid("integer default is out of range"))
        };
    };
    let range = match kind {
        Some("byte") => i64::from(i8::MIN)..=i64::from(i8::MAX),
        Some("short") => i64::from(i16::MIN)..=i64::from(i16::MAX),
        Some("integer") => i64::from(i32::MIN)..=i64::from(i32::MAX),
        _ => i64::MIN..=i64::MAX,
    };
    if range.contains(&value) {
        Ok(Value::Int(value))
    } else {
        Err(invalid("integer default is out of target range"))
    }
}

fn finite_or_text(value: &Number) -> Result<Value, CompileError> {
    if let Some(value) = value.as_f64().filter(|value| value.is_finite()) {
        return Ok(Value::Float(Float::new(value)));
    }
    Ok(Value::Text(value.to_string()))
}

fn timestamp(value: &Json) -> Result<Value, CompileError> {
    let seconds = value
        .as_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| invalid("timestamp default must be finite seconds"))?;
    let whole = seconds.floor();
    timestamp_parts(whole, seconds)
}

fn timestamp_parts(whole: f64, seconds: f64) -> Result<Value, CompileError> {
    let rounded_nanos = ((seconds - whole) * 1_000_000_000.0).round();
    let mut seconds = whole
        .to_i64()
        .ok_or_else(|| invalid("timestamp seconds out of range"))?;
    let mut nanos = rounded_nanos
        .to_u32()
        .ok_or_else(|| invalid("timestamp nanos out of range"))?;
    if nanos == 1_000_000_000 {
        seconds = seconds
            .checked_add(1)
            .ok_or_else(|| invalid("timestamp seconds out of range"))?;
        nanos = 0;
    }
    Time::new(seconds, nanos)
        .map(Value::Time)
        .map_err(|_| invalid("timestamp nanos out of range"))
}

fn decode_base64(text: &str) -> Result<Vec<u8>, CompileError> {
    base64::engine::general_purpose::STANDARD
        .decode(text)
        .map_err(|_| invalid("invalid base64 default"))
}

fn invalid(message: &str) -> CompileError {
    CompileError::Trait(message.into())
}
