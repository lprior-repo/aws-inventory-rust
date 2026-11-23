mod namespace;

use super::super::{RuntimeCatalogCompileError, ServiceIndex, encode};
use crate::{
    kind_map::KindMap,
    overlay::{MAX_RESOURCE_EMISSIONS, OperationOverlay, Paginator, ResourceEmission},
};
use inventory_core::{
    catalog::{Operation, Service},
    record::Value,
};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
pub(super) struct Callable<'a> {
    pub service_name: &'a str,
    pub operation_name: &'a str,
    pub primary_kind: &'a str,
    pub service: Service<'a>,
    pub operation: Operation<'a>,
}

/// Additional resource metadata uses the same numeric field meanings as the primary.
/// Both are normalized into one nonempty ResourceEmissions collection by the decoder.
pub(super) fn compile(
    callable: Callable<'_>,
    overlay: &OperationOverlay,
    kinds: &KindMap,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    let Callable {
        service_name,
        operation_name,
        primary_kind,
        service,
        operation,
    } = callable;
    validate_emission_count(overlay)?;
    let primary_kinds = compute_primary_kinds(primary_kind, overlay);
    let ctx = EmissionCompileCtx {
        service_name,
        operation_name,
        kinds,
        service,
        operation,
        services,
    };
    let values = compile_emission_list(&ctx, overlay, primary_kinds)?;
    validate_reviewed_count(kinds, service_name, operation_name, overlay)?;
    Ok(Value::List(values))
}

fn validate_emission_count(overlay: &OperationOverlay) -> Result<(), RuntimeCatalogCompileError> {
    if overlay.additional_resources.len() >= MAX_RESOURCE_EMISSIONS {
        return Err(RuntimeCatalogCompileError::Path {
            path: "additional resources count".to_owned(),
        });
    }
    Ok(())
}

fn compute_primary_kinds<'a>(
    primary_kind: &'a str,
    overlay: &'a OperationOverlay,
) -> BTreeSet<&'a str> {
    std::iter::once(primary_kind)
        .chain(
            overlay
                .classification
                .as_ref()
                .into_iter()
                .flat_map(|c| c.variants.values().map(String::as_str)),
        )
        .collect()
}

struct EmissionCompileCtx<'a> {
    service_name: &'a str,
    operation_name: &'a str,
    kinds: &'a KindMap,
    service: Service<'a>,
    operation: Operation<'a>,
    services: &'a ServiceIndex<'a>,
}

fn compile_emission_list<'a>(
    ctx: &'a EmissionCompileCtx<'a>,
    overlay: &'a OperationOverlay,
    primary_kinds: BTreeSet<&'a str>,
) -> Result<Vec<Value>, RuntimeCatalogCompileError> {
    let mut seen_kinds = primary_kinds;
    let mut keys = BTreeSet::new();
    let mut values = Vec::new();
    for emission in &overlay.additional_resources {
        compile_single_emission(ctx, emission, &mut seen_kinds, &mut keys, &mut values)?;
    }
    Ok(values)
}

fn compile_single_emission<'a>(
    ctx: &'a EmissionCompileCtx<'a>,
    emission: &'a ResourceEmission,
    seen_kinds: &mut BTreeSet<&'a str>,
    keys: &mut BTreeSet<&'a str>,
    values: &mut Vec<Value>,
) -> Result<(), RuntimeCatalogCompileError> {
    validate_emission_key(emission, keys)?;
    let kind = get_emission_kind(ctx, emission)?;
    let emitted = compute_emitted_kinds(emission, kind);
    check_duplicate_kinds(&emitted, seen_kinds)?;
    seen_kinds.extend(emitted);
    values.push(descriptor(
        ctx.service_name,
        ctx.operation_name,
        kind,
        ctx.service,
        ctx.operation,
        emission,
        ctx.services,
    )?);
    Ok(())
}

fn validate_emission_key<'a>(
    emission: &'a ResourceEmission,
    keys: &mut BTreeSet<&'a str>,
) -> Result<(), RuntimeCatalogCompileError> {
    if emission.key == "primary"
        || emission.key.is_empty()
        || emission.key.len() > 64
        || !emission
            .key
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        || !keys.insert(emission.key.as_str())
    {
        return Err(RuntimeCatalogCompileError::Path {
            path: format!("emission key {}", emission.key),
        });
    }
    Ok(())
}

fn get_emission_kind<'a>(
    ctx: &'a EmissionCompileCtx<'a>,
    emission: &ResourceEmission,
) -> Result<&'a str, RuntimeCatalogCompileError> {
    ctx.kinds
        .additional_emission_kind(ctx.service_name, ctx.operation_name, &emission.key)
        .ok_or_else(|| RuntimeCatalogCompileError::Path {
            path: format!("emission kind {}", emission.key),
        })
}

fn compute_emitted_kinds<'a>(emission: &'a ResourceEmission, kind: &'a str) -> BTreeSet<&'a str> {
    std::iter::once(kind)
        .chain(
            emission
                .classification
                .as_ref()
                .into_iter()
                .flat_map(|c| c.variants.values().map(String::as_str)),
        )
        .collect()
}

fn check_duplicate_kinds(
    emitted: &BTreeSet<&str>,
    seen_kinds: &BTreeSet<&str>,
) -> Result<(), RuntimeCatalogCompileError> {
    for kind in emitted {
        if seen_kinds.contains(kind) {
            return Err(RuntimeCatalogCompileError::Path {
                path: format!("duplicate emission kind {kind}"),
            });
        }
    }
    Ok(())
}

fn validate_reviewed_count(
    kinds: &KindMap,
    service_name: &str,
    operation_name: &str,
    overlay: &OperationOverlay,
) -> Result<(), RuntimeCatalogCompileError> {
    let reviewed = kinds
        .additional_emissions()
        .iter()
        .filter(|row| row.service == service_name && row.operation == operation_name)
        .count();
    if reviewed != overlay.additional_resources.len() {
        return Err(RuntimeCatalogCompileError::Path {
            path: "reviewed additional resources count".to_owned(),
        });
    }
    Ok(())
}

fn descriptor(
    service_name: &str,
    operation_name: &str,
    kind: &str,
    service: Service<'_>,
    operation: Operation<'_>,
    emission: &ResourceEmission,
    services: &ServiceIndex<'_>,
) -> Result<Value, RuntimeCatalogCompileError> {
    namespace::validate(&emission.identity, operation_name)?;
    let overlay = resource_overlay(emission);
    let item =
        super::super::selection::target(service, operation.output()?, overlay.items.as_ref())?;
    super::super::plan_validation::validate_resource(service, operation, &overlay)?;
    let fields = [
        (1, Value::Text(emission.key.clone())),
        (3, Value::Text(kind.to_owned())),
        (5, super::item_value(overlay.items.as_ref(), services)?),
        (
            6,
            super::identity_value(overlay.identity.as_ref(), service_name, services)?,
        ),
        (9, super::exclusions(&overlay, services)?),
    ];
    let tags = super::tags::compile(service, operation, &overlay)?;
    let capabilities = super::capabilities::compile(service, operation, &overlay)?;
    let map_exclusions = if overlay.exclude_map_keys.is_empty() {
        None
    } else {
        Some((21, super::map_exclusions::encode(service, item, &overlay)?))
    };
    encode::map(
        fields
            .into_iter()
            .chain(tags.map(|tags| (14, tags)))
            .chain(capabilities)
            .chain(map_exclusions),
    )
}

/// A cold compiler view reuses the existing modeled validators and field codecs.
/// It never contains bindings, branches, pagination or a second API operation.
pub(in crate::runtime_catalog) fn resource_overlay(
    emission: &ResourceEmission,
) -> OperationOverlay {
    OperationOverlay {
        items: Some(emission.items.clone()),
        identity: Some(emission.identity.clone()),
        projection: emission.projection.clone(),
        classification: emission.classification.clone(),
        exclude_from_hash: emission.exclude_from_hash.clone(),
        exclude_map_keys: emission.exclude_map_keys.clone(),
        queryable: emission.queryable.clone(),
        tags: emission.tags.clone(),
        paginator: Some(Paginator::None),
        ..OperationOverlay::default()
    }
}
