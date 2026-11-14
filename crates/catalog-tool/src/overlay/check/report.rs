use super::{CheckError, Violation};
use crate::{
    classes::Class,
    plans::{self, Model},
};
use std::fmt::Write as _;

pub(super) fn format_violations(violations: &[Violation]) -> String {
    violations
        .iter()
        .map(|error| {
            format!(
                "{}/{} {}={} {:?}",
                error.service, error.operation, error.key, error.value, error.rule
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

pub(super) fn plan_sources(
    plan: Option<&super::super::OperationOverlay>,
    class: Class,
    effective: &str,
    model: &Model<'_>,
    operation_id: Option<&str>,
    deny: &crate::classes::DenyList<'_>,
    vocabulary: &plans::Vocabulary,
) -> String {
    let modeled = modeled_sources(effective == "expand", model, operation_id, deny, vocabulary);
    let class_source = plan
        .filter(|plan| plan.class.is_some())
        .map_or_else(|| class_source(class), |_| "overlay");
    format_sources(class_source, source_fields(plan, modeled))
}

fn format_sources(class: &str, sources: [&str; 7]) -> String {
    let [
        items,
        page_size,
        paginator,
        identity,
        bindings,
        scope,
        batch,
    ] = sources;
    format!(
        "class={class},items={items},page_size={page_size},paginator={paginator},identity={identity},bindings={bindings},scope={scope},batch={batch}"
    )
}

fn source_fields(
    plan: Option<&super::super::OperationOverlay>,
    modeled: ModeledSources,
) -> [&'static str; 7] {
    [
        source(plan.is_some_and(|plan| plan.items.is_some()), modeled.items),
        source(
            plan.is_some_and(|plan| plan.page_size.is_some()),
            modeled.page_size,
        ),
        source(
            plan.is_some_and(|plan| plan.paginator.is_some()),
            modeled.paginator,
        ),
        source(plan.is_some_and(|plan| plan.identity.is_some()), false),
        source(
            plan.is_some_and(|plan| !plan.bind.is_empty()),
            modeled.bindings,
        ),
        source(plan.is_some_and(|plan| plan.scope.is_some()), false),
        source(plan.is_some_and(|plan| plan.batch.is_some()), false),
    ]
}

struct ModeledSources {
    items: bool,
    paginator: bool,
    page_size: bool,
    bindings: bool,
}

fn modeled_sources(
    is_expand: bool,
    model: &Model<'_>,
    operation_id: Option<&str>,
    deny: &crate::classes::DenyList<'_>,
    vocabulary: &plans::Vocabulary,
) -> ModeledSources {
    let pagination = operation_id.and_then(|id| plans::pagination(model, id, vocabulary).ok());
    let (items, paginator, page_size) = pagination_sources(pagination.as_ref());
    ModeledSources {
        items,
        paginator,
        page_size,
        bindings: modeled_bindings(is_expand, model, operation_id, deny),
    }
}

fn pagination_sources(pagination: Option<&plans::Pagination<'_>>) -> (bool, bool, bool) {
    (
        matches!(
            pagination,
            Some(plans::Pagination::Modeled { items: Some(_), .. })
        ),
        matches!(pagination, Some(plans::Pagination::Modeled { .. })),
        matches!(
            pagination,
            Some(plans::Pagination::Modeled {
                page_size: Some(_),
                ..
            })
        ),
    )
}

fn modeled_bindings(
    is_expand: bool,
    model: &Model<'_>,
    operation_id: Option<&str>,
    deny: &crate::classes::DenyList<'_>,
) -> bool {
    is_expand
        && operation_id
            .and_then(|id| plans::expansion(model, id, deny).ok())
            .is_some_and(|plan| matches!(plan, plans::Expansion::Modeled { .. }))
}

fn class_source(class: Class) -> &'static str {
    if matches!(class, Class::Denied(_)) {
        "deny-list"
    } else {
        "model"
    }
}

fn source(overlay: bool, modeled: bool) -> &'static str {
    if overlay {
        "overlay"
    } else if modeled {
        "model"
    } else {
        "missing"
    }
}

pub(super) fn render(
    rows: &[(String, String, String, String)],
    services: usize,
    reviewed: usize,
    unreviewed: [usize; 3],
) -> Result<String, CheckError> {
    let mut text = String::new();
    write_totals(&mut text, rows, services, reviewed, unreviewed)?;
    write_rows(&mut text, rows)?;
    Ok(text)
}

fn write_totals(
    text: &mut String,
    rows: &[(String, String, String, String)],
    services: usize,
    reviewed: usize,
    unreviewed: [usize; 3],
) -> Result<(), CheckError> {
    let counts = ReportCounts::from_rows(rows);
    writeln!(
        text,
        "totals\tservices_with_overlay={reviewed}\tcallable_enumerate={}\tcallable_expand={}\tdenied_by_overlay={}\tdeferred={}\tservices_without_overlay={}\tunreviewed_enumerate={}\tunreviewed_expand={}\tunreviewed_unclassified={}",
        counts.callable_enumerate,
        counts.callable_expand,
        counts.denied,
        counts.deferred,
        services.saturating_sub(reviewed),
        unreviewed[0],
        unreviewed[1],
        unreviewed[2]
    )?;
    Ok(())
}

struct ReportCounts {
    callable_enumerate: usize,
    callable_expand: usize,
    denied: usize,
    deferred: usize,
}

impl ReportCounts {
    fn from_rows(rows: &[(String, String, String, String)]) -> Self {
        Self {
            callable_enumerate: rows.iter().filter(|row| row.2 == "enumerate").count(),
            callable_expand: rows.iter().filter(|row| row.2 == "expand").count(),
            denied: rows
                .iter()
                .filter(|row| row.2 == "denied" && row.3.starts_with("class=overlay"))
                .count(),
            deferred: rows.iter().filter(|row| row.2 == "deferred").count(),
        }
    }
}

fn write_rows(
    text: &mut String,
    rows: &[(String, String, String, String)],
) -> Result<(), CheckError> {
    for (service, operation, effective, source) in rows {
        writeln!(
            text,
            "operation\t{service}\t{operation}\t{effective}\t{source}"
        )?;
    }
    Ok(())
}
