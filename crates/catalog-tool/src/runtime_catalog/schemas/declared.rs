use super::{RuntimeCatalogCompileError, Value};
use inventory_core::record::{FieldId, Key};
use std::collections::BTreeMap;

type Paths<'a> = BTreeMap<&'a str, Vec<&'a Value>>;

pub(super) fn paths(rows: &[Value]) -> Result<Paths<'_>, RuntimeCatalogCompileError> {
    let mut paths = BTreeMap::new();
    rows.iter().try_for_each(|row| {
        if field(row, 7)? != Some(&Value::Int(0)) {
            return Ok(());
        }
        let Some(Value::Text(kind)) = field(row, 3)? else {
            return Err(RuntimeCatalogCompileError::Capability);
        };
        append(&mut paths, kind, required(row, 4)?)?;
        if let Some(value) = field(row, 14)? {
            let Value::List(components) = value else {
                return Err(RuntimeCatalogCompileError::Capability);
            };
            components
                .iter()
                .try_for_each(|component| append(&mut paths, kind, required(component, 4)?))?;
        }
        Ok(())
    })?;
    Ok(paths)
}

fn append<'a>(
    paths: &mut Paths<'a>,
    kind: &'a str,
    path: &'a Value,
) -> Result<(), RuntimeCatalogCompileError> {
    let Value::List(fields) = path else {
        return Err(RuntimeCatalogCompileError::Capability);
    };
    if fields.is_empty() {
        return Ok(());
    }
    let columns = paths.entry(kind).or_default();
    if !columns.contains(&path) {
        columns.push(path);
    }
    Ok(())
}

fn required(value: &Value, id: u32) -> Result<&Value, RuntimeCatalogCompileError> {
    field(value, id)?.ok_or(RuntimeCatalogCompileError::Capability)
}

fn field(value: &Value, id: u32) -> Result<Option<&Value>, RuntimeCatalogCompileError> {
    let Value::Map(fields) = value else {
        return Err(RuntimeCatalogCompileError::Capability);
    };
    Ok(fields.get(&Key::Field(FieldId::new(id)?)))
}
