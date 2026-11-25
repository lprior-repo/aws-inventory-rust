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
    if overlay.additional_resources.len() >= MAX_RESOURCE_EMISSIONS {
        return Err(RuntimeCatalogCompileError::Path {
            path: "additional resources count".to_owned(),
        });
    }
    let primary_kinds = std::iter::once(primary_kind)
        .chain(
            overlay
                .classification
                .as_ref()
                .into_iter()
                .flat_map(|classification| classification.variants.values().map(String::as_str)),
        )
        .collect::<BTreeSet<_>>();
    let (_, _, values) =
        overlay.additional_resources.iter().try_fold(
            (primary_kinds, BTreeSet::new(), Vec::new()),
            |(mut seen_kinds, mut keys, mut values), emission| {
                if emission.key == "primary"
                    || emission.key.is_empty()
                    || emission.key.len() > 64
                    || !emission.key.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                    })
                    || !keys.insert(emission.key.as_str())
                {
                    return Err(RuntimeCatalogCompileError::Path {
                        path: format!("emission key {}", emission.key),
                    });
                }
                let kind = kinds
                    .additional_emission_kind(service_name, operation_name, &emission.key)
                    .ok_or_else(|| RuntimeCatalogCompileError::Path {
                        path: format!("emission kind {}", emission.key),
                    })?;
                let emitted = std::iter::once(kind)
                    .chain(emission.classification.as_ref().into_iter().flat_map(
                        |classification| classification.variants.values().map(String::as_str),
                    ))
                    .collect::<BTreeSet<_>>();
                emitted.into_iter().try_for_each(|kind| {
                    if seen_kinds.insert(kind) {
                        Ok(())
                    } else {
                        Err(RuntimeCatalogCompileError::Path {
                            path: format!("duplicate emission kind {kind}"),
                        })
                    }
                })?;
                values.push(descriptor(
                    service_name,
                    operation_name,
                    kind,
                    service,
                    operation,
                    emission,
                    services,
                )?);
                Ok((seen_kinds, keys, values))
            },
        )?;
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
    Ok(Value::List(values))
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
