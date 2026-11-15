use super::{RuntimeCatalogCompileError, ServiceIndex};
use crate::kind_map::{KindMap, KindTarget};
use inventory_core::record::{FieldId, Key, Value};
use std::collections::BTreeMap;

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
        (1, Value::Int(2)),
        (2, Value::List(plans)),
        (3, Value::List(schemas)),
        (4, Value::List(relations)),
        (5, Value::List(coverage)),
        (6, Value::List(decisions)),
    ])?;
    let encoded = inventory_core::record::Record::encode(&extension)?;
    let reviewed = inventory_core::catalog::runtime::compiled_inputs_sha256(encoded.root())
        .map_err(|_| RuntimeCatalogCompileError::Capability)?;
    let receipt = super::provenance::encode(sources, kinds, reviewed)?;
    let Value::Map(fields) = &mut extension else {
        return Err(RuntimeCatalogCompileError::Capability);
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
pub(super) fn relations(
    kinds: &KindMap,
    services: &ServiceIndex<'_>,
    checked: &crate::overlay::CheckedOverlays,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    let context = super::model_path::ValidationContext { kinds, checked, services };
    kinds
        .relations()
        .iter()
        .filter(|relation| super::model_path::active_kind(&context, &relation.source_kind)
            && super::model_path::active_kind(&context, &relation.target_kind))
        .filter(|relation| kinds.relation_target_kinds(relation).is_none_or(|targets|
            targets.iter().all(|target| super::model_path::active_kind(&context, target))))
        .map(|relation| {
            let scope = match relation.scope.as_str() {
                "account_region" | "arn_account_region" => 0,
                "account" => 1,
                "home_region" => 2,
                _ => return Err(RuntimeCatalogCompileError::Capability),
            };
            let mut value = map([
                (1, Value::Text(relation.source_kind.clone())),
                (2, path(services, &relation.source_path)?),
                (3, Value::Text(relation.target_kind.clone())),
                (4, path(services, &relation.target_path)?),
                (5, Value::Text(relation.relation.clone())),
                (6, Value::Int(scope)),
                (10, Value::Int(match relation.source {
                    crate::kind_map::RelationSource::SelectedOutput => 0,
                    crate::kind_map::RelationSource::Request => 1,
                })),
            ])?;
            if relation.scope == "arn_account_region" {
                if relation.source_kind != "AWS::Oam::Link" || relation.source_path != "SinkArn"
                    || relation.target_kind != "AWS::Oam::Sink" || relation.target_path != "Arn"
                    || relation.relation != "TARGETS_SINK" {
                    return Err(RuntimeCatalogCompileError::Capability);
                }
                let owner = map([
                    (1, Value::Int(1)), (2, Value::Text("aws".into())),
                    (3, Value::Text("oam".into())), (4, Value::Text("sink/".into())),
                    (5, Value::Int(0)),
                ])?;
                let Value::Map(fields) = &mut value else {
                    return Err(RuntimeCatalogCompileError::Capability);
                };
                fields.insert(Key::Field(FieldId::new(8)?), owner);
            }
            if let Some(targets) = kinds.relation_target_kinds(relation) {
                let Value::Map(fields) = &mut value else {
                    return Err(RuntimeCatalogCompileError::Capability);
                };
                fields.insert(Key::Field(FieldId::new(9)?), Value::List(
                    targets.iter().cloned().map(Value::Text).collect(),
                ));
            }
            Ok(value)
        })
        .collect()
}

pub(super) fn path(
    services: &ServiceIndex<'_>,
    path: &str,
) -> Result<Value, RuntimeCatalogCompileError> {
    let segments = path.split('.').collect::<Vec<_>>();
    let ids = segments
        .iter()
        .map(|segment| services.field_id(segment))
        .collect::<Result<Vec<_>, _>>()?;
    if ids.is_empty() || ids.len() > 8 {
        return Err(RuntimeCatalogCompileError::Capability);
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
