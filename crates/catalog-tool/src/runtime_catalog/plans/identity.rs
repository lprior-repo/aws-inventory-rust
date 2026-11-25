use super::{RuntimeCatalogCompileError, encode, selected_path};
use inventory_core::record::Value;

pub(in crate::runtime_catalog) fn fields(template: &str) -> impl Iterator<Item = &str> {
    template
        .split('{')
        .skip(1)
        .filter_map(|part| part.split_once('}').map(|(path, _)| path))
        .filter(|path| !matches!(*path, "partition" | "account" | "region"))
}

pub(super) fn compile(
    template: &str,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let mut segments = template.split('{');
    let first = segments
        .next()
        .ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: template.to_owned(),
        })?;
    let initial = if first.is_empty() {
        Vec::new()
    } else {
        vec![literal(first)?]
    };
    let parts = segments.try_fold(initial, |mut parts, segment| {
        let (path, tail) =
            segment
                .split_once('}')
                .ok_or_else(|| RuntimeCatalogCompileError::Path {
                    path: template.to_owned(),
                })?;
        parts.push(placeholder(path, services)?);
        if !tail.is_empty() {
            parts.push(literal(tail)?);
        }
        if parts.len() > 64 {
            return Err(RuntimeCatalogCompileError::Path {
                path: template.to_owned(),
            });
        }
        Ok(parts)
    })?;
    encode::map([(1, Value::Int(1)), (2, Value::List(parts))])
}

fn literal(value: &str) -> Result<Value, RuntimeCatalogCompileError> {
    encode::map([(1, Value::Int(0)), (2, Value::Text(value.to_owned()))])
}

fn placeholder(
    path: &str,
    services: &super::super::ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let kind = match path {
        "partition" => 2,
        "account" => 3,
        "region" => 4,
        _ => return encode::map([(1, Value::Int(1)), (2, selected_path(services, path)?)]),
    };
    encode::map([(1, Value::Int(kind))])
}
