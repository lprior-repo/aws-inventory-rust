use super::{ValidationContext, validate_kind_origin_path};
use crate::kind_map::{
    ReviewedRelationSource, ReviewedRelationTarget, ReviewedTargetKinds, ReviewedValuePath,
};
use crate::runtime_catalog::RuntimeCatalogCompileError;
use crate::runtime_catalog::model_path::emission::{Origin, Terminal};

pub(super) fn validate_relations(
    context: &ValidationContext<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    for relation in context.kinds.relations().iter() {
        if !active(context, &relation.source_kind, &relation.target_kinds) {
            continue;
        }
        validate_source(context, relation)?;
        for kind in relation.target_kinds.iter() {
            target(context, kind, &relation.target)?;
        }
    }
    validate_compound_relations(context)
}

fn validate_source(
    context: &ValidationContext<'_, '_>,
    relation: &crate::kind_map::ReviewedRelation,
) -> Result<(), RuntimeCatalogCompileError> {
    source(
        context,
        &relation.source_kind,
        &relation.source,
        Terminal::TextOrList,
        "relation source",
    )?;
    if let Some(guard) = &relation.guard {
        source(
            context,
            &relation.source_kind,
            &guard.source,
            Terminal::Text,
            "relation guard",
        )?;
    }
    Ok(())
}

fn active(
    context: &ValidationContext<'_, '_>,
    source: &str,
    targets: &ReviewedTargetKinds,
) -> bool {
    crate::runtime_catalog::model_path::active_kind(context, source)
        && targets
            .iter()
            .all(|kind| crate::runtime_catalog::model_path::active_kind(context, kind))
}

fn validate_compound_relations(
    context: &ValidationContext<'_, '_>,
) -> Result<(), RuntimeCatalogCompileError> {
    for relation in context.kinds.compound_relations().iter() {
        if !active(context, &relation.source_kind, &relation.target_kinds) {
            continue;
        }
        for component in &relation.components {
            source(
                context,
                &relation.source_kind,
                &component.source,
                Terminal::Text,
                "relation source",
            )?;
            for kind in relation.target_kinds.iter() {
                target(context, kind, &component.target)?;
            }
        }
    }
    Ok(())
}

fn source(
    context: &ValidationContext<'_, '_>,
    kind: &str,
    source: &ReviewedRelationSource,
    terminal: Terminal,
    side: &'static str,
) -> Result<(), RuntimeCatalogCompileError> {
    let (path, origin) = match source {
        ReviewedRelationSource::SelectedOutput(path) => (path, Origin::SelectedOutput),
        ReviewedRelationSource::Request(path) => (path, Origin::Request),
    };
    with_side(
        side,
        validate_kind_origin_path(context, kind, value_path_str(path), origin, terminal),
    )
}

fn target(
    context: &ValidationContext<'_, '_>,
    kind: &str,
    target: &ReviewedRelationTarget,
) -> Result<(), RuntimeCatalogCompileError> {
    let (path, origin) = match target {
        ReviewedRelationTarget::SelectedOutput(path) => (path, Origin::SelectedOutput),
        ReviewedRelationTarget::BoundContext(path) => (path, Origin::Request),
        ReviewedRelationTarget::CanonicalId => return Ok(()),
    };
    with_side(
        "relation target",
        validate_kind_origin_path(context, kind, value_path_str(path), origin, Terminal::Text),
    )
}

fn with_side(
    side: &'static str,
    result: Result<(), RuntimeCatalogCompileError>,
) -> Result<(), RuntimeCatalogCompileError> {
    result.map_err(|error| match error {
        RuntimeCatalogCompileError::KindContext { kind, path, reason } => {
            RuntimeCatalogCompileError::KindContext {
                kind,
                path,
                reason: relation_reason(side, reason),
            }
        }
        error => error,
    })
}

fn relation_reason(side: &'static str, reason: &'static str) -> &'static str {
    match side {
        "relation source" => source_reason(reason),
        "relation target" => target_reason(reason),
        "relation guard" => guard_reason(reason),
        _ => reason,
    }
}

fn source_reason(reason: &'static str) -> &'static str {
    match reason {
        "no operation row produces this kind" => {
            "relation source: no operation row produces this kind"
        }
        "path does not end in a string" => "relation source: path does not end in a string",
        "path is a protected projection" => "relation source: path is a protected projection",
        _ => reason,
    }
}

fn target_reason(reason: &'static str) -> &'static str {
    match reason {
        "no operation row produces this kind" => {
            "relation target: no operation row produces this kind"
        }
        "path does not end in a string" => "relation target: path does not end in a string",
        "path is a protected projection" => "relation target: path is a protected projection",
        _ => reason,
    }
}

fn guard_reason(reason: &'static str) -> &'static str {
    match reason {
        "no operation row produces this kind" => {
            "relation guard: no operation row produces this kind"
        }
        "path does not end in a string" => "relation guard: path does not end in a string",
        "path is a protected projection" => "relation guard: path is a protected projection",
        _ => reason,
    }
}

fn value_path_str(path: &ReviewedValuePath) -> &str {
    match path {
        ReviewedValuePath::Fields(path) => path.as_str(),
        ReviewedValuePath::RootText => "",
    }
}
