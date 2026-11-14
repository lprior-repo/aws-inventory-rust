use super::{Fault, partition};
use serde_json::Value;

#[derive(Debug, Clone, Copy)]
pub(super) enum Function {
    IsSet,
    BooleanEquals,
    StringEquals,
    Not,
    GetAttr,
    HostLabel,
    Partition,
}

impl Function {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "isSet" => Self::IsSet,
            "booleanEquals" => Self::BooleanEquals,
            "stringEquals" => Self::StringEquals,
            "not" => Self::Not,
            "getAttr" => Self::GetAttr,
            "isValidHostLabel" => Self::HostLabel,
            "aws.partition" => Self::Partition,
            _ => return None,
        })
    }

    pub const fn arity(self) -> usize {
        match self {
            Self::IsSet | Self::Not | Self::Partition => 1,
            _ => 2,
        }
    }
}

pub(super) fn apply(function: Function, args: &[Value]) -> Result<Value, Fault> {
    use Function as F;
    use Value::{Bool, String as S};
    let value = match (function, args) {
        (F::IsSet, [v]) => Bool(!v.is_null()),
        (F::BooleanEquals, [Bool(a), Bool(b)]) => Bool(a == b),
        (F::StringEquals, [S(a), S(b)]) => Bool(a == b),
        (F::Not, [Bool(a)]) => Bool(!a),
        (F::GetAttr, [v, S(path)]) if v.is_object() || v.is_array() => {
            optional(attr(v, path).cloned())
        }
        (F::HostLabel, [S(s), Bool(sub)]) => Bool(host_label(s, *sub)),
        (F::Partition, [S(s)]) => return partition::resolve(s),
        _ => return Err(Fault::Arguments(format!("{function:?}"))),
    };
    Ok(value)
}

fn optional(value: Option<Value>) -> Value {
    let Some(value) = value else {
        return Value::Null;
    };
    value
}

pub(super) fn attr<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(value, |v, segment| {
        let Some((key, index)) = segment.split_once('[') else {
            return v.get(segment);
        };
        let v = if key.is_empty() { v } else { v.get(key)? };
        let index = index.strip_suffix(']')?.parse::<usize>().ok()?;
        v.as_array()?.get(index)
    })
}

fn host_label(input: &str, subdomains: bool) -> bool {
    if subdomains {
        return input.split('.').all(label);
    }
    label(input)
}

fn label(input: &str) -> bool {
    (1..=63).contains(&input.len())
        && input
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && input
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
        && input
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
