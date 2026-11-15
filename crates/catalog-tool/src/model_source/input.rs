use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Clone, Copy)]
pub(crate) struct InputSpec<'a> {
    pub service: &'a str,
    pub operation: &'a str,
    pub target: Option<&'a str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputFailure {
    MissingShape,
    NonStructure,
    DepthLimit,
    MixinCycle,
}

#[derive(Debug, PartialEq, Eq, Error)]
#[error("{service}/{operation}: input {target} is invalid ({kind:?})")]
pub struct InputError {
    pub service: String,
    pub operation: String,
    pub target: String,
    pub kind: InputFailure,
}

pub(crate) trait InputModel {
    fn structure(&self, target: &str) -> Option<bool>;
    fn mixins<'a>(&'a self, target: &str, visit: &mut dyn FnMut(&'a str));
    fn members<'a>(&'a self, target: &str, visit: &mut dyn FnMut(&'a str, bool));
}

enum VisitState<'a> {
    Active,
    Complete(BTreeMap<&'a str, bool>),
}

type Memo<'a> = BTreeMap<&'a str, VisitState<'a>>;

pub(crate) fn resolve_input<'a>(
    model: &'a impl InputModel,
    spec: InputSpec<'a>,
) -> Result<bool, InputError> {
    let Some(target) = spec.target.filter(|target| *target != "smithy.api#Unit") else {
        return Ok(false);
    };
    let mut memo = Memo::new();
    let mut members = BTreeMap::new();
    resolve_structure(model, spec, target, 0, &mut memo, &mut members)?;
    Ok(members.values().any(|required| *required))
}

fn resolve_structure<'a>(
    model: &'a impl InputModel,
    spec: InputSpec<'a>,
    target: &'a str,
    depth: usize,
    memo: &mut Memo<'a>,
    output: &mut BTreeMap<&'a str, bool>,
) -> Result<(), InputError> {
    check_depth(spec, target, depth)?;
    if merge_if_complete(target, spec, memo, output)? {
        return Ok(());
    }
    check_structure(model, spec, target)?;
    memo.insert(target, VisitState::Active);
    let mut resolved = BTreeMap::new();
    resolve_mixins(model, spec, target, depth, memo, &mut resolved)?;
    model.members(target, &mut |name, required| {
        merge_member(&mut resolved, name, required)
    });
    finish_structure(target, spec, memo, output, resolved)
}

fn finish_structure<'a>(
    target: &'a str,
    spec: InputSpec<'a>,
    memo: &mut Memo<'a>,
    output: &mut BTreeMap<&'a str, bool>,
    resolved: BTreeMap<&'a str, bool>,
) -> Result<(), InputError> {
    let entry = memo
        .entry(target)
        .insert_entry(VisitState::Complete(resolved));
    match entry.get() {
        VisitState::Complete(members) => {
            merge_members(output, members);
            Ok(())
        }
        VisitState::Active => Err(input_error(spec, target, InputFailure::MixinCycle)),
    }
}

fn merge_if_complete<'a>(
    target: &'a str,
    spec: InputSpec<'a>,
    memo: &Memo<'a>,
    output: &mut BTreeMap<&'a str, bool>,
) -> Result<bool, InputError> {
    match memo.get(target) {
        Some(VisitState::Active) => Err(input_error(spec, target, InputFailure::MixinCycle)),
        Some(VisitState::Complete(members)) => {
            merge_members(output, members);
            Ok(true)
        }
        None => Ok(false),
    }
}

fn resolve_mixins<'a>(
    model: &'a impl InputModel,
    spec: InputSpec<'a>,
    target: &'a str,
    depth: usize,
    memo: &mut Memo<'a>,
    output: &mut BTreeMap<&'a str, bool>,
) -> Result<(), InputError> {
    let mut mixins = Vec::new();
    model.mixins(target, &mut |mixin| mixins.push(mixin));
    let next_depth = depth
        .checked_add(1)
        .ok_or_else(|| input_error(spec, target, InputFailure::DepthLimit))?;
    mixins
        .into_iter()
        .try_for_each(|mixin| resolve_structure(model, spec, mixin, next_depth, memo, output))
}

fn check_depth(spec: InputSpec<'_>, target: &str, depth: usize) -> Result<(), InputError> {
    if depth > 128 {
        return Err(input_error(spec, target, InputFailure::DepthLimit));
    }
    Ok(())
}

fn check_structure(
    model: &impl InputModel,
    spec: InputSpec<'_>,
    target: &str,
) -> Result<(), InputError> {
    match model.structure(target) {
        None => Err(input_error(spec, target, InputFailure::MissingShape)),
        Some(false) => Err(input_error(spec, target, InputFailure::NonStructure)),
        Some(true) => Ok(()),
    }
}

fn merge_member<'a>(members: &mut BTreeMap<&'a str, bool>, name: &'a str, required: bool) {
    members
        .entry(name)
        .and_modify(|inherited| *inherited |= required)
        .or_insert(required);
}

fn merge_members<'a>(output: &mut BTreeMap<&'a str, bool>, members: &BTreeMap<&'a str, bool>) {
    members
        .iter()
        .for_each(|(name, required)| merge_member(output, name, *required));
}

fn input_error(spec: InputSpec<'_>, target: &str, kind: InputFailure) -> InputError {
    InputError {
        service: spec.service.to_owned(),
        operation: spec.operation.to_owned(),
        target: target.to_owned(),
        kind,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Node {
        mixins: Vec<String>,
        required: bool,
    }

    struct Model {
        shapes: BTreeMap<String, Node>,
        member_reads: Cell<usize>,
    }

    impl InputModel for Model {
        fn structure(&self, target: &str) -> Option<bool> {
            self.shapes.get(target).map(|_| true)
        }

        fn mixins<'a>(&'a self, target: &str, visit: &mut dyn FnMut(&'a str)) {
            if let Some(node) = self.shapes.get(target) {
                node.mixins.iter().for_each(|mixin| visit(mixin));
            }
        }

        fn members<'a>(&'a self, target: &str, visit: &mut dyn FnMut(&'a str, bool)) {
            self.member_reads
                .set(self.member_reads.get().saturating_add(1));
            if let Some(node) = self.shapes.get(target) {
                if node.required {
                    visit("Required", true);
                }
            }
        }
    }

    fn node(mixins: &[&str], required: bool) -> Node {
        Node {
            mixins: mixins.iter().map(|mixin| (*mixin).to_owned()).collect(),
            required,
        }
    }

    fn spec(target: &'static str) -> InputSpec<'static> {
        InputSpec {
            service: "svc",
            operation: "op",
            target: Some(target),
        }
    }

    #[test]
    fn mixin_cycle_is_reported_as_a_typed_error() -> Result<(), String> {
        let model = Model {
            shapes: BTreeMap::from([
                ("a".to_owned(), node(&["b"], false)),
                ("b".to_owned(), node(&["a"], false)),
            ]),
            member_reads: Cell::new(0),
        };
        let error = resolve_input(&model, spec("a"))
            .err()
            .ok_or_else(|| "cycle accepted".to_owned())?;
        assert_eq!(error.kind, InputFailure::MixinCycle);
        assert_eq!(error.target, "a");
        Ok(())
    }

    fn diamond_node(level: usize) -> Node {
        match level {
            0 => node(&[], true),
            1 => node(&["n#0"], false),
            _ => Node {
                mixins: vec![format!("n#{}", level - 1), format!("n#{}", level - 2)],
                required: false,
            },
        }
    }

    fn diamond_model() -> Model {
        let shapes = (0..=30)
            .map(|level| (format!("n#{level}"), diamond_node(level)))
            .collect();
        Model {
            shapes,
            member_reads: Cell::new(0),
        }
    }
    fn chain_model(levels: usize) -> Model {
        let shapes = (0..=levels)
            .map(|level| {
                let node = if level == 0 {
                    node(&[], false)
                } else {
                    Node {
                        mixins: vec![format!("n#{}", level - 1)],
                        required: false,
                    }
                };
                (format!("n#{level}"), node)
            })
            .collect();
        Model {
            shapes,
            member_reads: Cell::new(0),
        }
    }

    #[test]
    fn mixin_depth_limit_is_enforced() -> Result<(), String> {
        let model = chain_model(130);
        let error = resolve_input(&model, spec("n#130"))
            .err()
            .ok_or_else(|| "deep chain accepted".to_owned())?;
        assert_eq!(error.kind, InputFailure::DepthLimit);
        Ok(())
    }

    #[test]
    fn diamond_mixins_resolve_each_shape_once_and_keep_required_members() -> Result<(), String> {
        let model = diamond_model();
        let required = resolve_input(&model, spec("n#30")).map_err(|error| error.to_string())?;
        assert!(required);
        assert_eq!(model.member_reads.get(), 31);
        Ok(())
    }
}
