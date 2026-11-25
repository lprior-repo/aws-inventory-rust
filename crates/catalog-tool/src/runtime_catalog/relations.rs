use super::encode::{RelationsResult, map, path};
use super::{RuntimeCatalogCompileError, ServiceIndex};
mod declared;
use crate::kind_map::{
    KindMap, ReviewedRelation, ReviewedRelationSource, ReviewedRelationTarget, ReviewedValuePath,
};
use inventory_core::record::Value;
use std::collections::BTreeMap;

pub(super) fn compile(
    kinds: &KindMap,
    services: &ServiceIndex<'_>,
    checked: &crate::overlay::CheckedOverlays,
) -> Result<RelationsResult, RuntimeCatalogCompileError> {
    let context = super::model_path::ValidationContext {
        kinds,
        checked,
        services,
    };
    let active = |source: &str, targets: &crate::kind_map::ReviewedTargetKinds| {
        if !super::model_path::active_kind(&context, source) {
            return Some(format!("source kind {source} is inactive"));
        }
        targets
            .iter()
            .find(|kind| !super::model_path::active_kind(&context, kind))
            .map(|kind| format!("target kind {kind} is inactive"))
    };
    let mut rows = Vec::new();
    let mut inactive = BTreeMap::new();
    compile_relations(kinds, services, &active, &mut rows, &mut inactive)?;
    compile_parent_relations(kinds, &active, &mut rows, &mut inactive)?;
    compile_compound_relations(kinds, services, &active, &mut rows, &mut inactive)?;
    let reviewed = rows.len();
    let mut result = RelationsResult {
        rows,
        inactive,
        reviewed,
        declared: 0,
        declared_thin: 0,
    };
    declared::append(&context, &mut result)?;
    Ok(result)
}

fn compile_relations(
    kinds: &KindMap,
    services: &ServiceIndex<'_>,
    active: &dyn Fn(&str, &crate::kind_map::ReviewedTargetKinds) -> Option<String>,
    rows: &mut Vec<Value>,
    inactive: &mut BTreeMap<String, String>,
) -> Result<(), RuntimeCatalogCompileError> {
    for row in kinds.relations().iter() {
        if let Some(reason) = active(&row.source_kind, &row.target_kinds) {
            inactive.insert(
                format!(
                    "{} -> {} (inactive source kind)",
                    row.source_kind,
                    row.target_kinds.primary()
                ),
                reason,
            );
            continue;
        }
        rows.push(encode(row, services)?);
    }
    Ok(())
}

fn compile_parent_relations(
    kinds: &KindMap,
    active: &dyn Fn(&str, &crate::kind_map::ReviewedTargetKinds) -> Option<String>,
    rows: &mut Vec<Value>,
    inactive: &mut BTreeMap<String, String>,
) -> Result<(), RuntimeCatalogCompileError> {
    for row in kinds.parent_relations().iter() {
        if let Some(reason) = active(&row.source_kind, &row.target_kinds) {
            inactive.insert(
                format!(
                    "{} -> {} (inactive source kind, parent)",
                    row.source_kind,
                    row.target_kinds.primary()
                ),
                reason,
            );
            continue;
        }
        rows.push(map([
            (1, Value::Text(row.source_kind.clone())),
            (2, Value::List(Vec::new())),
            (3, Value::Text(row.target_kinds.primary().to_owned())),
            (4, Value::List(Vec::new())),
            (5, Value::Text(row.relation.clone())),
            (6, Value::Int(scope(&row.source_kind, &row.scope)?)),
            (7, Value::Int(1)),
            (
                9,
                Value::List(
                    row.target_kinds
                        .iter()
                        .map(|kind| Value::Text(kind.to_owned()))
                        .collect(),
                ),
            ),
            (10, Value::Int(2)),
            (11, Value::Int(1)),
            (12, Value::Int(4)),
        ])?);
    }
    Ok(())
}

fn compile_compound_relations(
    kinds: &KindMap,
    services: &ServiceIndex<'_>,
    active: &dyn Fn(&str, &crate::kind_map::ReviewedTargetKinds) -> Option<String>,
    rows: &mut Vec<Value>,
    inactive: &mut BTreeMap<String, String>,
) -> Result<(), RuntimeCatalogCompileError> {
    for row in kinds.compound_relations().iter() {
        if let Some(reason) = active(&row.source_kind, &row.target_kinds) {
            inactive.insert(
                format!(
                    "{} -> {} (inactive source kind, compound)",
                    row.source_kind,
                    row.target_kinds.primary()
                ),
                reason,
            );
            continue;
        }
        rows.push(encode_compound(row, services)?);
    }
    Ok(())
}

fn encode_compound(
    row: &crate::kind_map::ReviewedCompoundRelation,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let (first, rest) =
        row.components
            .split_first()
            .ok_or_else(|| RuntimeCatalogCompileError::KindContext {
                kind: row.source_kind.clone(),
                path: "components".to_owned(),
                reason: "compound relation has no components",
            })?;
    let base = ReviewedRelation {
        source_kind: row.source_kind.clone(),
        source: first.source.clone(),
        target_kinds: row.target_kinds.clone(),
        target: first.target.clone(),
        relation: row.relation.clone(),
        scope: row.scope.clone(),
        guard: None,
    };
    let Value::Map(mut fields) = encode(&base, services)? else {
        return Err(RuntimeCatalogCompileError::KindContext {
            kind: row.source_kind.clone(),
            path: "encode".to_owned(),
            reason: "relation encode did not produce a map",
        });
    };
    let components = rest
        .iter()
        .map(|component| {
            let (origin, source_path) = source(&component.source);
            let (mode, target_path) = target(&component.target);
            map([
                (1, Value::Int(origin)),
                (2, value_path(services, source_path)?),
                (3, Value::Int(path_mode(source_path))),
                (4, optional_path(services, target_path)?),
                (5, Value::Int(mode)),
            ])
        })
        .collect::<Result<Vec<_>, RuntimeCatalogCompileError>>()?;
    fields.insert(
        inventory_core::record::Key::Field(inventory_core::record::FieldId::new(14)?),
        Value::List(components),
    );
    Ok(Value::Map(fields))
}

fn encode(
    row: &ReviewedRelation,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let (origin, source_path) = source(&row.source);
    let (target_mode, target_path) = target(&row.target);
    let mut fields = vec![
        (1, Value::Text(row.source_kind.clone())),
        (2, value_path(services, source_path)?),
        (3, Value::Text(row.target_kinds.primary().to_owned())),
        (4, optional_path(services, target_path)?),
        (5, Value::Text(row.relation.clone())),
        (6, Value::Int(scope(&row.source_kind, &row.scope)?)),
        (7, Value::Int(1)),
        (
            9,
            Value::List(
                row.target_kinds
                    .iter()
                    .map(|kind| Value::Text(kind.to_owned()))
                    .collect(),
            ),
        ),
        (10, Value::Int(origin)),
        (11, Value::Int(path_mode(source_path))),
        (12, Value::Int(target_mode)),
    ];
    if let Some(guard) = &row.guard {
        fields.push(encode_guard(guard, services)?);
    }
    if let Some(arn_field) = encode_arn_scope(row)? {
        fields.push(arn_field);
    }
    map(fields)
}

fn encode_guard(
    guard: &crate::kind_map::ReviewedRelationGuard,
    services: &ServiceIndex<'_>,
) -> Result<(u32, Value), RuntimeCatalogCompileError> {
    let (origin, guard_path) = source(&guard.source);
    Ok((
        13,
        map([
            (1, Value::Int(origin)),
            (2, Value::Int(path_mode(guard_path))),
            (3, value_path(services, guard_path)?),
            (4, Value::Text(guard.equals.clone())),
        ])?,
    ))
}

fn encode_arn_scope(
    row: &ReviewedRelation,
) -> Result<Option<(u32, Value)>, RuntimeCatalogCompileError> {
    if row.scope != "arn_account_region" {
        return Ok(None);
    }
    if row.source_kind != "AWS::Oam::Link"
        || row.target_kinds.primary() != "AWS::Oam::Sink"
        || row.relation != "TARGETS_SINK"
        || !matches!(&row.source, ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields(path)) if path == "SinkArn")
        || !matches!(&row.target, ReviewedRelationTarget::SelectedOutput(ReviewedValuePath::Fields(path)) if path == "Arn")
    {
        return Err(RuntimeCatalogCompileError::KindContext {
            kind: row.source_kind.clone(),
            path: "arn_account_region".to_owned(),
            reason: "invalid arn_account_region scope usage",
        });
    }
    Ok(Some((
        8,
        map([
            (1, Value::Int(1)),
            (2, Value::Text("aws".into())),
            (3, Value::Text("oam".into())),
            (4, Value::Text("sink/".into())),
            (5, Value::Int(0)),
        ])?,
    )))
}

pub(super) fn source(source: &ReviewedRelationSource) -> (i64, &ReviewedValuePath) {
    match source {
        ReviewedRelationSource::SelectedOutput(path) => (0, path),
        ReviewedRelationSource::Request(path) => (1, path),
    }
}

pub(super) fn target(target: &ReviewedRelationTarget) -> (i64, Option<&ReviewedValuePath>) {
    match target {
        ReviewedRelationTarget::SelectedOutput(path) => (path_mode(path), Some(path)),
        ReviewedRelationTarget::BoundContext(path) => (
            match path {
                ReviewedValuePath::Fields(_) => 2,
                ReviewedValuePath::RootText => 3,
            },
            Some(path),
        ),
        ReviewedRelationTarget::CanonicalId => (4, None),
    }
}

pub(super) fn path_mode(path: &ReviewedValuePath) -> i64 {
    match path {
        ReviewedValuePath::Fields(_) => 0,
        ReviewedValuePath::RootText => 1,
    }
}

fn optional_path(
    services: &ServiceIndex<'_>,
    value: Option<&ReviewedValuePath>,
) -> Result<Value, RuntimeCatalogCompileError> {
    value.map_or_else(
        || Ok(Value::List(Vec::new())),
        |value| value_path(services, value),
    )
}

pub(super) fn value_path(
    services: &ServiceIndex<'_>,
    value: &ReviewedValuePath,
) -> Result<Value, RuntimeCatalogCompileError> {
    match value {
        ReviewedValuePath::Fields(value) => path(services, value),
        ReviewedValuePath::RootText => Ok(Value::List(Vec::new())),
    }
}

pub(super) fn scope(kind: &str, scope: &str) -> Result<i64, RuntimeCatalogCompileError> {
    match scope {
        "account_region" | "arn_account_region" => Ok(0),
        "account" => Ok(1),
        "home_region" => Ok(2),
        "arn_region" => Ok(3),
        _ => Err(RuntimeCatalogCompileError::KindContext {
            kind: kind.to_owned(),
            path: scope.to_owned(),
            reason: "unsupported scope",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_scope_returns_kind_context_with_scope_as_path() {
        let result = scope("AWS::EC2::VPC", "same_vpc");
        assert!(matches!(result,
            Err(RuntimeCatalogCompileError::KindContext { kind, path, reason })
                if kind == "AWS::EC2::VPC" && path == "same_vpc" && reason == "unsupported scope"));
    }

    #[test]
    fn supported_scopes_encode_successfully() -> Result<(), RuntimeCatalogCompileError> {
        assert_eq!(scope("AWS::EC2::VPC", "account_region")?, 0);
        assert_eq!(scope("AWS::EC2::VPC", "account")?, 1);
        assert_eq!(scope("AWS::EC2::VPC", "home_region")?, 2);
        assert_eq!(scope("AWS::EC2::VPC", "arn_region")?, 3);
        Ok(())
    }
}
