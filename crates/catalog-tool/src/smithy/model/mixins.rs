mod size;
use super::{Member, ModelFile, Shape, ShapeKind};
use size::{reserve, shape_size, size_error};
use std::collections::{BTreeMap, BTreeSet};
const MAX_MIXIN_DEPTH: usize = 128;
const MAX_MIXIN_STEPS: usize = 1_000_000;
struct Resolved {
    traits: BTreeMap<String, serde_json::Value>,
    members: BTreeMap<String, Member>,
    kind: std::mem::Discriminant<ShapeKind>,
    bytes: usize,
}

pub(in crate::smithy) fn expand(mut model: ModelFile) -> Result<ModelFile, String> {
    if model
        .shapes
        .values()
        .all(|shape| shape.mixins.is_empty() && !shape.traits.contains_key("smithy.api#mixin"))
    {
        return Ok(model);
    }
    let mut resolved = BTreeMap::new();
    let mut steps = 0;
    let mut bytes = 0;
    for id in model.shapes.keys() {
        resolve(id, &model, &mut resolved, &mut steps, &mut bytes)?;
    }
    install_all(&mut model, &resolved, &mut bytes)?;
    Ok(model)
}

fn install_all(
    model: &mut ModelFile,
    resolved: &BTreeMap<String, Resolved>,
    budget: &mut usize,
) -> Result<(), String> {
    for (id, shape) in &mut model.shapes {
        let value = resolved
            .get(id)
            .ok_or_else(|| format!("missing resolution for {id}"))?;
        reserve(budget, value.bytes)?;
        install(id, shape, value)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "mixins/tests.rs"]
mod tests;
fn resolve(
    root: &str,
    model: &ModelFile,
    resolved: &mut BTreeMap<String, Resolved>,
    steps: &mut usize,
    bytes: &mut usize,
) -> Result<(), String> {
    let mut stack = vec![(root.to_owned(), false, 0usize)];
    let mut active = BTreeSet::new();
    while let Some(node) = stack.pop() {
        visit(node, model, resolved, steps, bytes, &mut stack, &mut active)?;
    }
    Ok(())
}

fn visit(
    (id, done, depth): (String, bool, usize),
    model: &ModelFile,
    resolved: &mut BTreeMap<String, Resolved>,
    steps: &mut usize,
    bytes: &mut usize,
    stack: &mut Vec<(String, bool, usize)>,
    active: &mut BTreeSet<String>,
) -> Result<(), String> {
    check_depth(depth, &id)?;
    if resolved.contains_key(&id) {
        return Ok(());
    }
    count_step(steps)?;
    let shape = source_shape(model, &id)?;
    if done {
        let value = resolved_shape(&id, shape, model, resolved, bytes)?;
        resolved.insert(id.clone(), value);
        active.remove(&id);
    } else {
        schedule(&id, shape, depth, stack, active, steps)?;
    }
    Ok(())
}

fn source_shape<'a>(model: &'a ModelFile, id: &str) -> Result<&'a Shape, String> {
    model
        .shapes
        .get(id)
        .ok_or_else(|| format!("mixin target is missing: {id}"))
}

fn check_depth(depth: usize, id: &str) -> Result<(), String> {
    if depth > MAX_MIXIN_DEPTH {
        return Err(format!("mixin depth exceeds {MAX_MIXIN_DEPTH}: {id}"));
    }
    Ok(())
}

fn count_step(steps: &mut usize) -> Result<(), String> {
    *steps = steps
        .checked_add(1)
        .filter(|count| *count <= MAX_MIXIN_STEPS)
        .ok_or_else(|| "mixin traversal exceeds 1000000 steps".to_owned())?;
    Ok(())
}

fn schedule(
    id: &str,
    shape: &Shape,
    depth: usize,
    stack: &mut Vec<(String, bool, usize)>,
    active: &mut BTreeSet<String>,
    steps: &mut usize,
) -> Result<(), String> {
    if !active.insert(id.to_owned()) {
        return Err(format!("mixin cycle includes {id}"));
    }
    stack.push((id.to_owned(), true, depth));
    for mixin in shape.mixins.iter().rev() {
        count_step(steps)?;
        if active.contains(&mixin.target) {
            return Err(format!("mixin cycle includes {}", mixin.target));
        }
        stack.push((mixin.target.clone(), false, depth + 1));
    }
    Ok(())
}

fn resolved_shape(
    id: &str,
    shape: &Shape,
    model: &ModelFile,
    resolved: &BTreeMap<String, Resolved>,
    budget: &mut usize,
) -> Result<Resolved, String> {
    let base = shape_size(shape)?.checked_mul(2).ok_or_else(size_error)?;
    reserve(budget, base)?;
    let mut output = Resolved {
        traits: BTreeMap::new(),
        members: BTreeMap::new(),
        kind: std::mem::discriminant(&shape.kind),
        bytes: base,
    };
    for mixin in &shape.mixins {
        inherit(id, mixin, shape, model, resolved, budget, &mut output)?;
    }
    merge_local(id, shape, &mut output)?;
    Ok(output)
}

fn inherit(
    id: &str,
    mixin: &super::Target,
    shape: &Shape,
    model: &ModelFile,
    resolved: &BTreeMap<String, Resolved>,
    budget: &mut usize,
    output: &mut Resolved,
) -> Result<(), String> {
    let source_shape = source_shape(model, &mixin.target)?;
    validate_mixin(id, shape, source_shape, &mixin.target)?;
    let source = resolved
        .get(&mixin.target)
        .ok_or_else(|| format!("mixin target is missing: {}", mixin.target))?;
    reserve(budget, source.bytes)?;
    output.bytes = output
        .bytes
        .checked_add(source.bytes)
        .ok_or_else(size_error)?;
    merge_members(&mut output.members, &source.members)?;
    output.traits.extend(source.traits.clone());
    Ok(())
}

fn validate_mixin(id: &str, shape: &Shape, source: &Shape, target: &str) -> Result<(), String> {
    if !source.traits.contains_key("smithy.api#mixin") {
        return Err(format!("mixin target is not a mixin: {target}"));
    }
    if std::mem::discriminant(&shape.kind) != std::mem::discriminant(&source.kind) {
        return Err(format!("mixin shape kind differs: {id}"));
    }
    Ok(())
}

fn merge_local(id: &str, shape: &Shape, output: &mut Resolved) -> Result<(), String> {
    merge_members(&mut output.members, &members(shape))?;
    output.traits.extend(shape.traits.clone());
    exclude_local_traits(shape, &mut output.traits)?;
    if output.members.len() > MAX_MIXIN_STEPS {
        return Err(format!(
            "mixin member expansion exceeds {MAX_MIXIN_STEPS}: {id}"
        ));
    }
    Ok(())
}

fn members(shape: &Shape) -> BTreeMap<String, Member> {
    match &shape.kind {
        ShapeKind::Structure { members } | ShapeKind::Union { members } => members.clone(),
        _ => BTreeMap::new(),
    }
}

fn merge_members(
    target: &mut BTreeMap<String, Member>,
    source: &BTreeMap<String, Member>,
) -> Result<(), String> {
    source
        .iter()
        .try_for_each(|(name, member)| match target.get_mut(name) {
            Some(previous) => merge_member(name, previous, member),
            None => {
                target.insert(name.clone(), member.clone());
                Ok(())
            }
        })
}

fn merge_member(name: &str, target: &mut Member, source: &Member) -> Result<(), String> {
    if target.target != source.target {
        return Err(format!("conflicting mixin member targets: {name}"));
    }
    source.traits.iter().for_each(|(key, value)| {
        target.traits.insert(key.clone(), value.clone());
    });
    Ok(())
}

fn exclude_local_traits(
    shape: &Shape,
    traits: &mut BTreeMap<String, serde_json::Value>,
) -> Result<(), String> {
    let Some(value) = shape.traits.get("smithy.api#mixin") else {
        return Ok(());
    };
    traits.remove("smithy.api#mixin");
    let mixin = value
        .as_object()
        .ok_or_else(|| "mixin trait must be an object".to_owned())?;
    let Some(local) = mixin.get("localTraits") else {
        return Ok(());
    };
    let values = local
        .as_array()
        .ok_or_else(|| "mixin localTraits must be an array".to_owned())?;
    values.iter().try_for_each(|value| {
        let name = value
            .as_str()
            .ok_or_else(|| "mixin localTraits entries must be strings".to_owned())?;
        traits.remove(name);
        Ok(())
    })
}

fn install(id: &str, shape: &mut Shape, resolved: &Resolved) -> Result<(), String> {
    if std::mem::discriminant(&shape.kind) != resolved.kind {
        return Err(format!("mixin shape kind differs: {id}"));
    }
    let direct = std::mem::take(&mut shape.traits);
    shape.traits = resolved.traits.clone();
    shape.traits.extend(direct);
    match &mut shape.kind {
        ShapeKind::Structure { members } | ShapeKind::Union { members } => {
            *members = resolved.members.clone()
        }
        _ if !resolved.members.is_empty() => {
            return Err(format!("mixin members target non-aggregate shape: {id}"));
        }
        _ => {}
    }
    shape.mixins.clear();
    Ok(())
}
