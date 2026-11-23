use crate::{
    overlay::{
        Binding, Identity, MemberPath, OperationName, OperationOverlay, OverlayClass,
        check::validation::{compatibility::compatible, selected_for_parent},
    },
    plans::{
        Model, Vocabulary,
        model::{ShapeRef, shape_type},
    },
};
use std::collections::{BTreeMap, BTreeSet};

type Bindings = BTreeMap<MemberPath, Binding>;

pub(super) fn derive(
    model: &Model<'_>,
    operation: &str,
    parents: &BTreeMap<String, OperationOverlay>,
    vocabulary: &Vocabulary,
) -> Result<Bindings, String> {
    let required = model
        .required_paths(operation, &BTreeSet::new())
        .map_err(str::to_owned)?;
    let mut candidates = parents
        .iter()
        .filter(|(parent, entry)| {
            parent.as_str() != operation && entry.class == Some(OverlayClass::Enumerate)
        })
        .filter_map(|(parent, entry)| {
            candidate(model, operation, parent, entry, &required, vocabulary)
        });
    let bindings = candidates
        .next()
        .ok_or("expand: no unique compatible identity source")?;
    if candidates.next().is_some() {
        return Err("expand: ambiguous identity sources".to_owned());
    }
    Ok(bindings)
}

fn candidate(
    model: &Model<'_>,
    operation: &str,
    parent: &str,
    entry: &OperationOverlay,
    required: &[String],
    vocabulary: &Vocabulary,
) -> Option<Bindings> {
    let selected = selected_for_parent(model, parent, Some(entry), vocabulary)?;
    let bindings: Bindings = required
        .iter()
        .map(|input| {
            let (_, target) = model.resolve_member(operation, "input", input).ok()??;
            let source = context(input, target)
                .or_else(|| source(model, selected, entry, input, target, parent))?;
            Some((MemberPath::new(input).ok()?, source))
        })
        .collect::<Option<_>>()?;
    bindings
        .values()
        .any(|binding| matches!(binding, Binding::From { .. }))
        .then_some(bindings)
}

fn context(input: &str, target: ShapeRef<'_>) -> Option<Binding> {
    if shape_type(target.value) != Some("string") {
        return None;
    }
    match input {
        "AccountId" => Some(Binding::Account),
        "Region" => Some(Binding::Region),
        _ => None,
    }
}

fn source(
    model: &Model<'_>,
    selected: ShapeRef<'_>,
    entry: &OperationOverlay,
    input: &str,
    target: ShapeRef<'_>,
    parent: &str,
) -> Option<Binding> {
    let paths: &[MemberPath] = match entry.identity.as_ref()? {
        Identity::Arn(path) => std::slice::from_ref(path),
        Identity::Key(paths) => paths,
        _ => return None,
    };
    let path = paths.iter().find(|path| path.as_str() == input)?;
    let source = model.resolve_path_from(selected, path.as_str()).ok()?;
    if !compatible(model, &source, &target) {
        return None;
    }
    Some(Binding::From {
        operation: OperationName::new(parent).ok()?,
        path: path.clone(),
    })
}
