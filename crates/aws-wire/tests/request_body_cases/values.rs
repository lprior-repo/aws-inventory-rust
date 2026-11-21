use inventory_core::{
    catalog::{Operation, Service, Target},
    record::{Float, Key, Record, Time, Value},
};
use serde_json::Value as Json;
use std::collections::BTreeMap;

pub(super) type BoxError = Box<dyn std::error::Error>;

pub(super) fn expect_ok<T, E: std::fmt::Debug>(
    result: Result<T, E>,
    what: &str,
) -> Result<T, BoxError> {
    result.map_err(|error| format!("{what}: {error:?}").into())
}

fn checked<T>(value: Option<T>) -> Result<T, BoxError> {
    value.ok_or_else(|| "fixture arithmetic overflow".into())
}

pub(super) fn input_record(
    service: Service<'_>,
    operation: Operation<'_>,
    params: &Json,
) -> Result<Record, BoxError> {
    let mut fields = BTreeMap::new();
    if let Some(params) = params.as_object() {
        let input = expect_ok(operation.input(), "operation input")?;
        let Target::Shape(index) = input else {
            assert!(params.is_empty(), "unit operation fixture params are empty");
            return expect_ok(Record::encode(&Value::Map(fields)), "empty input record");
        };
        let shape = expect_ok(service.shape(index), "input shape lookup")?.ok_or("input shape")?;
        for (name, json) in params {
            let mut found = None;
            for item in expect_ok(shape.members(), "input members")? {
                let item = expect_ok(item, "valid member")?;
                if expect_ok(item.name(), "member name")? == name {
                    found = Some(item);
                    break;
                }
            }
            let Some(member) = found else {
                return Err(format!(
                    "fixture param {name} is not modeled on {}",
                    expect_ok(shape.name(), "shape name")?
                )
                .into());
            };
            fields.insert(
                Key::Field(member.id),
                to_value(service, member.target, json)?,
            );
        }
    }
    expect_ok(Record::encode(&Value::Map(fields)), "fixture input record")
}

fn to_value(service: Service<'_>, target: Target, json: &Json) -> Result<Value, BoxError> {
    if json.is_null() {
        return Ok(Value::Null);
    }
    match target {
        Target::Prelude(kind) => match kind {
            "unit" => Ok(Value::Null),
            "string" => Ok(Value::Text(json.as_str().ok_or("string input")?.to_owned())),
            "boolean" => Ok(Value::Bool(json.as_bool().ok_or("boolean input")?)),
            "byte" | "short" | "integer" | "long" | "intEnum" | "bigInteger" => {
                Ok(Value::Int(integer(json)?))
            }
            "float" | "double" => Ok(float_value(json)?),
            "blob" => Ok(Value::Bytes(
                json.as_str().ok_or("blob input")?.as_bytes().to_vec(),
            )),
            "timestamp" => time_value(json),
            "document" => document_value(json),
            _ => Err(format!("unsupported fixture prelude kind {kind}").into()),
        },
        Target::Shape(index) => {
            let shape = expect_ok(service.shape(index), "shape lookup")?.ok_or("shape exists")?;
            match expect_ok(shape.kind(), "shape kind")? {
                "structure" | "union" => {
                    let object = json.as_object().ok_or("structure input")?;
                    let mut fields = BTreeMap::new();
                    for (name, json) in object {
                        let mut found = None;
                        for item in expect_ok(shape.members(), "members")? {
                            let item = expect_ok(item, "member")?;
                            if expect_ok(item.name(), "member name")? == name {
                                found = Some(item);
                                break;
                            }
                        }
                        let Some(member) = found else {
                            return Err(format!(
                                "fixture member {name} missing from {}",
                                expect_ok(shape.name(), "shape name")?
                            )
                            .into());
                        };
                        fields.insert(
                            Key::Field(member.id),
                            to_value(service, member.target, json)?,
                        );
                    }
                    Ok(Value::Map(fields))
                }
                "list" => {
                    let element =
                        expect_ok(shape.element(), "list element")?.ok_or("element target")?;
                    Ok(Value::List(
                        json.as_array()
                            .ok_or("list input")?
                            .iter()
                            .map(|item| to_value(service, element, item))
                            .collect::<Result<Vec<_>, _>>()?,
                    ))
                }
                "map" => {
                    let key_target = expect_ok(shape.map_key(), "map key")?.ok_or("key target")?;
                    let value_target =
                        expect_ok(shape.map_value(), "map value")?.ok_or("value target")?;
                    let mut entries = BTreeMap::new();
                    for (key, value) in json.as_object().ok_or("map input")? {
                        entries.insert(
                            Key::Name(map_key_text(service, key_target, key)?),
                            to_value(service, value_target, value)?,
                        );
                    }
                    Ok(Value::Map(entries))
                }
                "blob" => Ok(Value::Bytes(
                    json.as_str().ok_or("blob input")?.as_bytes().to_vec(),
                )),
                "boolean" => Ok(Value::Bool(json.as_bool().ok_or("boolean input")?)),
                "byte" | "short" | "integer" | "long" | "intEnum" | "bigInteger" => {
                    Ok(Value::Int(integer(json)?))
                }
                "float" | "double" => float_value(json),
                "bigDecimal" => Ok(Value::Text(json_number_text(json)?)),
                "timestamp" => time_value(json),
                "document" => document_value(json),
                "string" | "enum" => {
                    Ok(Value::Text(json.as_str().ok_or("string input")?.to_owned()))
                }
                kind => Err(format!("unsupported fixture shape kind {kind}").into()),
            }
        }
        _ => Err("unsupported catalog target variant in fixture".into()),
    }
}

fn integer(json: &Json) -> Result<i64, BoxError> {
    json.as_i64()
        .or_else(|| json.as_u64().and_then(|value| i64::try_from(value).ok()))
        .ok_or_else(|| "integer input".into())
}

fn float_value(json: &Json) -> Result<Value, BoxError> {
    match json.as_str() {
        Some(value @ ("NaN" | "Infinity" | "-Infinity")) => Ok(Value::Text(value.to_owned())),
        _ => Ok(Value::Float(Float::new(
            json.as_f64().ok_or("floating input")?,
        ))),
    }
}

fn map_key_text(service: Service<'_>, target: Target, key: &str) -> Result<String, BoxError> {
    match target {
        Target::Prelude("string") => Ok(key.to_owned()),
        Target::Shape(index) => {
            let shape =
                expect_ok(service.shape(index), "map key shape lookup")?.ok_or("map key shape")?;
            match expect_ok(shape.kind(), "map key shape kind")? {
                "string" | "enum" => Ok(key.to_owned()),
                kind => Err(format!("unsupported Smithy map key shape {kind}").into()),
            }
        }
        _ => Err("unsupported Smithy map key target".into()),
    }
}

fn document_value(json: &Json) -> Result<Value, BoxError> {
    match json {
        Json::Null => Ok(Value::Null),
        Json::Bool(value) => Ok(Value::Bool(*value)),
        Json::Number(value) => {
            if let Some(integer) = value.as_i64() {
                Ok(Value::Int(integer))
            } else if let Some(integer) = value.as_u64().and_then(|value| i64::try_from(value).ok())
            {
                Ok(Value::Int(integer))
            } else {
                Ok(Value::Float(Float::new(
                    value.as_f64().ok_or("document number")?,
                )))
            }
        }
        Json::String(value) => Ok(Value::Text(value.clone())),
        Json::Array(values) => Ok(Value::List(
            values
                .iter()
                .map(document_value)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Json::Object(values) => Ok(Value::Map(
            values
                .iter()
                .map(|(key, value)| Ok((Key::Name(key.clone()), document_value(value)?)))
                .collect::<Result<BTreeMap<_, _>, BoxError>>()?,
        )),
    }
}

fn time_value(json: &Json) -> Result<Value, BoxError> {
    if let Some(number) = json.as_f64() {
        let seconds = number.floor();
        let fractional = number - seconds;
        let nanos = (fractional * 1_000_000_000.0).round();
        return Ok(Value::Time(expect_ok(
            Time::new(
                expect_ok(format!("{seconds:.0}").parse::<i64>(), "timestamp seconds")?,
                expect_ok(format!("{nanos:.0}").parse::<u32>(), "timestamp nanos")?,
            ),
            "timestamp range",
        )?));
    }
    let value = json.as_str().ok_or("timestamp string")?;
    let (date, zone) = value.split_once('T').ok_or("RFC3339 timestamp")?;
    let year = date.get(0..4).ok_or("year")?.parse::<i64>()?;
    let month = date.get(5..7).ok_or("month")?.parse::<i64>()?;
    let day = date.get(8..10).ok_or("day")?.parse::<i64>()?;
    let days = days_from_civil(year, month, day).ok_or("date arithmetic overflow")?;
    let (clock, offset) = if let Some(clock) = zone.strip_suffix('Z') {
        (clock, 0i64)
    } else {
        let split = zone.rfind(['+', '-']).ok_or("timestamp timezone")?;
        let (clock, sign_and_offset) = zone.split_at_checked(split).ok_or("timestamp timezone")?;
        let sign: i64 = if sign_and_offset.starts_with('+') {
            1
        } else {
            -1
        };
        let (hour, minute) = sign_and_offset
            .get(1..)
            .ok_or("timezone offset")?
            .split_once(':')
            .ok_or("timezone offset")?;
        let hours = checked(hour.parse::<i64>()?.checked_mul(3600))?;
        let minutes = checked(minute.parse::<i64>()?.checked_mul(60))?;
        let magnitude = checked(hours.checked_add(minutes))?;
        (clock, checked(sign.checked_mul(magnitude))?)
    };
    let (whole, fraction) = clock.split_once('.').unwrap_or((clock, ""));
    let mut parts = whole.split(':');
    let hour = parts.next().ok_or("hour")?.parse::<i64>()?;
    let minute = parts.next().ok_or("minute")?.parse::<i64>()?;
    let second = parts.next().ok_or("second")?.parse::<i64>()?;
    let mut nanos = fraction.chars().take(9).collect::<String>();
    nanos.extend(std::iter::repeat_n('0', 9usize.saturating_sub(nanos.len())));
    let nanos = if nanos.is_empty() {
        0
    } else {
        nanos.parse::<u32>()?
    };
    let total = checked(
        days.checked_mul(86_400)
            .and_then(|value| value.checked_add(hour.checked_mul(3_600)?))
            .and_then(|value| value.checked_add(minute.checked_mul(60)?))
            .and_then(|value| value.checked_add(second))
            .and_then(|value| value.checked_sub(offset)),
    )?;
    Ok(Value::Time(expect_ok(
        Time::new(total, nanos),
        "timestamp range",
    )?))
}

fn days_from_civil(year: i64, month: i64, day: i64) -> Option<i64> {
    let year = year.checked_sub(i64::from(month <= 2))?;
    let era = year.div_euclid(400);
    let year_of_era = year.checked_sub(era.checked_mul(400)?)?;
    let shifted_month = month.checked_add(if month > 2 { -3 } else { 9 })?;
    let day_of_year = 153i64
        .checked_mul(shifted_month)?
        .checked_add(2)?
        .div_euclid(5)
        .checked_add(day)?
        .checked_sub(1)?;
    let day_of_era = year_of_era
        .checked_mul(365)?
        .checked_add(year_of_era.div_euclid(4))?
        .checked_sub(year_of_era.div_euclid(100))?
        .checked_add(day_of_year)?;
    era.checked_mul(146_097)?
        .checked_add(day_of_era)?
        .checked_sub(719_468)
}

fn json_number_text(json: &Json) -> Result<String, BoxError> {
    match json {
        Json::Number(number) => Ok(number.to_string()),
        _ => Err("numeric input".into()),
    }
}

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(super) fn decode_base64(value: &str) -> Result<Vec<u8>, BoxError> {
    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u8;
    for byte in value.bytes().filter(|byte| *byte != b'=') {
        let digit = BASE64_ALPHABET
            .iter()
            .position(|candidate| *candidate == byte)
            .ok_or("invalid base64 fixture")?;
        buffer = (buffer << 6) | u32::try_from(digit)?;
        bits = bits.saturating_add(6);
        if bits >= 8 {
            bits = bits.saturating_sub(8);
            output.push(u8::try_from(buffer >> bits)?);
            buffer &= (1u32 << bits).saturating_sub(1);
        }
    }
    Ok(output)
}
