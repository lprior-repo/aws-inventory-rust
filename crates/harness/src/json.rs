//! Python-compatible canonical JSON used by frozen fixture digests.
//!
//! Existing receipts hash `json.dumps(value, sort_keys=True,
//! separators=(",", ":"))` with Python's default `ensure_ascii=True`. Digests
//! must survive the toolchain migration unchanged, so the escaping and key
//! ordering rules are reproduced exactly rather than delegated to serde_json.

use serde_json::Value;

/// Serialize with sorted keys and no insignificant whitespace.
///
/// `ensure_ascii` mirrors Python's flag: when set, every code point above
/// `0x7e` is emitted as a `\uXXXX` escape (surrogate pairs above the BMP).
pub fn canonical(value: &Value, ensure_ascii: bool) -> String {
    let mut out = String::new();
    write_value(&mut out, value, ensure_ascii);
    out
}

/// Serialize with sorted keys, no whitespace and UTF-8 preserved.
pub fn canonical_utf8(value: &Value) -> String {
    canonical(value, false)
}

fn write_value(out: &mut String, value: &Value, ascii: bool) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => write_number(out, number),
        Value::String(text) => write_string(out, text, ascii),
        Value::Array(items) => write_array(out, items, ascii),
        Value::Object(object) => write_object(out, object, ascii),
    }
}

fn write_number(out: &mut String, number: &serde_json::Number) {
    if let Some(value) = number.as_i64() {
        out.push_str(&value.to_string());
    } else if let Some(value) = number.as_u64() {
        out.push_str(&value.to_string());
    } else {
        // Python renders the same shortest round-trip digits for ordinary
        // values; fixtures carry provider integers and strings, not floats.
        out.push_str(&number.to_string());
    }
}

fn write_array(out: &mut String, items: &[Value], ascii: bool) {
    out.push('[');
    for (index, item) in items.iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        write_value(out, item, ascii);
    }
    out.push(']');
}

fn write_object(out: &mut String, object: &serde_json::Map<String, Value>, ascii: bool) {
    let mut keys: Vec<&String> = object.keys().collect();
    keys.sort();
    out.push('{');
    for (index, key) in keys.iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        write_string(out, key, ascii);
        out.push(':');
        if let Some(value) = object.get(*key) {
            write_value(out, value, ascii);
        }
    }
    out.push('}');
}

fn write_string(out: &mut String, text: &str, ascii: bool) {
    out.push('"');
    for character in text.chars() {
        write_character(out, character, ascii);
    }
    out.push('"');
}

fn write_character(out: &mut String, character: char, ascii: bool) {
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
            if code < 0x20 || (ascii && code > 0x7e) {
                write_escape(out, code);
            } else {
                out.push(character);
            }
        }
    }
}

fn write_escape(out: &mut String, code: u32) {
    match u16::try_from(code) {
        Ok(unit) => {
            out.push_str("\\u");
            out.push_str(&format!("{unit:04x}"));
        }
        Err(_) => {
            // Surrogate pair for code points above the BMP.
            let adjusted = code.saturating_sub(0x1_0000);
            let high = 0xd800_u32.saturating_add(adjusted >> 10);
            let low = 0xdc00_u32.saturating_add(adjusted & 0x3ff);
            out.push_str(&format!("\\u{high:04x}\\u{low:04x}"));
        }
    }
}
