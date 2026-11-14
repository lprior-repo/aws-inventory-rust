use std::collections::BTreeSet;

use serde_json::Value;

use super::json_ordered::Json;

fn scalar(json: &Json) -> String {
    match json {
        Json::Null => "null".to_owned(),
        Json::Bool(true) => "true".to_owned(),
        Json::Bool(false) => "false".to_owned(),
        Json::Int(value) => value.to_string(),
        Json::UInt(value) => value.to_string(),
        Json::Float(value) => float(*value),
        Json::String(text) => string(text),
        Json::Array(_) | Json::Object(_) => String::new(),
    }
}

fn float(value: f64) -> String {
    if value.is_nan() {
        "NaN".to_owned()
    } else if value.is_infinite() {
        infinity(value)
    } else {
        finite(value)
    }
}

fn infinity(value: f64) -> String {
    if value > 0.0 { "Infinity" } else { "-Infinity" }.to_owned()
}

fn finite(value: f64) -> String {
    let rendered = value.to_string();
    if rendered.contains('.') || rendered.contains('e') || rendered.contains('E') {
        rendered
    } else {
        format!("{rendered}.0")
    }
}

fn string(text: &str) -> String {
    crate::json::canonical(&Value::String(text.to_owned()), true)
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push(' ');
    }
}

fn separate(out: &mut String, index: usize) {
    if index > 0 {
        out.push_str(",\n");
    }
}

fn write(out: &mut String, json: &Json, depth: usize) {
    match json {
        Json::Object(entries) => write_object(out, entries, depth),
        Json::Array(items) => write_array(out, items, depth),
        other => out.push_str(&scalar(other)),
    }
}

fn write_object(out: &mut String, entries: &[(String, Json)], depth: usize) {
    if entries.is_empty() {
        out.push_str("{}");
        return;
    }
    out.push_str("{\n");
    for (index, (key, value)) in entries.iter().enumerate() {
        separate(out, index);
        indent(out, depth.saturating_add(2));
        out.push_str(&string(key));
        out.push_str(": ");
        write(out, value, depth.saturating_add(2));
    }
    out.push('\n');
    indent(out, depth);
    out.push('}');
}

fn write_array(out: &mut String, items: &[Json], depth: usize) {
    if items.is_empty() {
        out.push_str("[]");
        return;
    }
    out.push_str("[\n");
    for (index, value) in items.iter().enumerate() {
        separate(out, index);
        indent(out, depth.saturating_add(2));
        write(out, value, depth.saturating_add(2));
    }
    out.push('\n');
    indent(out, depth);
    out.push(']');
}

pub fn pretty(json: &Json) -> String {
    let mut out = String::new();
    write(&mut out, json, 0);
    out
}

fn default_value(value: &Value) -> String {
    match value {
        Value::Object(object) => {
            let keys: BTreeSet<&str> = object.keys().map(String::as_str).collect();
            let mut parts = Vec::with_capacity(keys.len());
            for key in keys {
                let rendered = object
                    .get(key)
                    .map(default_value)
                    .unwrap_or_else(|| "null".to_owned());
                parts.push(format!("{}: {rendered}", string(key)));
            }
            format!("{{{}}}", parts.join(", "))
        }
        Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(default_value).collect();
            format!("[{}]", parts.join(", "))
        }
        scalar => crate::json::canonical(scalar, true),
    }
}

pub fn dumps_default(value: &Value) -> String {
    default_value(value)
}
