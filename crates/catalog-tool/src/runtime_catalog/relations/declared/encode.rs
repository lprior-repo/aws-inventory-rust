use super::*;

pub(super) fn relation(
    services: &ServiceIndex<'_>,
    source_kind: &str,
    target: &ResourceSource<'_>,
    bindings: &[(&str, &str)],
) -> Result<Value, RuntimeCatalogCompileError> {
    let ((source, identity), rest) = bindings
        .split_first()
        .ok_or(RuntimeCatalogCompileError::Capability)?;
    let components = rest
        .iter()
        .map(|(source, target)| component(services, source, target))
        .collect::<Result<Vec<_>, _>>()?;
    let scope = match target.scope {
        Some(Scope::Account) => 1,
        Some(Scope::HomeRegion) => 2,
        Some(Scope::Region) => 0,
        _ => return Err(RuntimeCatalogCompileError::Capability),
    };
    map([
        (1, Value::Text(source_kind.to_owned())),
        (2, selected(services, source)?),
        (3, Value::Text(target.kind.to_owned())),
        (4, selected(services, identity)?),
        (5, Value::Text("REFERENCES".to_owned())),
        (6, Value::Int(scope)),
        (7, Value::Int(0)),
        (11, Value::Int(i64::from(*source == "$"))),
        (12, Value::Int(i64::from(*identity == "$"))),
    ]
    .into_iter()
    .chain((!rest.is_empty()).then_some((14, Value::List(components)))))
}

fn component(
    services: &ServiceIndex<'_>,
    source: &str,
    target: &str,
) -> Result<Value, RuntimeCatalogCompileError> {
    map([
        (1, Value::Int(0)),
        (2, selected(services, source)?),
        (3, Value::Int(i64::from(source == "$"))),
        (4, selected(services, target)?),
        (5, Value::Int(i64::from(target == "$"))),
    ])
}

fn selected(services: &ServiceIndex<'_>, value: &str) -> Result<Value, RuntimeCatalogCompileError> {
    if value == "$" {
        Ok(Value::List(Vec::new()))
    } else {
        path(services, value)
    }
}
