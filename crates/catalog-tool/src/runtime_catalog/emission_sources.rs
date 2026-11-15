use crate::{kind_map::KindMap, overlay::{CallableOperation, CheckedOverlays, Emission, ItemSelector, KindClassification, MemberPath, OperationOverlay, ResourceProjection}};

/// A borrowed compiler view; discovery bindings and pagination stay on the operation.
pub(super) struct ResourceSource<'a> {
    pub service: &'a str,
    pub key: &'a str,
    pub operation: &'a str,
    pub kind: &'a str,
    pub items: Option<&'a ItemSelector>,
    pub projection: Option<&'a ResourceProjection>,
    pub classification: Option<&'a KindClassification>,
    pub queryable: &'a [MemberPath],
}

impl ResourceSource<'_> {
    pub fn emits_kind(&self, kind: &str) -> bool {
        self.kind == kind || self.classification.is_some_and(|classification|
            classification.variants.values().any(|variant| variant == kind))
    }
}

pub(super) fn sources<'a>(kinds: &'a KindMap, checked: &'a CheckedOverlays)
    -> impl Iterator<Item = ResourceSource<'a>> + 'a {
    let primary = kinds.operations().iter().filter_map(|row| {
        let overlay = callable(checked, &row.service, &row.operation)?.plan();
        Some(ResourceSource { key: "primary", service: &row.service, operation: &row.operation, kind: &row.kind,
            items: overlay.items.as_ref(), projection: overlay.projection.as_ref(),
            classification: overlay.classification.as_ref(), queryable: &overlay.queryable })
    });
    let additional = kinds.additional_emissions().iter().filter_map(|row| {
        let overlay = callable(checked, &row.service, &row.operation)?.plan();
        let emission = overlay.additional_resources.iter().find(|emission| emission.key == row.emitter_key)?;
        Some(ResourceSource { key: &row.emitter_key, service: &row.service, operation: &row.operation, kind: &row.kind,
            items: Some(&emission.items), projection: emission.projection.as_ref(),
            classification: emission.classification.as_ref(), queryable: &emission.queryable })
    });
    primary.chain(additional)
}

fn callable<'a>(checked: &'a CheckedOverlays, service: &str, operation: &str) -> Option<&'a CallableOperation> {
    checked.services().iter().find(|review| review.service() == service)
        .and_then(|review| review.callable().iter().find(|call| call.operation() == operation
            && emits(call.plan()) && call.plan().tag_endpoint.is_none() && !call.plan().endpoint_discovery))
}

fn emits(overlay: &OperationOverlay) -> bool {
    overlay.emission == Emission::EmitMetadata || overlay.branches.iter()
        .any(|branch| branch.emission.unwrap_or(overlay.emission) == Emission::EmitMetadata)
}
