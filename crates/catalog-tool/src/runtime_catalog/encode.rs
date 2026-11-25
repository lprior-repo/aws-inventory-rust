use super::{RuntimeCatalogCompileError, ServiceIndex};
use crate::kind_map::{KindMap, KindTarget};
use inventory_core::record::{FieldId, Key, Value};
use std::collections::BTreeMap;

pub struct RelationsResult {
    pub rows: Vec<Value>,
    pub inactive: BTreeMap<String, String>,
    pub reviewed: usize,
    pub declared: usize,
    pub declared_thin: usize,
}

impl RelationsResult {
    pub fn report(&self) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;
        let mut text = format!(
            "relations live={} pending={} declared={} reviewed={}\ndeclared thin={}\n",
            self.reviewed.saturating_add(self.declared),
            self.inactive.len(),
            self.declared,
            self.reviewed,
            self.declared_thin,
        );
        self.inactive
            .iter()
            .try_for_each(|(id, reason)| writeln!(text, "{id}\t{reason}"))?;
        Ok(text)
    }
}

pub(super) fn extension(
    plans: Vec<Value>,
    schemas: Vec<Value>,
    relations: Vec<Value>,
    coverage: Vec<Value>,
    decisions: Vec<Value>,
    sources: &super::CatalogSources,
    kinds: &KindMap,
) -> Result<Value, RuntimeCatalogCompileError> {
    let mut extension = map([
        (1, Value::Int(4)),
        (2, Value::List(plans)),
        (3, Value::List(schemas)),
        (4, Value::List(relations)),
        (5, Value::List(coverage)),
        (6, Value::List(decisions)),
    ])?;
    let encoded = inventory_core::record::Record::encode(&extension)?;
    let reviewed = inventory_core::catalog::runtime::compiled_inputs_sha256(encoded.root())
        .map_err(|_| RuntimeCatalogCompileError::Path {
            path: "extension sha256".to_owned(),
        })?;
    let receipt = super::provenance::encode(sources, kinds, reviewed)?;
    let Value::Map(fields) = &mut extension else {
        return Err(RuntimeCatalogCompileError::Path {
            path: "extension map".to_owned(),
        });
    };
    fields.insert(Key::Field(FieldId::new(7)?), receipt);
    Ok(extension)
}

pub(super) fn coverage(kinds: &KindMap) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    let mapped = kinds.entries().iter().map(|entry| match &entry.target {
        KindTarget::Mapped { kind, .. } => {
            coverage_row(Some(&entry.cfn_type), Some(kind), None, false)
        }
        KindTarget::Unmapped { reason } => {
            coverage_row(Some(&entry.cfn_type), None, Some(reason), false)
        }
    });
    let api_only = kinds
        .api_only()
        .iter()
        .map(|entry| coverage_row(None, Some(&entry.kind), None, true));
    mapped.chain(api_only).collect()
}

fn coverage_row(
    cfn_type: Option<&String>,
    kind: Option<&String>,
    reason: Option<&String>,
    api_only: bool,
) -> Result<Value, RuntimeCatalogCompileError> {
    let text =
        |value: Option<&String>| value.map_or(Value::Null, |value| Value::Text(value.clone()));
    map([
        (1, text(cfn_type)),
        (2, text(kind)),
        (3, Value::Bool(kind.is_some())),
        (4, text(reason)),
        (5, Value::Bool(api_only)),
    ])
}
pub(in crate::runtime_catalog) fn relations(
    kinds: &KindMap,
    services: &ServiceIndex<'_>,
    checked: &crate::overlay::CheckedOverlays,
) -> Result<RelationsResult, RuntimeCatalogCompileError> {
    super::relations::compile(kinds, services, checked)
}

pub(super) fn path(
    services: &ServiceIndex<'_>,
    path: &str,
) -> Result<Value, RuntimeCatalogCompileError> {
    let segments = path.split('.').collect::<Vec<_>>();
    if segments.is_empty() {
        return Err(RuntimeCatalogCompileError::Path {
            path: path.to_owned(),
        });
    }
    let ids = segments
        .iter()
        .map(|segment| services.field_id(segment))
        .collect::<Result<Vec<_>, _>>()?;
    if ids.len() > 8 {
        return Err(RuntimeCatalogCompileError::Path {
            path: path.to_owned(),
        });
    }
    Ok(Value::List(
        ids.into_iter()
            .map(|id| Value::Int(i64::from(id.value())))
            .collect(),
    ))
}
pub(super) fn map(
    entries: impl IntoIterator<Item = (u32, Value)>,
) -> Result<Value, RuntimeCatalogCompileError> {
    entries
        .into_iter()
        .map(|(field, value)| Ok((Key::Field(FieldId::new(field)?), value)))
        .collect::<Result<BTreeMap<_, _>, inventory_core::record::RecordError>>()
        .map(Value::Map)
        .map_err(RuntimeCatalogCompileError::Record)
}
