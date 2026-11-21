use super::*;

pub(super) fn fixture_registry(root: &Json) -> FixtureResult<Registry> {
    let registry = Registry::parse(include_str!("../../../../../../catalog/field-ids.txt"))?;
    let mut names = BTreeSet::new();
    let mut pending = vec![root];
    while let Some(value) = pending.pop() {
        match value {
            Json::Object(map) => {
                if let Some(members) = map.get("members").and_then(Json::as_object) {
                    names.extend(members.keys().cloned());
                }
                pending.extend(map.values());
            }
            Json::Array(values) => pending.extend(values),
            _ => {}
        }
    }
    Ok(registry.extend(names)?)
}

pub(super) fn find_operation<'a>(service: Service<'a>, name: &str) -> FixtureResult<Operation<'a>> {
    for entry in service.operations()? {
        let operation = entry?;
        if operation.name()? == name {
            return Ok(operation);
        }
    }
    Err(fixture_error(format!("operation {name} absent")))
}

pub(super) fn find_list_operation(service: Service<'_>) -> FixtureResult<Operation<'_>> {
    for item in service.operations()? {
        let operation = item?;
        let Target::Shape(index) = operation.input()? else {
            continue;
        };
        let Some(shape) = service.shape(index)? else {
            continue;
        };
        for member in shape.members()? {
            let member = member?;
            if let Target::Shape(index) = member.target
                && let Some(shape) = service.shape(index)?
                && shape.kind()? == "list"
            {
                return Ok(operation);
            }
        }
    }
    Err(fixture_error("service has no list input operation"))
}

pub(super) fn convert(
    service: Service<'_>,
    registry: &Registry,
    target: Target,
    json: &Json,
) -> FixtureResult<Value> {
    if json.is_null() {
        return Ok(Value::Null);
    }
    match target {
        Target::Prelude(kind) => scalar_value(kind, json),
        Target::Shape(index) => {
            let shape = service
                .shape(index)?
                .ok_or_else(|| fixture_error(format!("shape {index} absent")))?;
            match shape.kind()? {
                "structure" | "union" => {
                    let object = json
                        .as_object()
                        .ok_or_else(|| fixture_error("expected structure value"))?;
                    let mut fields = BTreeMap::new();
                    for item in shape.members()? {
                        let member = item?;
                        let name = service.field_name(member.id)?;
                        if let Some(value) = object.get(name) {
                            let id = registry.id(name).ok_or_else(|| {
                                fixture_error(format!("registry missing member {name}"))
                            })?;
                            fields.insert(
                                Key::Field(id),
                                convert(service, registry, member.target, value)?,
                            );
                        }
                    }
                    Ok(Value::Map(fields))
                }
                "list" => {
                    let items = json
                        .as_array()
                        .ok_or_else(|| fixture_error("expected list value"))?;
                    let target = shape
                        .element()?
                        .ok_or_else(|| fixture_error("list target absent"))?;
                    items
                        .iter()
                        .map(|item| convert(service, registry, target, item))
                        .collect::<FixtureResult<Vec<_>>>()
                        .map(Value::List)
                }
                "map" => {
                    let object = json
                        .as_object()
                        .ok_or_else(|| fixture_error("expected map value"))?;
                    let target = shape
                        .map_value()?
                        .ok_or_else(|| fixture_error("map value target absent"))?;
                    let mut entries = BTreeMap::new();
                    for (key, value) in object {
                        entries.insert(
                            Key::Name(key.clone()),
                            convert(service, registry, target, value)?,
                        );
                    }
                    Ok(Value::Map(entries))
                }
                kind => scalar_value(kind, json),
            }
        }
        _ => Err(fixture_error("unsupported fixture target")),
    }
}

fn scalar_value(kind: &str, json: &Json) -> FixtureResult<Value> {
    match kind {
        "unit" => match json.as_object() {
            Some(members) if members.is_empty() => Ok(Value::Map(BTreeMap::new())),
            _ => Err(fixture_error("unit input must be an empty object")),
        },
        "string" | "enum" | "bigDecimal" => json
            .as_str()
            .map(|value| Value::Text(value.to_owned()))
            .ok_or_else(|| fixture_error("expected text scalar")),
        "boolean" => json
            .as_bool()
            .map(Value::Bool)
            .ok_or_else(|| fixture_error("expected boolean scalar")),
        "byte" | "short" | "integer" | "long" | "bigInteger" | "intEnum" => json
            .as_i64()
            .map(Value::Int)
            .ok_or_else(|| fixture_error("expected integer scalar")),
        "float" | "double" => match json.as_str() {
            Some(value @ ("NaN" | "Infinity" | "-Infinity")) => Ok(Value::Text(value.to_owned())),
            _ => json
                .as_f64()
                .map(|value| Value::Float(Float::new(value)))
                .ok_or_else(|| fixture_error("expected float scalar")),
        },
        "timestamp" => {
            let seconds = json
                .as_i64()
                .ok_or_else(|| fixture_error("expected integer timestamp"))?;
            Ok(Value::Time(Time::new(seconds, 0)?))
        }
        "blob" => Ok(Value::Bytes(
            json.as_str()
                .ok_or_else(|| fixture_error("expected blob string"))?
                .as_bytes()
                .to_vec(),
        )),
        _ => Err(fixture_error(format!("unsupported fixture target {kind}"))),
    }
}
