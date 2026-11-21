#[path = "fixture_oracle/collections.rs"]
mod collections;
use collections::{compare_list, compare_object};
mod timestamp_oracle;
use inventory_core::{
    catalog::{Service, Target, TimestampFormat},
    record::{Float, MapRef, Record, Time, ValueRef},
};
use serde_json::Value as Json;

pub fn assert_fixture_root(
    record: &Record,
    service: Service<'_>,
    target: Target,
    expected: &Json,
    label: &str,
) {
    let empty = Json::Object(Default::default());
    let expected = if expected.is_null() { &empty } else { expected };
    let Target::Shape(index) = target else {
        assert_eq!(
            expected.as_object().map_or(0, serde_json::Map::len),
            0,
            "{label}"
        );
        assert!(
            record
                .root()
                .iter()
                .is_ok_and(|entries| entries.count() == 0),
            "{label}"
        );
        return;
    };
    compare_value(
        ValueRef::Map(record.root()),
        expected,
        Target::Shape(index),
        service,
        None,
        label,
    );
}

fn compare_value(
    actual: ValueRef<'_>,
    expected: &Json,
    target: Target,
    service: Service<'_>,
    format: Option<TimestampFormat>,
    label: &str,
) {
    let kind = kind(service, target);
    let matches = kind.is_some_and(|kind| match expected {
        Json::Null => matches!(actual, ValueRef::Null),
        Json::Bool(value) => matches!(actual, ValueRef::Bool(found) if found == *value),
        Json::Number(value) => compare_number(actual, value, kind),
        Json::String(value) => compare_string(actual, value, kind, target, service, format),
        Json::Array(values) => compare_list(actual, values, target, service, label),
        Json::Object(values) => compare_object(actual, values, target, service, label),
    });
    assert!(
        matches,
        "{label}: decoded {} value {actual:?} differs from published fixture {expected}",
        kind.unwrap_or("unknown shape"),
    );
}

fn compare_number(actual: ValueRef<'_>, expected: &serde_json::Number, kind: &str) -> bool {
    if kind == "timestamp" {
        return matches!(actual, ValueRef::Time(value) if timestamp_time(&expected.to_string(), Some(TimestampFormat::EpochSeconds)).is_some_and(|wanted| wanted == value));
    }
    if matches!(kind, "float" | "double") {
        return expected.as_f64().is_some_and(
            |wanted| matches!(actual, ValueRef::Float(value) if value == Float::new(wanted)),
        );
    }
    expected
        .as_i64()
        .is_some_and(|wanted| matches!(actual, ValueRef::Int(value) if value == wanted))
        || (kind == "document"
            && expected.as_f64().is_some_and(
                |wanted| matches!(actual, ValueRef::Float(value) if value == Float::new(wanted)),
            ))
}

fn compare_string(
    actual: ValueRef<'_>,
    expected: &str,
    kind: &str,
    target: Target,
    service: Service<'_>,
    format: Option<TimestampFormat>,
) -> bool {
    match kind {
        "blob" => matches!(actual, ValueRef::Bytes(value) if value == expected.as_bytes()),
        "timestamp" => {
            matches!(actual, ValueRef::Time(value) if timestamp_time(expected, format.or(timestamp_format(service, target))).is_some_and(|wanted| wanted == value))
        }
        "float" | "double" => {
            matches!(actual, ValueRef::Float(value) if special_float(expected).is_some_and(|wanted| value == Float::new(wanted)))
        }
        _ => matches!(actual, ValueRef::Text(value) if value == expected),
    }
}

fn kind<'a>(service: Service<'a>, target: Target) -> Option<&'a str> {
    match target {
        Target::Prelude(name) => Some(name),
        Target::Shape(index) => service.shape(index).ok()??.kind().ok(),
        _ => Some("unknown"),
    }
}

fn element_target(service: Service<'_>, target: Target) -> Option<Target> {
    match target {
        Target::Prelude("document") => Some(target),
        Target::Shape(index) => {
            let shape = service.shape(index).ok().flatten()?;
            if shape.kind().ok()? == "document" {
                Some(Target::Prelude("document"))
            } else {
                shape.element().ok().flatten()
            }
        }
        _ => None,
    }
}

fn timestamp_format(service: Service<'_>, target: Target) -> Option<TimestampFormat> {
    match target {
        Target::Prelude(_) => None,
        Target::Shape(index) => service
            .shape(index)
            .ok()
            .flatten()?
            .wire()
            .ok()?
            .timestamp_format()
            .ok()
            .flatten(),
        _ => None,
    }
}

fn special_float(value: &str) -> Option<f64> {
    match value {
        "NaN" => Some(f64::NAN),
        "Infinity" => Some(f64::INFINITY),
        "-Infinity" => Some(f64::NEG_INFINITY),
        _ => None,
    }
}

fn timestamp_time(value: &str, format: Option<TimestampFormat>) -> Option<Time> {
    timestamp_oracle::parse(value, format)
}
