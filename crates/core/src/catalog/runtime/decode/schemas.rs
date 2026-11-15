use super::{RuntimeCatalogError, list_value, map, metadata};
use crate::inventory::{Column, Kind, KindSchema, ScalarType, SqlName};
use crate::record::{ListRef, ValueRef};
use std::{collections::BTreeSet, sync::Arc};

pub(super) struct SchemaData {
    pub(super) schemas: Vec<Arc<KindSchema>>,
    pub(super) name_paths: Vec<super::super::KindNamePath>,
}

pub(super) fn read(list: ListRef<'_>) -> Result<SchemaData, RuntimeCatalogError> {
    let entries = metadata::bounded(list, schema, metadata::MAX_ITEMS)?;
    let kinds = entries
        .iter()
        .map(|(schema, _)| schema.kind().as_str())
        .collect::<BTreeSet<_>>();
    if kinds.len() != entries.len() {
        return Err(RuntimeCatalogError::Capability);
    }
    let (schemas, name_paths) = entries.into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut schemas, mut names), (schema, name_path)| {
            if let Some(path) = name_path {
                names.push(super::super::KindNamePath {
                    kind: schema.kind().clone(),
                    path,
                });
            }
            schemas.push(Arc::new(schema));
            (schemas, names)
        },
    );
    Ok(SchemaData {
        schemas,
        name_paths,
    })
}

type SchemaEntry = (KindSchema, Option<Box<[crate::record::FieldId]>>);

fn schema(value: ValueRef<'_>) -> Result<SchemaEntry, RuntimeCatalogError> {
    let fields = map(value)?;
    let kind = Kind::parse(&metadata::text(fields, 1)?)?;
    let table = SqlName::parse(&metadata::text(fields, 2)?)?;
    let columns = metadata::bounded(metadata::list(fields, 3)?, column, metadata::MAX_COLUMNS)?;
    let name_path = optional_path(fields)?;
    if name_path
        .as_ref()
        .is_some_and(|path| !columns.iter().any(|column| column.path() == path.as_ref()))
    {
        return Err(RuntimeCatalogError::Capability);
    }
    metadata::only_fields(fields, &[1, 2, 3, 4])?;
    Ok((KindSchema::new(kind, table, columns)?, name_path))
}

fn optional_path(
    fields: crate::record::MapRef<'_>,
) -> Result<Option<Box<[crate::record::FieldId]>>, RuntimeCatalogError> {
    match fields.get_field(crate::record::FieldId::new(4)?)? {
        Some(ValueRef::List(path)) => Ok(Some(metadata::id_path(path)?)),
        Some(ValueRef::Null) | None => Ok(None),
        _ => Err(RuntimeCatalogError::Capability),
    }
}

fn column(value: ValueRef<'_>) -> Result<Column, RuntimeCatalogError> {
    let fields = map(value)?;
    let path = metadata::selected_path(list_value(metadata::field(fields, 1)?)?)?;
    let scalar = scalar(metadata::integer(fields, 2)?)?;
    metadata::only_fields(fields, &[1, 2])?;
    if path.is_empty() {
        return Ok(Column::selected_scalar(scalar));
    }
    Ok(Column::new(path.into_vec(), scalar)?)
}

fn scalar(value: i64) -> Result<ScalarType, RuntimeCatalogError> {
    match value {
        0 => Ok(ScalarType::Bool),
        1 => Ok(ScalarType::Integer),
        2 => Ok(ScalarType::Real),
        3 => Ok(ScalarType::Text),
        4 => Ok(ScalarType::Blob),
        5 => Ok(ScalarType::Timestamp),
        _ => Err(RuntimeCatalogError::Capability),
    }
}
