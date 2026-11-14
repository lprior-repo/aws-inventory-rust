use super::{
    Error, Expansion, ExpansionReason, LargestPageSize, Model, Pagination, PaginationReason,
    Vocabulary,
    counts::Counts,
    expansion,
    model::{name, trait_value},
    pagination,
};
use crate::classes::{Class, DenyList, classify};
use std::collections::BTreeSet;

pub(super) fn collect(
    model: &Model<'_>,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
    matched: &mut BTreeSet<usize>,
) -> Result<Counts, Error> {
    let mut counts = Counts::default();
    model.of_type("operation").try_for_each(|(id, _)| {
        let facts = model.facts(id)?;
        matched.extend(deny.0.iter().filter(|e| e.matches(&facts)).map(|e| e.line));
        operation(
            &mut counts,
            model,
            id,
            classify(&facts, deny),
            vocabulary,
            deny,
        )
    })?;
    model
        .of_type("resource")
        .try_for_each(|(_, r)| resource(&mut counts, r))?;
    counts.validate()?;
    Ok(counts)
}

fn operation(
    c: &mut Counts,
    model: &Model<'_>,
    id: &str,
    class: Class,
    vocabulary: &Vocabulary,
    deny: &DenyList<'_>,
) -> Result<(), Error> {
    record_class(c, class)?;
    if class == Class::Expand {
        record_expansion(c, expansion(model, id, deny)?)?;
    }
    let named = name(id).starts_with("List") || name(id).starts_with("Describe");
    if named || trait_value(model.operation(id)?, "smithy.api#paginated").is_some() {
        record_pagination(c, model, id, vocabulary)?;
    }
    if named && matches!(class, Class::Denied(_)) {
        c.increment("list_describe_denied")?;
    }
    batch(c, model, name(id))
}

fn record_class(c: &mut Counts, class: Class) -> Result<(), Error> {
    c.increment("operations")?;
    c.increment(match class {
        Class::Enumerate => "enumerate",
        Class::Expand => "expand",
        Class::Denied(_) => "denied",
        Class::Unclassified => "unclassified",
    })
}

fn record_expansion(c: &mut Counts, expansion: Expansion<'_>) -> Result<(), Error> {
    match expansion {
        Expansion::Modeled {
            list_source,
            ambiguous_resource,
            ..
        } => {
            modeled_expansion(c, list_source, ambiguous_resource)?;
        }
        Expansion::NeedsOverlay(reason) => {
            c.increment("expand_needs_overlay")?;
            c.increment(expansion_key(reason))?;
        }
    }
    Ok(())
}

fn modeled_expansion(c: &mut Counts, list_source: bool, ambiguous: bool) -> Result<(), Error> {
    c.increment("expand_modeled")?;
    if list_source {
        c.increment("expand_modeled_with_list_source")?;
    }
    if ambiguous {
        c.increment("ambiguous_resource")?;
    }
    Ok(())
}
fn expansion_key(reason: ExpansionReason) -> &'static str {
    match reason {
        ExpansionReason::NotInResource => "expand_needs_overlay_not_in_resource",
        ExpansionReason::RequiredMemberNotIdentifier => {
            "expand_needs_overlay_required_member_not_identifier"
        }
    }
}

fn record_pagination(
    c: &mut Counts,
    model: &Model<'_>,
    id: &str,
    vocab: &Vocabulary,
) -> Result<(), Error> {
    let p = pagination(model, id, vocab)?;
    if matches!(p, Pagination::Modeled { .. }) {
        page_size(c, model, id, &p)?;
    }
    if !(name(id).starts_with("List") || name(id).starts_with("Describe")) {
        return Ok(());
    }
    c.increment("list_describe")?;
    if matches!(p, Pagination::Modeled { .. }) {
        return c.increment("list_describe_paginated");
    }
    record_unmodeled(c, p)
}
fn record_unmodeled(c: &mut Counts, p: Pagination<'_>) -> Result<(), Error> {
    c.increment("list_describe_without_paginator")?;
    match p {
        Pagination::Unpaginated => c.increment("unpaginated"),
        Pagination::NeedsOverlay(reason) => {
            c.increment("pagination_needs_overlay")?;
            c.increment(pagination_key(reason))
        }
        Pagination::Modeled { .. } => Err(Error::CountMismatch),
    }
}
fn pagination_key(reason: PaginationReason) -> &'static str {
    match reason {
        PaginationReason::TokenBothSides => "pagination_needs_overlay_token_both_sides",
        PaginationReason::TokenInputOnly => "pagination_needs_overlay_token_input_only",
        PaginationReason::TokenOutputOnly => "pagination_needs_overlay_token_output_only",
        PaginationReason::TruncationFlag => "pagination_needs_overlay_truncation_flag",
    }
}
fn page_size(c: &mut Counts, model: &Model<'_>, id: &str, p: &Pagination<'_>) -> Result<(), Error> {
    c.increment("paginated")?;
    let op = trait_value(model.operation(id)?, "smithy.api#paginated");
    if op.is_some_and(|v| v.get("inputToken").is_none() || v.get("outputToken").is_none()) {
        c.increment("service_tokens_completed")?;
    }
    c.increment(match p.largest_page_size(model, id)? {
        Some(LargestPageSize::Known(_)) => "page_size_known",
        Some(LargestPageSize::Unbounded) => "page_size_unbounded",
        None => "paginated_without_page_size",
    })
}
fn resource(c: &mut Counts, r: &serde_json::Value) -> Result<(), Error> {
    c.increment("resources")?;
    if r.get("list").is_some() {
        c.increment("resources_with_list")?;
    }
    if r.get("read").is_some() {
        c.increment("resources_with_read")?;
    }
    if r.get("list").is_some() && r.get("read").is_some() {
        c.increment("resources_with_list_and_read")?;
    }
    if trait_value(r, "aws.api#arn").is_some() {
        c.increment("resources_with_arn")?;
    }
    Ok(())
}
fn batch(c: &mut Counts, model: &Model<'_>, operation: &str) -> Result<(), Error> {
    if !(operation.starts_with("BatchGet") || operation.starts_with("BatchDescribe")) {
        return Ok(());
    }
    c.increment("batch_reads")?;
    let Some(single) = operation.strip_prefix("Batch") else {
        return Ok(());
    };
    if model
        .of_type("operation")
        .any(|(id, _)| name(id) == single || single.strip_suffix('s') == Some(name(id)))
    {
        c.increment("batch_reads_with_single_counterpart")?;
    }
    Ok(())
}
