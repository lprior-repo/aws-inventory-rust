use inventory_core::{
    catalog::{Member, Operation, Service, Shape, Target},
    record::{Float, Key, Record, Time, Value},
};
use serde_json::Value as Json;
use std::{collections::BTreeMap, str::FromStr};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub(super) fn record(
    service: Service<'_>,
    operation: Operation<'_>,
    params: &Json,
) -> TestResult<Record> {
    let mut fields = BTreeMap::new();
    if let Some(params) = params.as_object() {
        let input = operation.input()?;
        let Target::Shape(index) = input else {
            if !params.is_empty() {
                return Err("unit operation has fixture params".into());
            }
            return Ok(Record::encode(&Value::Map(fields))?);
        };
        let shape = shape_at(service, index)?;
        for (name, json) in params {
            let member = member_named(shape, name)?;
            fields.insert(Key::Field(member.id), value(service, member.target, json)?);
        }
    }
    Ok(Record::encode(&Value::Map(fields))?)
}

fn shape_at(service: Service<'_>, index: usize) -> TestResult<Shape<'_>> {
    service
        .shape(index)?
        .ok_or_else(|| "fixture shape missing".into())
}

fn member_named<'a>(shape: Shape<'a>, name: &str) -> TestResult<Member<'a>> {
    for item in shape.members()? {
        let member = item?;
        if member.name()? == name {
            return Ok(member);
        }
    }
    Err(format!("unknown fixture member {name}").into())
}

fn value(service: Service<'_>, target: Target, json: &Json) -> TestResult<Value> {
    if json.is_null() {
        return Ok(Value::Null);
    }
    match target {
        Target::Prelude(kind) => primitive(kind, json),
        Target::Shape(index) => {
            let shape = shape_at(service, index)?;
            match shape.kind()? {
                "structure" | "union" => {
                    let mut fields = BTreeMap::new();
                    let object = json.as_object().ok_or("structure fixture object")?;
                    for (name, json) in object {
                        let member = member_named(shape, name)?;
                        fields.insert(Key::Field(member.id), value(service, member.target, json)?);
                    }
                    Ok(Value::Map(fields))
                }
                "list" => {
                    let element = shape.element()?.ok_or("list element target")?;
                    let items = json.as_array().ok_or("list fixture array")?;
                    Ok(Value::List(
                        items
                            .iter()
                            .map(|item| value(service, element, item))
                            .collect::<TestResult<Vec<_>>>()?,
                    ))
                }
                "map" => {
                    let key_target = shape.map_key()?.ok_or("map key target")?;
                    let value_target = shape.map_value()?.ok_or("map value target")?;
                    let object = json.as_object().ok_or("map fixture object")?;
                    let mut entries = BTreeMap::new();
                    for (key, item) in object {
                        let Value::Text(key) =
                            value(service, key_target, &Json::String(key.clone()))?
                        else {
                            return Err("Smithy map key must be text".into());
                        };
                        entries.insert(Key::Name(key), value(service, value_target, item)?);
                    }
                    Ok(Value::Map(entries))
                }
                kind => primitive(kind, json),
            }
        }
        _ => Err("unknown fixture target".into()),
    }
}

fn primitive(kind: &str, json: &Json) -> TestResult<Value> {
    Ok(match kind {
        "unit" => {
            if !json.as_object().is_some_and(serde_json::Map::is_empty) {
                return Err(format!("fixture unit has members: {json}").into());
            }
            Value::Map(BTreeMap::new())
        }
        "string" | "enum" => Value::Text(
            json.as_str()
                .ok_or_else(|| format!("fixture {kind} is not text: {json}"))?
                .to_owned(),
        ),
        "boolean" => Value::Bool(json.as_bool().ok_or("fixture boolean")?),
        "byte" | "short" | "integer" | "long" | "intEnum" | "bigInteger" => Value::Int(
            json.as_i64()
                .or_else(|| json.as_u64().and_then(|number| i64::try_from(number).ok()))
                .ok_or("fixture integer")?,
        ),
        "float" | "double" => float_value(json)?,
        "bigDecimal" => Value::Text(json.to_string()),
        "blob" => Value::Bytes(json.as_str().ok_or("fixture blob")?.as_bytes().to_vec()),
        "timestamp" => timestamp(json)?,
        "document" => document(json)?,
        other => return Err(format!("unsupported fixture target {other}").into()),
    })
}

fn float_value(json: &Json) -> TestResult<Value> {
    match json.as_str() {
        Some(value @ ("NaN" | "Infinity" | "-Infinity")) => Ok(Value::Text(value.to_owned())),
        _ => Ok(Value::Float(Float::new(
            json.as_f64().ok_or("fixture float")?,
        ))),
    }
}

fn document(json: &Json) -> TestResult<Value> {
    Ok(match json {
        Json::Null => Value::Null,
        Json::Bool(value) => Value::Bool(*value),
        Json::Number(value) => match value
            .as_i64()
            .or_else(|| value.as_u64().and_then(|n| i64::try_from(n).ok()))
        {
            Some(int) => Value::Int(int),
            None => Value::Float(Float::new(value.as_f64().ok_or("document number")?)),
        },
        Json::String(value) => Value::Text(value.clone()),
        Json::Array(values) => Value::List(
            values
                .iter()
                .map(document)
                .collect::<TestResult<Vec<_>>>()?,
        ),
        Json::Object(values) => Value::Map(
            values
                .iter()
                .map(|(key, value)| Ok((Key::Name(key.clone()), document(value)?)))
                .collect::<TestResult<BTreeMap<_, _>>>()?,
        ),
    })
}

fn field<T: FromStr>(text: Option<&str>) -> TestResult<T>
where
    T::Err: std::error::Error + 'static,
{
    Ok(text.ok_or("missing timestamp field")?.parse()?)
}

fn timestamp(json: &Json) -> TestResult<Value> {
    if let Some(value) = json.as_f64() {
        let seconds = value.floor();
        let seconds_text = format!("{seconds:.0}");
        let nanos = ((value - seconds) * 1_000_000_000.0).round();
        let nanos_text = format!("{nanos:.0}");
        return Ok(Value::Time(Time::new(
            seconds_text.parse()?,
            nanos_text.parse()?,
        )?));
    }
    let value = json.as_str().ok_or("fixture timestamp")?;
    let (date, clock) = value.split_once('T').ok_or("RFC3339 timestamp")?;
    let year: i64 = field(date.get(..4))?;
    let month: i64 = field(date.get(5..7))?;
    let day: i64 = field(date.get(8..10))?;
    let clock = clock.strip_suffix('Z').unwrap_or(clock);
    let (clock, fraction) = clock.split_once('.').unwrap_or((clock, ""));
    let mut parts = clock.split(':');
    let hour: i64 = field(parts.next())?;
    let minute: i64 = field(parts.next())?;
    let second: i64 = field(parts.next())?;
    let mut nanos = fraction.chars().take(9).collect::<String>();
    nanos.extend(std::iter::repeat_n('0', 9usize.saturating_sub(nanos.len())));
    let nanos: u32 = field(Some(&nanos))?;
    let days = days_from_civil(year, month, day).ok_or("timestamp overflow")?;
    let seconds = days
        .checked_mul(86_400)
        .and_then(|s| s.checked_add(hour.checked_mul(3600)?))
        .and_then(|s| s.checked_add(minute.checked_mul(60)?))
        .and_then(|s| s.checked_add(second))
        .ok_or("timestamp overflow")?;
    Ok(Value::Time(Time::new(seconds, nanos)?))
}

fn days_from_civil(year: i64, month: i64, day: i64) -> Option<i64> {
    let year = year.checked_sub(i64::from(month <= 2))?;
    let era = year.div_euclid(400);
    let yoe = year.checked_sub(era.checked_mul(400)?)?;
    let shifted_month = month.checked_add(if month > 2 { -3 } else { 9 })?;
    let doy = 153i64
        .checked_mul(shifted_month)?
        .checked_add(2)?
        .checked_div(5)?;
    era.checked_mul(146_097)?
        .checked_add(yoe.checked_mul(365)?)?
        .checked_add(yoe.checked_div(4)?)?
        .checked_sub(yoe.checked_div(100)?)?
        .checked_add(doy)?
        .checked_add(day)?
        .checked_sub(1)?
        .checked_sub(719_468)
}
