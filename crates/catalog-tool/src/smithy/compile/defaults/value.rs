use super::super::CompileError;
use inventory_core::record::{Float, Key, Time, Value};
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
        Some("byte") => i8::MIN as i64..=i8::MAX as i64,
        Some("short") => i16::MIN as i64..=i16::MAX as i64,
        Some("integer") => i32::MIN as i64..=i32::MAX as i64,
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
    if whole < i64::MIN as f64 || whole >= i64::MAX as f64 {
        return Err(invalid("timestamp seconds out of range"));
    }
    timestamp_parts(whole, seconds)
}

fn timestamp_parts(mut whole: f64, seconds: f64) -> Result<Value, CompileError> {
    let rounded_nanos = ((seconds - whole) * 1_000_000_000.0).round();
    let carry = rounded_nanos >= 1_000_000_000.0;
    if carry {
        whole += 1.0;
    }
    if whole < i64::MIN as f64 || whole >= i64::MAX as f64 {
        return Err(invalid("timestamp seconds out of range"));
    }
    let nanos = if carry {
        0
    } else {
        u32::try_from(rounded_nanos as u64).map_err(|_| invalid("timestamp nanos out of range"))?
    };
    let time =
        Time::new(whole as i64, nanos).map_err(|_| invalid("timestamp nanos out of range"))?;
    Ok(Value::Time(time))
}

fn decode_base64(text: &str) -> Result<Vec<u8>, CompileError> {
    let bytes = text.as_bytes();
    if bytes.len() % 4 != 0 {
        return Err(invalid("invalid base64 length"));
    }
    let mut output = Vec::with_capacity(bytes.len().saturating_mul(3) / 4);
    for (index, block) in bytes.chunks_exact(4).enumerate() {
        decode_block(block, index + 1 == bytes.len() / 4, &mut output)?;
    }
    Ok(output)
}

fn decode_block(block: &[u8], final_block: bool, output: &mut Vec<u8>) -> Result<(), CompileError> {
    let padding = if block[3] == b'=' { 1 } else { 0 } + if block[2] == b'=' { 1 } else { 0 };
    if (padding > 0 && !final_block) || (block[2] == b'=' && block[3] != b'=') {
        return Err(invalid("invalid base64 padding"));
    }
    let digits = decode_digits(block)?;
    validate_padding(digits, padding)?;
    decoded_bytes(digits, padding, output);
    Ok(())
}

fn validate_padding([_, b, c, _]: [u8; 4], padding: usize) -> Result<(), CompileError> {
    if (padding == 2 && b & 15 != 0) || (padding == 1 && c & 3 != 0) {
        return Err(invalid("invalid base64 padding bits"));
    }
    Ok(())
}

fn decode_digits(block: &[u8]) -> Result<[u8; 4], CompileError> {
    let mut digits = [0; 4];
    for (index, byte) in block.iter().copied().enumerate() {
        if byte == b'=' {
            if index < 2 {
                return Err(invalid("invalid base64 padding"));
            }
        } else {
            digits[index] = base64_digit(byte).ok_or_else(|| invalid("invalid base64 default"))?;
        }
    }
    Ok(digits)
}

fn decoded_bytes([a, b, c, d]: [u8; 4], padding: usize, output: &mut Vec<u8>) {
    output.push((a << 2) | (b >> 4));
    if padding < 2 {
        output.push((b << 4) | (c >> 2));
    }
    if padding == 0 {
        output.push((c << 6) | d);
    }
}

fn base64_digit(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn invalid(message: &str) -> CompileError {
    CompileError::Trait(message.into())
}
