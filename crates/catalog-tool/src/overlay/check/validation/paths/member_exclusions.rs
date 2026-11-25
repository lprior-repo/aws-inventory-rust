use super::*;
use crate::plans::model::{ShapeRef, shape_type, target};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate(
    service: &str,
    operation: &str,
    item: &OperationOverlay,
    model: &Model<'_>,
    violations: &mut Vec<Violation>,
) {
    if item.exclude_response_members.len() > 8 {
        violations.push(violation(
            service,
            operation,
            "exclude_response_members",
            "more than 8 paths",
            ViolationRule::InvalidPath,
        ));
    }
    check_paths(
        service,
        operation,
        "exclude_response_members",
        &item.exclude_response_members,
        model,
        root_selection(model, operation).as_ref(),
        violations,
    );
    check_safety(service, operation, item, model, violations);
}

fn check_safety(
    service: &str,
    operation: &str,
    item: &OperationOverlay,
    model: &Model<'_>,
    violations: &mut Vec<Violation>,
) {
    let Some(root) = root_selection(model, operation) else {
        return;
    };
    let reviewed = item
        .exclude_response_members
        .iter()
        .filter_map(|path| {
            let path = path.as_str();
            match path_walk(model, root, path) {
                Ok(()) => Some(path),
                Err(reason) => {
                    report_site_error(service, operation, path, reason, violations);
                    None
                }
            }
        })
        .collect::<BTreeSet<_>>();
    let site_counts = reviewed
        .into_iter()
        .fold(BTreeMap::new(), |mut counts, path| {
            let name = path.rsplit_once('.').map_or(path, |(_, name)| name);
            let count = counts.entry(name).or_insert(0usize);
            *count = count.saturating_add(1);
            counts
        });
    site_counts.into_iter().for_each(|(name, approved)| {
        if let Err(reason) = validate_sites(model, root, name, approved) {
            report_site_error(service, operation, name, reason, violations);
        }
    });
}

fn report_site_error(
    service: &str,
    operation: &str,
    path: &str,
    reason: &str,
    violations: &mut Vec<Violation>,
) {
    violations.push(violation(
        service,
        operation,
        "exclude_response_members",
        &format!("{path}: {reason}"),
        ViolationRule::InvalidPath,
    ));
}

fn path_walk<'a>(model: &Model<'a>, root: ShapeRef<'a>, path: &str) -> Result<(), &'static str> {
    let mut current = root;
    for segment in path.split('.') {
        current = step_lists(model, current)?;
        if shape_type(current.value) == Some("map") {
            return Err("path traverses a map");
        }
        if !matches!(shape_type(current.value), Some("structure" | "union")) {
            return Err("path traverses neither a structure nor a union");
        }
        let members = current
            .value
            .get("members")
            .and_then(Value::as_object)
            .ok_or("structure has no members")?;
        let member = members
            .get(segment)
            .ok_or("member is not a structure member")?;
        let target_id = target(member).ok_or("structure member target is absent")?;
        current = model
            .shape(target_id)
            .ok_or("structure member target does not exist")?;
    }
    Ok(())
}

fn step_lists<'a>(model: &Model<'a>, shape: ShapeRef<'a>) -> Result<ShapeRef<'a>, &'static str> {
    let mut current = shape;
    for _ in 0..8 {
        if shape_type(current.value) != Some("list") {
            return Ok(current);
        }
        let member = current.value.get("member").ok_or("list has no member")?;
        let target_id = target(member).ok_or("list member target is absent")?;
        current = model
            .shape(target_id)
            .ok_or("list member target does not exist")?;
    }
    if shape_type(current.value) == Some("list") {
        return Err("path nests more than 8 lists");
    }
    Ok(current)
}

fn validate_sites<'a>(
    model: &Model<'a>,
    root: ShapeRef<'a>,
    name: &str,
    approved: usize,
) -> Result<(), &'static str> {
    let mut pending = vec![root];
    let mut visits = BTreeMap::new();
    let mut sites = 0usize;
    while let Some(shape) = pending.pop() {
        let seen = visits.entry(shape.id).or_insert(0usize);
        if *seen > approved {
            continue;
        }
        *seen = seen.saturating_add(1);
        sites = sites.saturating_add(usize::from(is_struct_site(shape, name)?));
        if sites > approved {
            return Err("terminal member name occurs at multiple output sites not all reviewed");
        }
        enqueue_children(model, shape, &mut pending)?;
    }
    if sites != approved {
        return Err("reviewed response paths do not match modeled output sites");
    }
    Ok(())
}

fn is_struct_site(shape: ShapeRef<'_>, name: &str) -> Result<bool, &'static str> {
    if !matches!(shape_type(shape.value), Some("structure" | "union")) {
        return Ok(false);
    }
    if shape
        .value
        .get("members")
        .and_then(|members| members.get(name))
        .is_none()
    {
        return Ok(false);
    }
    if shape_type(shape.value) == Some("union") {
        return Err("excluded member has a union parent");
    }
    Ok(true)
}

fn enqueue_children<'a>(
    model: &Model<'a>,
    shape: ShapeRef<'a>,
    pending: &mut Vec<ShapeRef<'a>>,
) -> Result<(), &'static str> {
    if !matches!(
        shape_type(shape.value),
        Some("structure" | "union" | "list" | "map")
    ) {
        return Ok(());
    }
    for member in children(shape.value) {
        let id = target(member).ok_or("output member target is absent")?;
        pending.push(
            model
                .shape(id)
                .ok_or("output member target does not exist")?,
        );
    }
    Ok(())
}

fn children(shape: &Value) -> impl Iterator<Item = &Value> {
    let members = shape
        .get("members")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|members| members.values());
    let elements = ["member", "value"]
        .into_iter()
        .filter_map(|name| shape.get(name));
    members.chain(elements)
}
