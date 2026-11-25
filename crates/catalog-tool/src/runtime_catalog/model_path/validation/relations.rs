use crate::kind_map::{
    ReviewedRelationSource, ReviewedRelationTarget, ReviewedTargetKinds, ReviewedValuePath,
};
use crate::runtime_catalog::RuntimeCatalogCompileError;

use super::{ValidationContext, validate_kind_origin_path};
use crate::runtime_catalog::model_path::emission::{Origin, Terminal};

pub(super) fn validate_relations(
    context: &ValidationContext<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    let active = |source: &str, targets: &ReviewedTargetKinds| {
        crate::runtime_catalog::model_path::active_kind(context, source)
            && targets.iter().all(|kind| {
                crate::runtime_catalog::model_path::active_kind(context, kind)
            })
    };
    for relation in context.kinds.relations().iter() {
        if !active(&relation.source_kind, &relation.target_kinds) {
            continue;
        }
        validate_simple_relation_source(
            context,
            &relation.source_kind,
            &relation.source,
        )?;
        if let Some(guard) = &relation.guard {
            validate_simple_relation_guard(
                context,
                &relation.source_kind,
                &guard.source,
            )?;
        }
        validate_simple_relation_target(
            context,
            &relation.source_kind,
            &relation.target_kinds,
            &relation.target,
        )?;
    }
    validate_compound_relations(context)
}

fn validate_compound_relations(
    context: &ValidationContext<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    let active = |source: &str, targets: &ReviewedTargetKinds| {
        crate::runtime_catalog::model_path::active_kind(context, source)
            && targets.iter().all(|kind| {
                crate::runtime_catalog::model_path::active_kind(context, kind)
            })
    };
    for relation in context.kinds.compound_relations().iter() {
        if !active(&relation.source_kind, &relation.target_kinds) {
            continue;
        }
        for component in &relation.components {
            validate_compound_component_source(
                context,
                &relation.source_kind,
                &component.source,
            )?;
            for kind in relation.target_kinds.iter() {
                validate_compound_component_target(
                    context,
                    kind,
                    &component.target,
                )?;
            }
        }
    }
    Ok(())
}

fn validate_simple_relation_source(
    context: &ValidationContext<'_, '_>,
    kind: &str,
    source: &ReviewedRelationSource,
) -> Result<(), RuntimeCatalogCompileError> {
    let (path, origin) = match source {
        ReviewedRelationSource::SelectedOutput(path) => (path, Origin::SelectedOutput),
        ReviewedRelationSource::Request(path) => (path, Origin::Request),
    };
    let path_str = value_path_str(path);
    let result = validate_kind_origin_path(
        context,
        kind,
        path_str,
        origin,
        Terminal::TextOrList,
    );
    map_kind_error(kind, path_str, "relation source", result)
}

fn validate_simple_relation_guard(
    context: &ValidationContext<'_, '_>,
    kind: &str,
    source: &ReviewedRelationSource,
) -> Result<(), RuntimeCatalogCompileError> {
    let (path, origin) = match source {
        ReviewedRelationSource::SelectedOutput(path) => (path, Origin::SelectedOutput),
        ReviewedRelationSource::Request(path) => (path, Origin::Request),
    };
    let path_str = value_path_str(path);
    let result = validate_kind_origin_path(
        context,
        kind,
        path_str,
        origin,
        Terminal::Text,
    );
    map_kind_error(kind, path_str, "relation guard", result)
}

fn validate_simple_relation_target(
    context: &ValidationContext<'_, '_>,
    _source_kind: &str,
    target_kinds: &ReviewedTargetKinds,
    target: &ReviewedRelationTarget,
) -> Result<(), RuntimeCatalogCompileError> {
    target_kinds.iter().try_for_each(|kind| match target {
        ReviewedRelationTarget::SelectedOutput(path) => {
            let path_str = value_path_str(path);
            let result = validate_kind_origin_path(
                context,
                kind,
                path_str,
                Origin::SelectedOutput,
                Terminal::Text,
            );
            map_kind_error(kind, path_str, "relation target", result)
        }
        ReviewedRelationTarget::BoundContext(path) => {
            let path_str = value_path_str(path);
            let result = validate_kind_origin_path(
                context,
                kind,
                path_str,
                Origin::Request,
                Terminal::Text,
            );
            map_kind_error(kind, path_str, "relation target", result)
        }
        ReviewedRelationTarget::CanonicalId => Ok(()),
    })
}

fn validate_compound_component_source(
    context: &ValidationContext<'_, '_>,
    kind: &str,
    source: &ReviewedRelationSource,
) -> Result<(), RuntimeCatalogCompileError> {
    let (path, origin) = match source {
        ReviewedRelationSource::SelectedOutput(path) => (path, Origin::SelectedOutput),
        ReviewedRelationSource::Request(path) => (path, Origin::Request),
    };
    let path_str = value_path_str(path);
    let result = validate_kind_origin_path(
        context,
        kind,
        path_str,
        origin,
        Terminal::Text,
    );
    map_kind_error(kind, path_str, "relation source", result)
}

fn validate_compound_component_target(
    context: &ValidationContext<'_, '_>,
    kind: &str,
    target: &ReviewedRelationTarget,
) -> Result<(), RuntimeCatalogCompileError> {
    match target {
        ReviewedRelationTarget::SelectedOutput(path) => {
            let path_str = value_path_str(path);
            let result = validate_kind_origin_path(
                context,
                kind,
                path_str,
                Origin::SelectedOutput,
                Terminal::Text,
            );
            map_kind_error(kind, path_str, "relation target", result)
        }
        ReviewedRelationTarget::BoundContext(path) => {
            let path_str = value_path_str(path);
            let result = validate_kind_origin_path(
                context,
                kind,
                path_str,
                Origin::Request,
                Terminal::Text,
            );
            map_kind_error(kind, path_str, "relation target", result)
        }
        ReviewedRelationTarget::CanonicalId => Ok(()),
    }
}

fn map_kind_error(
    kind: &str,
    path: &str,
    side: &'static str,
    result: Result<(), RuntimeCatalogCompileError>,
) -> Result<(), RuntimeCatalogCompileError> {
    match result {
        Err(RuntimeCatalogCompileError::KindContext { reason, .. }) => {
            Err(RuntimeCatalogCompileError::KindContext {
                kind: kind.to_owned(),
                path: path.to_owned(),
                reason: relation_reason(side, reason),
            })
        }
        Err(RuntimeCatalogCompileError::Path { path: inner }) => {
            Err(RuntimeCatalogCompileError::Path {
                path: format!("{side}: {inner}"),
            })
        }
        Err(RuntimeCatalogCompileError::CapabilityContext {
            service,
            operation,
            path: inner,
        }) => Err(RuntimeCatalogCompileError::CapabilityContext {
            service,
            operation,
            path: format!("{side}: {inner}"),
        }),
        other => other,
    }
}

fn relation_reason(side: &'static str, original: &'static str) -> &'static str {
    match (side, original) {
        ("relation source", "no operation row produces this kind") => {
            "relation source: no operation row produces this kind"
        }
        ("relation target", "no operation row produces this kind") => {
            "relation target: no operation row produces this kind"
        }
        ("relation guard", "no operation row produces this kind") => {
            "relation guard: no operation row produces this kind"
        }
        ("relation source", "path does not end in a string") => {
            "relation source: path does not end in a string"
        }
        ("relation target", "path does not end in a string") => {
            "relation target: path does not end in a string"
        }
        ("relation guard", "path does not end in a string") => {
            "relation guard: path does not end in a string"
        }
        ("relation source", "path is a protected projection") => {
            "relation source: path is a protected projection"
        }
        ("relation target", "path is a protected projection") => {
            "relation target: path is a protected projection"
        }
        ("relation guard", "path is a protected projection") => {
            "relation guard: path is a protected projection"
        }
        ("relation source", "no callable emission of this kind resolves the path") => {
            "relation source: no callable emission of this kind resolves the path"
        }
        ("relation target", "no callable emission of this kind resolves the path") => {
            "relation target: no callable emission of this kind resolves the path"
        }
        ("relation guard", "no callable emission of this kind resolves the path") => {
            "relation guard: no callable emission of this kind resolves the path"
        }
        _ => original,
    }
}

fn value_path_str(path: &ReviewedValuePath) -> &str {
    match path {
        ReviewedValuePath::Fields(path) => path.as_str(),
        ReviewedValuePath::RootText => "",
    }
}