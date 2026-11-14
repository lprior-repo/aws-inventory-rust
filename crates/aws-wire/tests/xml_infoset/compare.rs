use super::Node;
use inventory_core::catalog::{HttpBinding, Operation, Service, Shape, Target};
#[path = "compare_maps.rs"]
mod maps;

pub(super) fn root_target(
    service: Service<'_>,
    operation: Operation<'_>,
) -> Option<(Target, Option<String>)> {
    let target = operation.input().ok()?;
    let index = match target {
        Target::Shape(index) => index,
        Target::Prelude(_) => return Some((target, None)),
        _ => return None,
    };
    let shape = service.shape(index).ok()??;
    let members = shape.members().ok()?.collect::<Result<Vec<_>, _>>().ok()?;
    let mut payload = None;
    for member in members {
        if matches!(member.wire().ok()?.binding().ok()?, HttpBinding::Payload) {
            if payload.is_some() {
                return None;
            }
            payload = Some(member);
        }
    }
    match payload {
        Some(member) => {
            let target = member.target;
            let member_name = member.wire().ok()?.xml_name().ok()?.map(str::to_owned);
            let target_name = shape_name(service, target)?;
            Some((target, member_name.or(Some(target_name))))
        }
        None => Some((target, Some(shape_name(service, target)?))),
    }
}

fn shape_name(service: Service<'_>, target: Target) -> Option<String> {
    match target {
        Target::Shape(index) => {
            let shape = service.shape(index).ok()??;
            shape
                .wire()
                .ok()?
                .xml_name()
                .ok()?
                .map(str::to_owned)
                .or_else(|| shape.name().ok().map(str::to_owned))
        }
        Target::Prelude(_) => None,
        _ => None,
    }
}

pub(super) fn equal_roots(
    left: &Node,
    right: &Node,
    service: Service<'_>,
    target: Target,
    expected_name: Option<&str>,
) -> bool {
    if left.children.len() != 1
        || right.children.len() != 1
        || !left.text.trim().is_empty()
        || !right.text.trim().is_empty()
    {
        return false;
    }
    match (left.children.first(), right.children.first()) {
        (Some(l), Some(r)) => {
            let expected = match expected_name {
                Some(name) => local_name(&l.name) == name && local_name(&r.name) == name,
                None => true,
            };
            expected && compare(l, r, service, target, 0)
        }
        _ => false,
    }
}

fn compare(left: &Node, right: &Node, service: Service<'_>, target: Target, depth: usize) -> bool {
    if depth >= 256 || left.name != right.name || left.attributes != right.attributes {
        return false;
    }
    let index = match target {
        Target::Shape(index) => index,
        Target::Prelude(_) => {
            return left.children.is_empty()
                && right.children.is_empty()
                && left.text == right.text;
        }
        _ => return false,
    };
    let Ok(Some(shape)) = service.shape(index) else {
        return false;
    };
    let Ok(kind) = shape.kind() else {
        return false;
    };
    let container = matches!(kind, "structure" | "union" | "list" | "map");
    if !text_matches(&left.text, &right.text, container) {
        return false;
    }
    match kind {
        "structure" | "union" => compare_structure(left, right, service, shape, depth),
        "list" => compare_list(left, right, service, shape, depth),
        "map" => maps::compare_map(left, right, service, shape, depth, "entry"),
        _ => left.children.is_empty() && right.children.is_empty(),
    }
}

fn text_matches(left: &str, right: &str, container: bool) -> bool {
    if container && left.trim().is_empty() && right.trim().is_empty() {
        true
    } else {
        left == right
    }
}

fn compare_structure(
    left: &Node,
    right: &Node,
    service: Service<'_>,
    shape: Shape<'_>,
    depth: usize,
) -> bool {
    if matches!(shape.kind(), Ok("union")) && (left.children.len() > 1 || right.children.len() > 1)
    {
        return false;
    }
    let Ok(members) = shape.members() else {
        return false;
    };
    let Ok(members) = members.collect::<Result<Vec<_>, _>>() else {
        return false;
    };
    members.iter().all(|member| {
        let Ok(wire) = member.wire() else {
            return false;
        };
        if !matches!(wire.binding(), Ok(HttpBinding::Document))
            || matches!(wire.xml_attribute(), Ok(true))
        {
            return true;
        }
        let Some((name, flattened, member_shape)) = member_info(*member, service) else {
            return false;
        };
        let list = member_shape.filter(|item| matches!(item.kind(), Ok("list")));
        let tag = name;
        let l = children_named(&left.children, tag);
        let r = children_named(&right.children, tag);
        if l.len() != r.len() {
            return false;
        }
        let map = member_shape.filter(|item| matches!(item.kind(), Ok("map")));
        if !flattened && list.is_none() && map.is_none() && l.len() > 1 {
            return false;
        }
        match (flattened, list, map) {
            (true, Some(list), _) => {
                let Ok(Some(element)) = list.element() else {
                    return false;
                };
                l.iter()
                    .zip(&r)
                    .all(|(a, b)| compare(a, b, service, element, depth + 1))
            }
            (true, _, Some(map)) => maps::compare_map_nodes(&l, &r, service, map, depth + 1, name),
            _ => l
                .iter()
                .zip(&r)
                .all(|(a, b)| compare(a, b, service, member.target, depth + 1)),
        }
    }) && known_children(left, service, shape)
        && known_children(right, service, shape)
}

fn member_info<'a>(
    member: inventory_core::catalog::Member<'a>,
    service: Service<'a>,
) -> Option<(&'a str, bool, Option<Shape<'a>>)> {
    let wire = member.wire().ok()?;
    if !matches!(wire.binding(), Ok(HttpBinding::Document))
        || matches!(wire.xml_attribute(), Ok(true))
    {
        return None;
    }
    let name = wire.xml_name().ok()?.or(member.name().ok())?;
    let shape = match member.target {
        Target::Shape(index) => service.shape(index).ok().flatten(),
        Target::Prelude(_) => None,
        _ => return None,
    };
    let flattened = if wire.xml_flattened().ok()? {
        true
    } else if let Some(shape) = shape {
        shape.wire().ok()?.xml_flattened().ok()?
    } else {
        false
    };
    Some((name, flattened, shape))
}

fn known_children(node: &Node, service: Service<'_>, shape: Shape<'_>) -> bool {
    let Ok(members) = shape.members() else {
        return false;
    };
    let Ok(members) = members.collect::<Result<Vec<_>, _>>() else {
        return false;
    };
    node.children.iter().all(|child| {
        members.iter().any(|member| {
            let Some((name, _, _)) = member_info(*member, service) else {
                return false;
            };
            local_name(&child.name) == name
        })
    })
}

fn compare_list(
    left: &Node,
    right: &Node,
    service: Service<'_>,
    shape: Shape<'_>,
    depth: usize,
) -> bool {
    let (Ok(Some(target)), Ok(wire)) = (shape.element(), shape.element_wire()) else {
        return false;
    };
    let name = match wire.xml_name() {
        Ok(Some(name)) => name,
        Ok(None) => "member",
        Err(_) => return false,
    };
    left.children.len() == right.children.len()
        && left.children.iter().zip(&right.children).all(|(l, r)| {
            local_name(&l.name) == name
                && local_name(&r.name) == name
                && compare(l, r, service, target, depth + 1)
        })
}

fn children_named<'a>(children: &'a [Node], name: &str) -> Vec<&'a Node> {
    children
        .iter()
        .filter(|child| local_name(&child.name) == name)
        .collect()
}

fn local_name(name: &str) -> &str {
    match name.rsplit('}').next() {
        Some(local) => local,
        None => name,
    }
}
