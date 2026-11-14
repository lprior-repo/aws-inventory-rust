use inventory_core::{
    catalog::{Operation, Service, Target},
    record::{Float, Key, Record, Time, Value},
};
use serde_json::Value as Json;
use std::collections::BTreeMap;
pub(super) fn input_record(
    service: Service<'_>,
    operation: Operation<'_>,
    params: &Json,
) -> Record {
    let mut fields = BTreeMap::new();
    if let Some(params) = params.as_object() {
        let input = operation.input().expect("operation input");
        let Target::Shape(index) = input else {
            assert!(params.is_empty(), "unit operation fixture params are empty");
            return Record::encode(&Value::Map(fields)).expect("empty input record");
        };
        let shape = service
            .shape(index)
            .expect("input shape lookup")
            .expect("input shape");
        for (name, json) in params {
            let member = shape
                .members()
                .expect("input members")
                .map(|member| member.expect("valid member"))
                .find(|member| member.name().expect("member name") == name)
                .unwrap_or_else(|| {
                    panic!(
                        "fixture param {name} is not modeled on {}",
                        shape.name().expect("shape name")
                    )
                });
            fields.insert(
                Key::Field(member.id),
                to_value(service, member.target, json),
            );
        }
    }
    Record::encode(&Value::Map(fields)).expect("fixture input record")
}

fn to_value(service: Service<'_>, target: Target, json: &Json) -> Value {
    if json.is_null() {
        return Value::Null;
    }
    match target {
        Target::Prelude(kind) => match kind {
            "unit" => Value::Null,
            "string" => Value::Text(json.as_str().expect("string input").to_owned()),
            "boolean" => Value::Bool(json.as_bool().expect("boolean input")),
            "byte" | "short" | "integer" | "long" | "intEnum" | "bigInteger" => Value::Int(
                json.as_i64()
                    .or_else(|| json.as_u64().and_then(|value| i64::try_from(value).ok()))
                    .expect("integer input"),
            ),
            "float" | "double" => float_value(json),
            "blob" => Value::Bytes(json.as_str().expect("blob input").as_bytes().to_vec()),
            "timestamp" => time_value(json),
            "document" => document_value(json),
            _ => panic!("unsupported fixture prelude kind {kind}"),
        },
        Target::Shape(index) => {
            let shape = service
                .shape(index)
                .expect("shape lookup")
                .expect("shape exists");
            match shape.kind().expect("shape kind") {
                "structure" | "union" => {
                    let object = json.as_object().expect("structure input");
                    let mut fields = BTreeMap::new();
                    for (name, json) in object {
                        let member = shape
                            .members()
                            .expect("members")
                            .map(|member| member.expect("member"))
                            .find(|member| member.name().expect("member name") == name)
                            .unwrap_or_else(|| {
                                panic!(
                                    "fixture member {name} missing from {}",
                                    shape.name().expect("shape name")
                                )
                            });
                        fields.insert(
                            Key::Field(member.id),
                            to_value(service, member.target, json),
                        );
                    }
                    Value::Map(fields)
                }
                "list" => {
                    let element = shape
                        .element()
                        .expect("list element")
                        .expect("element target");
                    Value::List(
                        json.as_array()
                            .expect("list input")
                            .iter()
                            .map(|item| to_value(service, element, item))
                            .collect(),
                    )
                }
                "map" => {
                    let key_target = shape.map_key().expect("map key").expect("key target");
                    let value_target = shape.map_value().expect("map value").expect("value target");
                    let mut entries = BTreeMap::new();
                    for (key, value) in json.as_object().expect("map input") {
                        entries.insert(
                            Key::Name(map_key_text(service, key_target, key)),
                            to_value(service, value_target, value),
                        );
                    }
                    Value::Map(entries)
                }
                "blob" => Value::Bytes(json.as_str().expect("blob input").as_bytes().to_vec()),
                "boolean" => Value::Bool(json.as_bool().expect("boolean input")),
                "byte" | "short" | "integer" | "long" | "intEnum" | "bigInteger" => Value::Int(
                    json.as_i64()
                        .or_else(|| json.as_u64().and_then(|value| i64::try_from(value).ok()))
                        .expect("integer input"),
                ),
                "float" | "double" => float_value(json),
                "bigDecimal" => Value::Text(json_number_text(json)),
                "timestamp" => time_value(json),
                "document" => document_value(json),
                "string" | "enum" => Value::Text(json.as_str().expect("string input").to_owned()),
                kind => panic!("unsupported fixture shape kind {kind}"),
            }
        }
        _ => panic!("unsupported catalog target variant in fixture"),
    }
}

fn float_value(json: &Json) -> Value {
    match json.as_str() {
        Some(value @ ("NaN" | "Infinity" | "-Infinity")) => Value::Text(value.to_owned()),
        _ => Value::Float(Float::new(json.as_f64().expect("floating input"))),
    }
}

fn map_key_text(service: Service<'_>, target: Target, key: &str) -> String {
    match target {
        Target::Prelude("string") => key.to_owned(),
        Target::Shape(index) => {
            let shape = service
                .shape(index)
                .expect("map key shape lookup")
                .expect("map key shape");
            match shape.kind().expect("map key shape kind") {
                "string" | "enum" => key.to_owned(),
                kind => panic!("unsupported Smithy map key shape {kind}"),
            }
        }
        _ => panic!("unsupported Smithy map key target"),
    }
}

fn document_value(json: &Json) -> Value {
    match json {
        Json::Null => Value::Null,
        Json::Bool(value) => Value::Bool(*value),
        Json::Number(value) => {
            if let Some(integer) = value.as_i64() {
                Value::Int(integer)
            } else if let Some(integer) = value.as_u64().and_then(|value| i64::try_from(value).ok())
            {
                Value::Int(integer)
            } else {
                Value::Float(Float::new(value.as_f64().expect("document number")))
            }
        }
        Json::String(value) => Value::Text(value.clone()),
        Json::Array(values) => Value::List(values.iter().map(document_value).collect()),
        Json::Object(values) => Value::Map(
            values
                .iter()
                .map(|(key, value)| (Key::Name(key.clone()), document_value(value)))
                .collect(),
        ),
    }
}

fn time_value(json: &Json) -> Value {
    if let Some(number) = json.as_f64() {
        let seconds = number.floor();
        let fractional = number - seconds;
        return Value::Time(
            Time::new(
                seconds as i64,
                (fractional * 1_000_000_000.0).round() as u32,
            )
            .expect("timestamp range"),
        );
    }
    let value = json.as_str().expect("timestamp string");
    let (date, zone) = value.split_once('T').expect("RFC3339 timestamp");
    let (year, month, day) = (
        date[0..4].parse::<i64>().expect("year"),
        date[5..7].parse::<i64>().expect("month"),
        date[8..10].parse::<i64>().expect("day"),
    );
    let days = days_from_civil(year, month, day);
    let (clock, offset) = if let Some(clock) = zone.strip_suffix('Z') {
        (clock, 0i64)
    } else {
        let split = zone.rfind(['+', '-']).expect("timestamp timezone");
        let sign = if zone.as_bytes()[split] == b'+' {
            1
        } else {
            -1
        };
        let (hour, minute) = zone[split + 1..].split_once(':').expect("timezone offset");
        (
            &zone[..split],
            sign * (hour.parse::<i64>().expect("offset hour") * 3600
                + minute.parse::<i64>().expect("offset minute") * 60),
        )
    };
    let (whole, fraction) = clock.split_once('.').unwrap_or((clock, ""));
    let mut parts = whole.split(':');
    let hour = parts.next().expect("hour").parse::<i64>().expect("hour");
    let minute = parts
        .next()
        .expect("minute")
        .parse::<i64>()
        .expect("minute");
    let second = parts
        .next()
        .expect("second")
        .parse::<i64>()
        .expect("second");
    let mut nanos = fraction.chars().take(9).collect::<String>();
    nanos.extend(std::iter::repeat_n('0', 9usize.saturating_sub(nanos.len())));
    let nanos = if nanos.is_empty() {
        0
    } else {
        nanos.parse::<u32>().expect("fraction")
    };
    Value::Time(
        Time::new(
            days * 86_400 + hour * 3600 + minute * 60 + second - offset,
            nanos,
        )
        .expect("timestamp range"),
    )
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn json_number_text(json: &Json) -> String {
    match json {
        Json::Number(number) => number.to_string(),
        _ => panic!("numeric input"),
    }
}

pub(super) fn decode_base64(value: &str) -> Vec<u8> {
    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u8;
    for byte in value.bytes().filter(|byte| *byte != b'=') {
        let digit = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("invalid base64 fixture"),
        };
        buffer = (buffer << 6) | u32::from(digit);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((buffer >> bits) as u8);
            buffer &= (1u32 << bits) - 1;
        }
    }
    output
}
