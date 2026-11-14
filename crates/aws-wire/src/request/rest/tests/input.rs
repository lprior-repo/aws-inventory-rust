use inventory_core::{catalog::{Operation, Service, Target}, record::{Float, Key, Record, Time, Value}};
use serde_json::Value as Json;
use std::collections::BTreeMap;

pub(super) fn record(service: Service<'_>, operation: Operation<'_>, params: &Json) -> Record {
    let mut fields = BTreeMap::new();
    if let Some(params) = params.as_object() {
        let input = operation.input().expect("operation input");
        let Target::Shape(index) = input else {
            assert!(params.is_empty(), "unit operation has fixture params");
            return Record::encode(&Value::Map(fields)).expect("empty fixture record");
        };
        let shape = service.shape(index).expect("input shape lookup").expect("input shape");
        for (name, json) in params {
            let member = shape.members().expect("input members").map(|item| item.expect("valid member"))
                .find(|member| member.name().expect("member name") == name)
                .unwrap_or_else(|| panic!("unknown fixture input member {name}"));
            fields.insert(Key::Field(member.id), value(service, member.target, json));
        }
    }
    Record::encode(&Value::Map(fields)).expect("fixture input record")
}

fn value(service: Service<'_>, target: Target, json: &Json) -> Value {
    match target {
        Target::Prelude(kind) => primitive(kind, json),
        Target::Shape(index) => {
            let shape = service.shape(index).expect("shape lookup").expect("shape exists");
            match shape.kind().expect("shape kind") {
                "structure" | "union" => {
                    let mut fields = BTreeMap::new();
                    for (name, json) in json.as_object().expect("structure fixture object") {
                        let member = shape.members().expect("members").map(|item| item.expect("member"))
                            .find(|member| member.name().expect("member name") == name)
                            .unwrap_or_else(|| panic!("unknown fixture shape member {name}"));
                        fields.insert(Key::Field(member.id), value(service, member.target, json));
                    }
                    Value::Map(fields)
                }
                "list" => {
                    let element = shape.element().expect("list element").expect("element target");
                    Value::List(json.as_array().expect("list fixture array").iter().map(|item| value(service, element, item)).collect())
                }
                "map" => {
                    let key_target = shape.map_key().expect("map key").expect("key target");
                    let value_target = shape.map_value().expect("map value").expect("value target");
                    let mut entries = BTreeMap::new();
                    for (key, item) in json.as_object().expect("map fixture object") {
                        let key = value(service, key_target, &Json::String(key.clone()));
                        let Value::Text(key) = key else { panic!("Smithy map key must be text") };
                        entries.insert(Key::Name(key), value(service, value_target, item));
                    }
                    Value::Map(entries)
                }
                kind => primitive(kind, json),
            }
        }
    }
}

fn primitive(kind: &str, json: &Json) -> Value {
    match kind {
        "unit" => Value::Null,
        "string" | "enum" => Value::Text(json.as_str().expect("fixture string").to_owned()),
        "boolean" => Value::Bool(json.as_bool().expect("fixture boolean")),
        "byte" | "short" | "integer" | "long" | "intEnum" | "bigInteger" => Value::Int(
            json.as_i64().or_else(|| json.as_u64().and_then(|number| i64::try_from(number).ok())).expect("fixture integer")),
        "float" | "double" => Value::Float(Float::new(json.as_f64().expect("fixture float"))),
        "bigDecimal" => Value::Text(json.to_string()),
        "blob" => Value::Bytes(base64(json.as_str().expect("fixture blob"))),
        "timestamp" => timestamp(json),
        "document" => document(json),
        other => panic!("unsupported fixture target {other}"),
    }
}

fn document(json: &Json) -> Value {
    match json {
        Json::Null => Value::Null,
        Json::Bool(value) => Value::Bool(*value),
        Json::Number(value) => value.as_i64().map(Value::Int).or_else(|| value.as_u64().and_then(|n| i64::try_from(n).ok()).map(Value::Int))
            .unwrap_or_else(|| Value::Float(Float::new(value.as_f64().expect("document number")))),
        Json::String(value) => Value::Text(value.clone()),
        Json::Array(values) => Value::List(values.iter().map(document).collect()),
        Json::Object(values) => Value::Map(values.iter().map(|(key, value)| (Key::Name(key.clone()), document(value))).collect()),
    }
}

fn timestamp(json: &Json) -> Value {
    if let Some(value) = json.as_f64() {
        let seconds = value.floor();
        return Value::Time(Time::new(seconds as i64, ((value - seconds) * 1_000_000_000.0).round() as u32).expect("timestamp range"));
    }
    let value = json.as_str().expect("fixture timestamp");
    let (date, clock) = value.split_once('T').expect("RFC3339 timestamp");
    let year = date[..4].parse::<i64>().expect("year");
    let month = date[5..7].parse::<i64>().expect("month");
    let day = date[8..10].parse::<i64>().expect("day");
    let (clock, offset) = if let Some(clock) = clock.strip_suffix('Z') { (clock, 0) } else { (clock, 0) };
    let (clock, fraction) = clock.split_once('.').unwrap_or((clock, ""));
    let mut parts = clock.split(':');
    let hour = parts.next().expect("hour").parse::<i64>().expect("hour");
    let minute = parts.next().expect("minute").parse::<i64>().expect("minute");
    let second = parts.next().expect("second").parse::<i64>().expect("second");
    let mut nanos = fraction.chars().take(9).collect::<String>();
    nanos.extend(std::iter::repeat_n('0', 9usize.saturating_sub(nanos.len())));
    let nanos = if nanos.is_empty() { 0 } else { nanos.parse::<u32>().expect("fraction") };
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let days = era * 146_097 + (year - era * 400) * 365 + (year - era * 400) / 4 - (year - era * 400) / 100
        + (153 * shifted_month + 2) / 5 + day - 1 - 719_468;
    Value::Time(Time::new(days * 86_400 + hour * 3600 + minute * 60 + second - offset, nanos).expect("timestamp range"))
}

fn base64(value: &str) -> Vec<u8> {
    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u8;
    for byte in value.bytes().filter(|byte| *byte != b'=') {
        let digit = match byte { b'A'..=b'Z' => byte - b'A', b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52, b'+' => 62, b'/' => 63, _ => panic!("invalid fixture base64") };
        buffer = (buffer << 6) | u32::from(digit);
        bits += 6;
        if bits >= 8 { bits -= 8; output.push((buffer >> bits) as u8); buffer &= (1u32 << bits) - 1; }
    }
    output
}
