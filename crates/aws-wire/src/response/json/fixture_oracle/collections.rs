use super::*;
pub(super) fn compare_list(
    actual: ValueRef<'_>,
    expected: &[Json],
    target: Target,
    service: Service<'_>,
    label: &str,
) -> bool {
    let ValueRef::List(list) = actual else {
        return false;
    };
    let Some(element) = element_target(service, target) else {
        return false;
    };
    let format = match target {
        Target::Shape(index) => {
            let Ok(Some(shape)) = service.shape(index) else {
                return false;
            };
            if shape.kind().is_ok_and(|kind| kind == "list") {
                shape
                    .element_wire()
                    .ok()
                    .and_then(|wire| wire.timestamp_format().ok())
                    .flatten()
            } else {
                None
            }
        }
        _ => None,
    };
    let Ok(items) = list.iter() else {
        return false;
    };
    let mut count = 0usize;
    for (index, actual) in items.enumerate() {
        let (Ok(actual), Some(expected)) = (actual, expected.get(index)) else {
            return false;
        };
        compare_value(
            actual,
            expected,
            element,
            service,
            format,
            &format!("{label}[{index}]"),
        );
        let Some(next) = count.checked_add(1) else {
            return false;
        };
        count = next;
    }
    count == expected.len()
}

pub(super) fn compare_object(
    actual: ValueRef<'_>,
    expected: &serde_json::Map<String, Json>,
    target: Target,
    service: Service<'_>,
    label: &str,
) -> bool {
    let ValueRef::Map(map) = actual else {
        return false;
    };
    let Ok(items) = map.iter() else {
        return false;
    };
    let mut actual_count = 0usize;
    for item in items {
        if item.is_err() {
            return false;
        }
        let Some(next) = actual_count.checked_add(1) else {
            return false;
        };
        actual_count = next;
    }
    let expected_count = if matches!(kind(service, target), Some("structure" | "union")) {
        expected.values().filter(|value| !value.is_null()).count()
    } else {
        expected.len()
    };
    if actual_count != expected_count {
        return false;
    }
    match kind(service, target) {
        Some("structure" | "union") => expected
            .iter()
            .all(|(name, value)| compare_member(map, name, value, target, service, label)),
        Some("map" | "document") => expected
            .iter()
            .all(|(name, value)| compare_named(map, name, value, target, service, label)),
        Some("unit") => expected.is_empty(),
        _ => false,
    }
}

fn compare_member(
    map: MapRef<'_>,
    name: &str,
    expected: &Json,
    target: Target,
    service: Service<'_>,
    label: &str,
) -> bool {
    let Target::Shape(index) = target else {
        return false;
    };
    let Ok(Some(shape)) = service.shape(index) else {
        return false;
    };
    let Ok(members) = shape.members() else {
        return false;
    };
    let member = members.filter_map(Result::ok).find(|member| {
        member.name().is_ok_and(|actual| actual == name)
            || member
                .wire()
                .and_then(|wire| wire.json_name())
                .is_ok_and(|actual| actual == Some(name))
    });
    let Some(member) = member else {
        return false;
    };
    if expected.is_null() {
        let optional = !member.required
            || member
                .wire()
                .and_then(|wire| wire.client_optional())
                .unwrap_or(false);
        return optional && map.get_field(member.id).is_ok_and(|value| value.is_none());
    }
    let Some(actual) = map.get_field(member.id).ok().flatten() else {
        return false;
    };
    let format = member
        .wire()
        .ok()
        .and_then(|wire| wire.timestamp_format().ok())
        .flatten();
    compare_value(
        actual,
        expected,
        member.target,
        service,
        format,
        &format!("{label}.{name}"),
    );
    true
}

fn compare_named(
    map: MapRef<'_>,
    name: &str,
    expected: &Json,
    target: Target,
    service: Service<'_>,
    label: &str,
) -> bool {
    let Some(actual) = map.get_name(name).ok().flatten() else {
        return false;
    };
    let (child, format) = match target {
        Target::Prelude("document") => (Target::Prelude("document"), None),
        Target::Shape(index) => {
            let Ok(Some(shape)) = service.shape(index) else {
                return false;
            };
            match shape.kind().ok() {
                Some("map") => {
                    let Some(value) = shape.map_value().ok().flatten() else {
                        return false;
                    };
                    let format = shape
                        .map_value_wire()
                        .ok()
                        .and_then(|wire| wire.timestamp_format().ok())
                        .flatten();
                    (value, format)
                }
                Some("document") => (Target::Prelude("document"), None),
                _ => return false,
            }
        }
        _ => return false,
    };
    compare_value(
        actual,
        expected,
        child,
        service,
        format,
        &format!("{label}.{name}"),
    );
    true
}
