use super::{Model, ShapeRef, required, shape_type, target};
use serde_json::{Value, map::Iter};
use std::collections::BTreeSet;

struct Frame<'a> {
    id: &'a str,
    prefix: String,
    depth: usize,
    members: Iter<'a>,
}
struct Traversal<'a, 's> {
    active: BTreeSet<&'a str>,
    paths: Vec<String>,
    stack: Vec<Frame<'a>>,
    supplied: &'s BTreeSet<String>,
}

impl<'a> Model<'a> {
    pub(crate) fn required_paths(
        &self,
        operation: &str,
        supplied: &BTreeSet<String>,
    ) -> Result<Vec<String>, &'static str> {
        let Some(root) = self.side_root(operation, "input") else {
            return Ok(Vec::new());
        };
        let mut traversal = Traversal {
            active: BTreeSet::new(),
            paths: Vec::new(),
            stack: Vec::new(),
            supplied,
        };
        push_frame(self, root, String::new(), 0, &mut traversal)?;
        walk_required(self, &mut traversal)?;
        Ok(traversal.paths)
    }
}

fn walk_required<'a>(
    model: &Model<'a>,
    traversal: &mut Traversal<'a, '_>,
) -> Result<(), &'static str> {
    while !traversal.stack.is_empty() {
        advance(model, traversal)?;
    }
    Ok(())
}

fn advance<'a>(model: &Model<'a>, traversal: &mut Traversal<'a, '_>) -> Result<(), &'static str> {
    let next = traversal.stack.last_mut().and_then(|frame| {
        frame
            .members
            .next()
            .map(|(name, member)| (name, member, frame.prefix.clone(), frame.depth))
    });
    let Some((name, member, prefix, depth)) = next else {
        if let Some(frame) = traversal.stack.pop() {
            traversal.active.remove(frame.id);
        }
        return Ok(());
    };
    visit_member(model, name, member, prefix, depth, traversal)
}
fn visit_member<'a>(
    model: &Model<'a>,
    name: &str,
    member: &'a Value,
    prefix: String,
    depth: usize,
    traversal: &mut Traversal<'a, '_>,
) -> Result<(), &'static str> {
    let shape = member_shape(model, member)?;
    let path = join_path(&prefix, name);
    let is_required = required(member);
    if is_required {
        traversal.paths.push(path.clone());
    }
    if is_required || has_supplied_descendant(traversal.supplied, &path) {
        let child_depth = depth
            .checked_add(1)
            .ok_or("required input depth overflow")?;
        push_frame(model, shape, path, child_depth, traversal)?;
    }
    Ok(())
}

fn has_supplied_descendant(supplied: &BTreeSet<String>, path: &str) -> bool {
    supplied.iter().any(|candidate| {
        candidate
            .strip_prefix(path)
            .is_some_and(|suffix| suffix.starts_with('.'))
    })
}

fn member_shape<'a>(model: &Model<'a>, member: &'a Value) -> Result<ShapeRef<'a>, &'static str> {
    let target_id = target(member).ok_or("member target is absent")?;
    model.shape(target_id).ok_or("member target does not exist")
}

fn push_frame<'a>(
    model: &Model<'a>,
    shape: ShapeRef<'a>,
    prefix: String,
    depth: usize,
    traversal: &mut Traversal<'a, '_>,
) -> Result<(), &'static str> {
    if depth > 32 || !traversal.active.insert(shape.id) {
        return Ok(());
    }
    push_resolved_frame(model, shape, prefix, depth, traversal)
}

fn push_resolved_frame<'a>(
    model: &Model<'a>,
    shape: ShapeRef<'a>,
    prefix: String,
    depth: usize,
    traversal: &mut Traversal<'a, '_>,
) -> Result<(), &'static str> {
    match structure_members(model, shape) {
        Ok(Some(members)) => traversal.stack.push(Frame {
            id: shape.id,
            prefix,
            depth,
            members,
        }),
        Ok(None) => {
            traversal.active.remove(shape.id);
        }
        Err(error) => {
            traversal.active.remove(shape.id);
            return Err(error);
        }
    }
    Ok(())
}

fn structure_members<'a>(
    model: &Model<'a>,
    shape: ShapeRef<'a>,
) -> Result<Option<Iter<'a>>, &'static str> {
    let shape = model.unwrap_aggregates(shape)?;
    let members = shape
        .value
        .get("members")
        .and_then(Value::as_object)
        .filter(|_| shape_type(shape.value) == Some("structure"));
    Ok(members.map(|members| members.iter()))
}

fn join_path(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}.{name}")
    }
}
