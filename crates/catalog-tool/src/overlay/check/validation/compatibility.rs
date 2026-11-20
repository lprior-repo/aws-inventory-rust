use crate::plans::{
    Model,
    model::{self, ShapeRef},
};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub(in crate::overlay::check::validation) fn compatible<'m>(
    model: &Model<'m>,
    left: &ShapeRef<'m>,
    right: &ShapeRef<'m>,
) -> bool {
    compatible_inner(model, *left, *right)
}

type ShapePair<'a> = (ShapeRef<'a>, ShapeRef<'a>);
type StructureMembers<'a> = (&'a Map<String, Value>, &'a Map<String, Value>);

enum Comparison<'a> {
    Compare(ShapeRef<'a>, ShapeRef<'a>, usize),
    Exit((&'a str, &'a str)),
}

fn compatible_inner<'m>(model: &Model<'m>, left: ShapeRef<'m>, right: ShapeRef<'m>) -> bool {
    let mut active = BTreeSet::new();
    let mut pending = vec![Comparison::Compare(left, right, 0)];
    while let Some(comparison) = pending.pop() {
        match comparison {
            Comparison::Exit(pair) => {
                active.remove(&pair);
            }
            Comparison::Compare(left, right, depth) => {
                if !compare_shapes(model, left, right, depth, &mut active, &mut pending) {
                    return false;
                }
            }
        }
    }
    true
}

fn compare_shapes<'m>(
    model: &Model<'m>,
    left: ShapeRef<'m>,
    right: ShapeRef<'m>,
    depth: usize,
    active: &mut BTreeSet<(&'m str, &'m str)>,
    pending: &mut Vec<Comparison<'m>>,
) -> bool {
    if depth > 32 {
        return false;
    }
    match (
        model::shape_type(left.value),
        model::shape_type(right.value),
    ) {
        (Some(left_kind), Some(right_kind)) if left_kind == right_kind => {
            compare_children(model, left, right, left_kind, depth, active, pending)
        }
        _ => false,
    }
}

fn compare_children<'m>(
    model: &Model<'m>,
    left: ShapeRef<'m>,
    right: ShapeRef<'m>,
    kind: &str,
    depth: usize,
    active: &mut BTreeSet<(&'m str, &'m str)>,
    pending: &mut Vec<Comparison<'m>>,
) -> bool {
    match kind {
        "map" => schedule_map(model, left, right, depth, active, pending),
        "list" => schedule_list(model, left, right, depth, active, pending),
        "structure" => schedule_structure(model, left, right, depth, active, pending),
        _ => true,
    }
}

fn schedule_map<'m>(
    model: &Model<'m>,
    left: ShapeRef<'m>,
    right: ShapeRef<'m>,
    depth: usize,
    active: &mut BTreeSet<(&'m str, &'m str)>,
    pending: &mut Vec<Comparison<'m>>,
) -> bool {
    let Some(children) = map_children(model, left.value, right.value) else {
        return false;
    };
    schedule_pair((left.id, right.id), children, depth, active, pending)
}

fn map_children<'m>(model: &Model<'m>, left: &Value, right: &Value) -> Option<[ShapePair<'m>; 2]> {
    Some([
        (
            shape_target(model, left, "key")?,
            shape_target(model, right, "key")?,
        ),
        (
            shape_target(model, left, "value")?,
            shape_target(model, right, "value")?,
        ),
    ])
}

fn schedule_list<'m>(
    model: &Model<'m>,
    left: ShapeRef<'m>,
    right: ShapeRef<'m>,
    depth: usize,
    active: &mut BTreeSet<(&'m str, &'m str)>,
    pending: &mut Vec<Comparison<'m>>,
) -> bool {
    let Some(children) = list_children(model, left.value, right.value) else {
        return false;
    };
    schedule_pair((left.id, right.id), [children], depth, active, pending)
}

fn list_children<'m>(model: &Model<'m>, left: &Value, right: &Value) -> Option<ShapePair<'m>> {
    Some((
        shape_target(model, left, "member")?,
        shape_target(model, right, "member")?,
    ))
}

fn schedule_structure<'m>(
    model: &Model<'m>,
    left: ShapeRef<'m>,
    right: ShapeRef<'m>,
    depth: usize,
    active: &mut BTreeSet<(&'m str, &'m str)>,
    pending: &mut Vec<Comparison<'m>>,
) -> bool {
    let Some((left_members, right_members)) = structure_members(left.value, right.value) else {
        return false;
    };
    if left_members.len() != right_members.len() {
        return false;
    }
    schedule_members(
        model,
        (left.id, right.id),
        left_members,
        right_members,
        depth,
        active,
        pending,
    )
}

fn schedule_members<'m>(
    model: &Model<'m>,
    pair: (&'m str, &'m str),
    left: &'m Map<String, Value>,
    right: &'m Map<String, Value>,
    depth: usize,
    active: &mut BTreeSet<(&'m str, &'m str)>,
    pending: &mut Vec<Comparison<'m>>,
) -> bool {
    let Some(child_depth) = depth.checked_add(1) else {
        return false;
    };
    if !active.insert(pair) {
        return true;
    }
    pending.push(Comparison::Exit(pair));
    left.iter().all(|(name, member)| {
        queue_structural_member(model, right, name, member, child_depth, pending)
    })
}

fn queue_structural_member<'m>(
    model: &Model<'m>,
    right: &'m Map<String, Value>,
    name: &str,
    left_member: &'m Value,
    depth: usize,
    pending: &mut Vec<Comparison<'m>>,
) -> bool {
    let Some((left, right)) = right
        .get(name)
        .and_then(|right_member| member_pair(model, left_member, right_member))
    else {
        return false;
    };
    pending.push(Comparison::Compare(left, right, depth));
    true
}
fn structure_members<'a>(left: &'a Value, right: &'a Value) -> Option<StructureMembers<'a>> {
    Some((
        left.get("members")?.as_object()?,
        right.get("members")?.as_object()?,
    ))
}

fn member_pair<'m>(model: &Model<'m>, left: &Value, right: &Value) -> Option<ShapePair<'m>> {
    Some((
        shape_target(model, left, "target")?,
        shape_target(model, right, "target")?,
    ))
}

fn shape_target<'m>(model: &Model<'m>, shape: &Value, member: &str) -> Option<ShapeRef<'m>> {
    shape
        .get(member)
        .and_then(model::target)
        .and_then(|id| model.shape(id))
}

fn schedule_pair<'m>(
    pair: (&'m str, &'m str),
    children: impl IntoIterator<Item = ShapePair<'m>>,
    depth: usize,
    active: &mut BTreeSet<(&'m str, &'m str)>,
    pending: &mut Vec<Comparison<'m>>,
) -> bool {
    let Some(child_depth) = depth.checked_add(1) else {
        return false;
    };
    if !active.insert(pair) {
        return true;
    }
    pending.push(Comparison::Exit(pair));
    children
        .into_iter()
        .for_each(|(left, right)| pending.push(Comparison::Compare(left, right, child_depth)));
    true
}
