use super::{RelationsResult, RuntimeCatalogCompileError, ServiceIndex, map, path};
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
    let context = super::super::model_path::ValidationContext {
        kinds,
        checked,
        services,
    };
    let active = |source: &str, targets: &crate::kind_map::ReviewedTargetKinds| {
        super::super::model_path::active_kind(&context, source)
            && targets
                .iter()
                .all(|kind| super::super::model_path::active_kind(&context, kind))
    };
    let mut rows = Vec::new();
    let mut inactive = BTreeMap::new();
    for row in kinds.relations().iter() {
        if !active(&row.source_kind, &row.target_kinds) {
            inactive.insert(
                format!("{} -> {} (inactive source kind)", row.source_kind, row.target_kinds.primary()),
                format!("source kind {} is inactive", row.source_kind),
            );
            continue;
        }
        rows.push(encode(row, services)?);
    }
    for row in kinds
        .parent_relations()
        .iter()
    {
        if !active(&row.source_kind, &row.target_kinds) {
            inactive.insert(
                format!("{} -> {} (inactive source kind, parent)", row.source_kind, row.target_kinds.primary()),
                format!("source kind {} is inactive", row.source_kind),
            );
            continue;
        }
        rows.push(map([
            (1, Value::Text(row.source_kind.clone())),
            (2, Value::List(Vec::new())),
            (3, Value::Text(row.target_kinds.primary().to_owned())),
            (4, Value::List(Vec::new())),
            (5, Value::Text(row.relation.clone())),
            (6, Value::Int(scope(&row.scope)?)),
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
    for row in kinds
        .compound_relations()
        .iter()
    {
        if !active(&row.source_kind, &row.target_kinds) {
            inactive.insert(
                format!("{} -> {} (inactive source kind, compound)", row.source_kind, row.target_kinds.primary()),
                format!("source kind {} is inactive", row.source_kind),
            );
            continue;
        }
        let (first, rest) = row
            .components
            .split_first()
            .ok_or(RuntimeCatalogCompileError::Capability)?;
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
            return Err(RuntimeCatalogCompileError::Capability);
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
        rows.push(Value::Map(fields));
    }
    Ok(RelationsResult { rows, inactive })
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
        (6, Value::Int(scope(&row.scope)?)),
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
        let (origin, guard_path) = source(&guard.source);
        fields.push((
            13,
            map([
                (1, Value::Int(origin)),
                (2, Value::Int(path_mode(guard_path))),
                (3, value_path(services, guard_path)?),
                (4, Value::Text(guard.equals.clone())),
            ])?,
        ));
    }
    if row.scope == "arn_account_region" {
        if row.source_kind != "AWS::Oam::Link"
            || row.target_kinds.primary() != "AWS::Oam::Sink"
            || row.relation != "TARGETS_SINK"
            || !matches!(&row.source, ReviewedRelationSource::SelectedOutput(ReviewedValuePath::Fields(path)) if path == "SinkArn")
            || !matches!(&row.target, ReviewedRelationTarget::SelectedOutput(ReviewedValuePath::Fields(path)) if path == "Arn")
        {
            return Err(RuntimeCatalogCompileError::Capability);
        }
        fields.push((
            8,
            map([
                (1, Value::Int(1)),
                (2, Value::Text("aws".into())),
                (3, Value::Text("oam".into())),
                (4, Value::Text("sink/".into())),
                (5, Value::Int(0)),
            ])?,
        ));
    }
    map(fields)
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

pub(super) fn scope(scope: &str) -> Result<i64, RuntimeCatalogCompileError> {
    match scope {
        "account_region" | "arn_account_region" => Ok(0),
        "account" => Ok(1),
        "home_region" => Ok(2),
        "arn_region" => Ok(3),
        _ => Err(RuntimeCatalogCompileError::Capability),
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_returns_relations_result_type() {
        // Test that compile returns RelationsResult type
        fn _check_type(
            result: Result<RelationsResult, RuntimeCatalogCompileError>,
        ) -> bool {
            result.is_ok()
        }
        // Can't actually call compile without setup, but we can verify the type signature
        let _: fn(
            &KindMap,
            &ServiceIndex<'_>,
            &crate::overlay::CheckedOverlays,
        ) -> Result<RelationsResult, RuntimeCatalogCompileError> = compile;
    }
}
