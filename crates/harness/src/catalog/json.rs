mod parser;

use crate::HarnessError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Json {
    Null,

    Bool(bool),

    Number(String),

    String(String),

    Array(Vec<Json>),

    Object(Vec<(String, Json)>),
}

impl Json {
    pub fn object(entries: Vec<(&str, Json)>) -> Self {
        Self::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
        )
    }

    pub fn array(items: Vec<Json>) -> Self {
        Self::Array(items)
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn count(value: usize) -> Self {
        Self::Number(value.to_string())
    }

    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Self::Object(entries) => entries
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Json> {
        match self {
            Self::Object(entries) => entries
                .iter_mut()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn set(&mut self, key: &str, value: Json) {
        if let Self::Object(entries) = self {
            if let Some((_, slot)) = entries.iter_mut().find(|(name, _)| name == key) {
                *slot = value;
            } else {
                entries.push((key.to_string(), value));
            }
        }
    }

    pub fn remove(&mut self, key: &str) {
        if let Self::Object(entries) = self {
            entries.retain(|(name, _)| name != key);
        }
    }

    pub fn as_object(&self) -> Option<&[(String, Json)]> {
        match self {
            Self::Object(entries) => Some(entries),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut [Json]> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn at(&self, path: &[&str]) -> Option<&Json> {
        let mut current = self;
        for key in path {
            current = current.get(key)?;
        }
        Some(current)
    }

    pub fn pretty(&self) -> String {
        let mut out = String::new();
        write_value(&mut out, self, 0);
        out
    }

    pub fn parse(text: &str) -> Result<Self, HarnessError> {
        parser::parse(text)
    }

    pub fn to_serde(&self) -> Result<serde_json::Value, HarnessError> {
        Ok(match self {
            Self::Null => serde_json::Value::Null,
            Self::Bool(value) => serde_json::Value::Bool(*value),
            Self::Number(raw) => serde_json::from_str(raw)?,
            Self::String(value) => serde_json::Value::String(value.clone()),
            Self::Array(items) => serde_json::Value::Array(
                items
                    .iter()
                    .map(Self::to_serde)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            Self::Object(entries) => {
                let mut object = serde_json::Map::new();
                for (key, value) in entries {
                    object.insert(key.clone(), value.to_serde()?);
                }
                serde_json::Value::Object(object)
            }
        })
    }

    pub fn from_serde(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(value) => Self::Bool(*value),
            serde_json::Value::Number(raw) => Self::Number(raw.to_string()),
            serde_json::Value::String(value) => Self::String(value.clone()),
            serde_json::Value::Array(items) => {
                Self::Array(items.iter().map(Self::from_serde).collect())
            }
            serde_json::Value::Object(entries) => Self::Object(
                entries
                    .iter()
                    .map(|(key, value)| (key.clone(), Self::from_serde(value)))
                    .collect(),
            ),
        }
    }
}

fn write_value(out: &mut String, value: &Json, depth: usize) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(true) => out.push_str("true"),
        Json::Bool(false) => out.push_str("false"),
        Json::Number(raw) => out.push_str(raw),
        Json::String(text) => write_string(out, text),
        Json::Array(items) if items.is_empty() => out.push_str("[]"),
        Json::Array(items) => write_array(out, items, depth),
        Json::Object(entries) if entries.is_empty() => out.push_str("{}"),
        Json::Object(entries) => write_object(out, entries, depth),
    }
}

fn write_array(out: &mut String, items: &[Json], depth: usize) {
    out.push('[');
    for (index, item) in items.iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        newline(out, depth.saturating_add(1));
        write_value(out, item, depth.saturating_add(1));
    }
    newline(out, depth);
    out.push(']');
}

fn write_object(out: &mut String, entries: &[(String, Json)], depth: usize) {
    out.push('{');
    for (index, (key, item)) in entries.iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        newline(out, depth.saturating_add(1));
        write_string(out, key);
        out.push_str(": ");
        write_value(out, item, depth.saturating_add(1));
    }
    newline(out, depth);
    out.push('}');
}

fn newline(out: &mut String, depth: usize) {
    out.push('\n');
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn write_string(out: &mut String, text: &str) {
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            _ => {
                let code = u32::from(character);
                if !(0x20..=0x7e).contains(&code) {
                    escape(out, code);
                } else {
                    out.push(character);
                }
            }
        }
    }
    out.push('"');
}

fn escape(out: &mut String, code: u32) {
    match u16::try_from(code) {
        Ok(unit) => out.push_str(&format!("\\u{unit:04x}")),
        Err(_) => {
            let adjusted = code.saturating_sub(0x1_0000);
            let high = 0xd800_u32.saturating_add(adjusted >> 10);
            let low = 0xdc00_u32.saturating_add(adjusted & 0x3ff);
            out.push_str(&format!("\\u{high:04x}\\u{low:04x}"));
        }
    }
}
