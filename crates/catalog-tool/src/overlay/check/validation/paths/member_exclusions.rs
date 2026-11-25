use super::*;
use crate::plans::model::{ShapeRef, shape_type, target};
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) fn validate(
    service: &str,
    operation: &str,
    item: &OperationOverlay,
    model: &Model<'_>,
    violations: &mut Vec<Violation>,
) {
    check_paths(
        service,
        operation,
        "exclude_response_members",
        &item.exclude_response_members,
        model,
        root_selection(model, operation).as_ref(),
        violations,
    );
    check_count(service, operation, item, violations);
    check_safety(service, operation, item, model, violations);
}

fn check_count(
    service: &str,
    operation: &str,
    item: &OperationOverlay,
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
    for path in &item.exclude_response_members {
        let path_str = path.as_str();
        if let Err(reason) = path_walk(model, root, path_str) {
            violations.push(violation(
                service,
                operation,
                "exclude_response_members",
                &format!("{path_str}: {reason}"),
                ViolationRule::InvalidPath,
            ));
        } else if let Err(reason) = unique_struct_member(model, root, path_str) {
            violations.push(violation(
                service,
                operation,
                "exclude_response_members",
                &format!("{path_str}: {reason}"),
                ViolationRule::InvalidPath,
            ));
        }
    }
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

fn unique_struct_member<'a>(
    model: &Model<'a>,
    root: ShapeRef<'a>,
    path: &str,
) -> Result<(), &'static str> {
    let name = path.rsplit('.').next().ok_or("empty path")?;
    let mut pending = vec![root];
    let mut visits = BTreeMap::new();
    let mut sites = 0;
    while let Some(shape) = pending.pop() {
        let seen = visits.entry(shape.id).or_insert(0u8);
        if *seen == 2 {
            continue;
        }
        *seen = seen.saturating_add(1);
        sites = check_site(shape, name, sites)?;
        enqueue_children(model, shape, &mut pending)?;
    }
    Ok(())
}

fn check_site(shape: ShapeRef<'_>, name: &str, sites: u8) -> Result<u8, &'static str> {
    if !matches!(shape_type(shape.value), Some("structure" | "union")) {
        return Ok(sites);
    }
    if shape
        .value
        .get("members")
        .and_then(|members| members.get(name))
        .is_none()
    {
        return Ok(sites);
    }
    if shape_type(shape.value) == Some("union") {
        return Err("excluded member has a union parent");
    }
    if sites != 0 {
        return Err("terminal member name occurs at multiple output sites");
    }
    Ok(1)
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
