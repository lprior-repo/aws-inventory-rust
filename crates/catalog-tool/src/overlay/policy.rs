use super::{
    Binding, FixedValue, Identity, NameError, OperationName, OperationOverlay, OverlayClass,
    Paginator,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate(
    operations: &BTreeMap<OperationName, OperationOverlay>,
) -> Result<(), NameError> {
    operations
        .iter()
        .try_for_each(|(name, plan)| operation(name, plan))?;
    let stages = operations
        .iter()
        .flat_map(|(name, plan)| {
            branches(plan).map(move |branch| {
                (
                    (name, branch.map(|branch| branch.discriminator.as_str())),
                    (plan, branch),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    stages.iter().try_for_each(|(stage, (plan, branch))| {
        stage_bindings(plan, *branch).try_for_each(|binding| {
            binding_parents(binding)?
                .try_for_each(|parent| validate_parent(*stage, parent, &stages))
        })
    })?;
    let remaining = stages.keys().copied().collect::<BTreeSet<_>>();
    let remaining = (0..stages.len()).try_fold(remaining, |mut remaining, _| {
        let ready = remaining.iter().copied().find(|stage| {
            stages.get(stage).is_some_and(|(plan, branch)| {
                stage_bindings(plan, *branch).all(|binding| {
                    binding_parents(binding).is_ok_and(|mut parents| {
                        parents.all(|parent| {
                            !remaining.iter().any(|stage| matches_parent(*stage, parent))
                        })
                    })
                })
            })
        });
        match ready {
            Some(name) => {
                remaining.remove(&name);
                Ok(remaining)
            }
            None => Err(invalid("bind", "cycle")),
        }
    })?;
    if !remaining.is_empty() {
        return Err(invalid("bind", "cycle"));
    }
    Ok(())
}

pub(in crate::overlay) fn effective_bindings(
    plan: &OperationOverlay,
) -> impl Iterator<Item = &Binding> {
    branches(plan).flat_map(move |branch| stage_bindings(plan, branch))
}

fn branches(plan: &OperationOverlay) -> impl Iterator<Item = Option<&super::ExecutionBranch>> {
    plan.branches
        .iter()
        .map(Some)
        .chain(std::iter::once(None).filter(|_| plan.branches.is_empty()))
}

fn stage_bindings<'a>(
    plan: &'a OperationOverlay,
    branch: Option<&'a super::ExecutionBranch>,
) -> impl Iterator<Item = &'a Binding> {
    plan.bind
        .iter()
        .filter(move |(path, _)| {
            !branch.is_some_and(|branch| {
                branch.fixed.contains_key(*path) || branch.bind.contains_key(*path)
            })
        })
        .map(|(_, binding)| binding)
        .chain(branch.into_iter().flat_map(|branch| branch.bind.values()))
}

type Stage<'a> = (&'a OperationName, Option<&'a str>);

fn matches_parent(stage: Stage<'_>, parent: Stage<'_>) -> bool {
    stage.0 == parent.0 && parent.1.is_none_or(|branch| stage.1 == Some(branch))
}

fn validate_parent(
    stage: Stage<'_>,
    parent: Stage<'_>,
    stages: &BTreeMap<Stage<'_>, (&OperationOverlay, Option<&super::ExecutionBranch>)>,
) -> Result<(), NameError> {
    if matches_parent(stage, parent) {
        return Err(invalid("operation", "self reference"));
    }
    if parent.1.is_some() && !stages.contains_key(&parent) {
        return Err(invalid("bind.from_branch", "unknown branch"));
    }
    Ok(())
}

fn operation(name: &OperationName, plan: &OperationOverlay) -> Result<(), NameError> {
    disposition(plan)?;
    bounded(plan.exclude_from_hash.len(), 0, 64, "exclude_from_hash")?;
    bounded(plan.skip.len(), 0, 64, "skip")?;
    super::skip::validate(plan)?;
    bounded(plan.queryable.len(), 0, 64, "queryable")?;
    if plan
        .page_size
        .is_some_and(|size| !(1..=100_000).contains(&size))
    {
        return Err(invalid("page_size", "outside 1..=100000"));
    }
    if plan.quota_group.as_ref().is_some_and(|group| {
        group.is_empty()
            || group.len() > 32
            || !group
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_'))
    }) {
        return Err(invalid("quota_group", "invalid quota group"));
    }
    if let Some(identity) = &plan.identity {
        identity_rule(identity)?;
    }
    if let Some(paginator) = &plan.paginator {
        paginator_rule(paginator)?;
    }
    plan.fixed.values().try_for_each(fixed_rule)?;
    if plan.skip.iter().any(|skipped| skipped == name)
        || plan
            .batch
            .as_ref()
            .is_some_and(|batch| &batch.replaces == name)
    {
        return Err(invalid("operation", "self reference"));
    }
    if plan
        .batch
        .as_ref()
        .is_some_and(|batch| !(1..=10_000).contains(&batch.max))
    {
        return Err(invalid("batch.max", "outside 1..=10000"));
    }
    Ok(())
}

fn disposition(plan: &OperationOverlay) -> Result<(), NameError> {
    match (plan.class, plan.defer_reason.as_deref()) {
        (Some(OverlayClass::Defer), Some(reason))
            if !reason.trim().is_empty() && !reason.chars().any(char::is_control) => {}
        (Some(OverlayClass::Defer), _) => {
            return Err(invalid("defer_reason", "missing or invalid reason"));
        }
        (_, Some(_)) => {
            return Err(invalid(
                "defer_reason",
                "only deferred operations carry a reason",
            ));
        }
        _ => {}
    }
    if matches!(
        plan.class,
        Some(OverlayClass::Deny(_) | OverlayClass::Defer)
    ) && (plan.scope.is_some()
        || plan.items.is_some()
        || plan.page_size.is_some()
        || !plan.exclude_from_hash.is_empty()
        || !plan.skip.is_empty()
        || plan.quota_group.is_some()
        || !plan.queryable.is_empty()
        || plan.tags.is_some()
        || !plan.fixed.is_empty()
        || plan.paginator.is_some()
        || plan.identity.is_some()
        || !plan.bind.is_empty()
        || plan.batch.is_some())
    {
        return Err(invalid("class", "noncallable decision contains a plan"));
    }
    Ok(())
}

fn identity_rule(identity: &Identity) -> Result<(), NameError> {
    match identity {
        Identity::Key(paths) => {
            bounded(paths.len(), 1, 8, "identity.key")?;
            let unique = paths.iter().collect::<BTreeSet<_>>();
            if unique.len() != paths.len() {
                return Err(invalid("identity.key", "duplicate path"));
            }
            Ok(())
        }
        Identity::Template(template) => template_rule(template),
        Identity::Arn(_) => Ok(()),
        Identity::Choice(choice) => choice.validate(),
        Identity::CorrelatedKey(fields) => super::selection::validate_fields(fields),
        Identity::MapArn {
            key,
            resource_kind,
            request_url,
            ..
        } => {
            if key.is_empty()
                || key.len() > 4096
                || key.chars().any(char::is_control)
                || matches!(resource_kind, super::ArnResourceKind::SqsQueue)
                    != request_url.is_some()
            {
                Err(invalid(
                    "identity.map_arn",
                    "invalid key or request correlation",
                ))
            } else {
                Ok(())
            }
        }
    }
}

fn template_rule(template: &str) -> Result<(), NameError> {
    let (open, seen) = template.char_indices().try_fold(
        (None, false),
        |(open, seen), (index, character)| match (open, character) {
            (None, '{') => Ok((Some(index.saturating_add(1)), seen)),
            (Some(start), '}') => {
                let field = template
                    .get(start..index)
                    .ok_or_else(|| invalid("identity.template", template))?;
                if !matches!(field, "partition" | "region" | "account") {
                    super::MemberPath::selected(field)?;
                }
                Ok((None, true))
            }
            (Some(_), '{') | (None, '}') => Err(invalid("identity.template", template)),
            _ => Ok((open, seen)),
        },
    )?;
    if open.is_some() || !seen {
        return Err(invalid("identity.template", template));
    }
    Ok(())
}

fn paginator_rule(paginator: &Paginator) -> Result<(), NameError> {
    let Paginator::Tokens { pairs, .. } = paginator else {
        return Ok(());
    };
    bounded(pairs.len(), 1, 8, "paginator.pairs")?;
    let inputs = pairs
        .iter()
        .map(|pair| &pair.input)
        .collect::<BTreeSet<_>>();
    if inputs.len() != pairs.len() {
        return Err(invalid("paginator.pairs", "duplicate input"));
    }
    Ok(())
}

fn fixed_rule(value: &FixedValue) -> Result<(), NameError> {
    if super::fixed_input::bounded(value) {
        Ok(())
    } else {
        Err(invalid("fixed", "literal bounds exceeded"))
    }
}

fn bounded(count: usize, min: usize, max: usize, key: &str) -> Result<(), NameError> {
    if !(min..=max).contains(&count) {
        return Err(invalid(key, &count.to_string()));
    }
    Ok(())
}

fn invalid(key: &str, value: &str) -> NameError {
    NameError {
        value: format!("{key}: {value}"),
    }
}

fn binding_parents(binding: &Binding) -> Result<impl Iterator<Item = Stage<'_>>, NameError> {
    let binding = match binding {
        Binding::Transformed {
            binding,
            transform: super::BindingTransform::StripPrefix(prefix),
        } => {
            if prefix.is_empty()
                || prefix.len() > 256
                || prefix.chars().any(char::is_control)
                || !matches!(
                    binding.as_ref(),
                    Binding::From { .. }
                        | Binding::FromArn { .. }
                        | Binding::FromService { .. }
                        | Binding::Context { .. }
                )
            {
                return Err(invalid("bind.transform", "invalid source or prefix"));
            }
            binding.as_ref()
        }
        binding => binding,
    };
    let (parent, alternatives) = match binding {
        Binding::From { operation, .. }
        | Binding::FromArn { operation, .. }
        | Binding::FromUrl { operation, .. }
        | Binding::Context { operation, .. } => (Some((operation, None)), None),
        Binding::FromBranch {
            operation, branch, ..
        }
        | Binding::ContextFromBranch {
            operation, branch, ..
        } => (Some((operation, Some(branch.as_str()))), None),
        Binding::FromAny(alternatives) => (None, Some(alternatives.as_slice())),
        Binding::FromService { service, .. } if service.is_empty() || service.len() > 64 => {
            return Err(invalid("bind.service", service));
        }
        Binding::FromService { .. } | Binding::Account | Binding::Region | Binding::RegionList => {
            (None, None)
        }
        Binding::Transformed { .. } => return Err(invalid("bind.transform", "nested transform")),
    };
    Ok(parent.into_iter().chain(
        alternatives
            .into_iter()
            .flatten()
            .map(|alternative| (&alternative.operation, None)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay::{
        BindingAlternative, ContextSource, ExecutionBranch, MemberPath, Scope, ServiceOverlay,
    };

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn operation_with(binding: Binding) -> Result<OperationOverlay, NameError> {
        Ok(OperationOverlay {
            bind: BTreeMap::from([(MemberPath::new("Id")?, binding)]),
            ..OperationOverlay::default()
        })
    }

    fn alternatives(names: &[&str]) -> Result<Binding, NameError> {
        Ok(Binding::FromAny(
            names
                .iter()
                .map(|name| {
                    Ok(BindingAlternative {
                        operation: OperationName::new(name)?,
                        source: ContextSource::SelectedItem,
                        path: MemberPath::new("Id")?,
                    })
                })
                .collect::<Result<_, NameError>>()?,
        ))
    }

    fn branch(
        bind: BTreeMap<MemberPath, Binding>,
        fixed: BTreeMap<MemberPath, FixedValue>,
    ) -> ExecutionBranch {
        ExecutionBranch {
            discriminator: "detail".into(),
            bind,
            fixed,
            emission: None,
            row_policy: None,
            scope: Scope::Region,
            request_region: None,
        }
    }

    fn assert_branch_only_self_reference_rejected() -> TestResult {
        let bindings = [
            alternatives(&["B", "A"])?,
            Binding::FromBranch {
                operation: OperationName::new("A")?,
                branch: "detail".into(),
                path: MemberPath::new("Id")?,
            },
            Binding::ContextFromBranch {
                operation: OperationName::new("A")?,
                branch: "detail".into(),
                source: ContextSource::Request,
                path: MemberPath::new("Id")?,
            },
            Binding::Transformed {
                binding: Box::new(Binding::From {
                    operation: OperationName::new("A")?,
                    path: MemberPath::new("Id")?,
                }),
                transform: crate::overlay::BindingTransform::StripPrefix("prefix".into()),
            },
        ];
        for binding in bindings {
            let a = OperationOverlay {
                branches: vec![branch(
                    BTreeMap::from([(MemberPath::new("Id")?, binding)]),
                    BTreeMap::new(),
                )],
                ..OperationOverlay::default()
            };
            assert_eq!(
                ServiceOverlay::new("example", vec![("A", a)])
                    .map(|overlay| overlay.operations().len()),
                Err(invalid("operation", "self reference"))
            );
        }
        Ok(())
    }

    fn assert_mixed_branch_common_cycle_detected() -> TestResult {
        let a = OperationOverlay {
            branches: vec![branch(
                BTreeMap::from([(MemberPath::new("Id")?, alternatives(&["C", "B"])?)]),
                BTreeMap::new(),
            )],
            ..OperationOverlay::default()
        };
        let b = operation_with(Binding::From {
            operation: OperationName::new("A")?,
            path: MemberPath::new("Id")?,
        })?;
        assert_eq!(
            ServiceOverlay::new(
                "example",
                vec![("A", a), ("B", b), ("C", OperationOverlay::default())]
            )
            .map(|overlay| overlay.operations().len()),
            Err(invalid("bind", "cycle"))
        );
        Ok(())
    }

    #[test]
    fn branch_only_dependencies_reject_self_references_and_mixed_cycles() -> TestResult {
        assert_branch_only_self_reference_rejected()?;
        assert_mixed_branch_common_cycle_detected()?;
        Ok(())
    }

    #[test]
    fn every_branch_override_removes_only_its_common_destination_dependency() -> TestResult {
        for parent in ["A", "B"] {
            for fixed in [false, true] {
                let mut a = operation_with(alternatives(&[parent])?)?;
                let path = MemberPath::new("Id")?;
                a.branches = vec![if fixed {
                    branch(
                        BTreeMap::new(),
                        BTreeMap::from([(path, FixedValue::String("literal".into()))]),
                    )
                } else {
                    branch(BTreeMap::from([(path, Binding::Region)]), BTreeMap::new())
                }];
                let b = operation_with(Binding::From {
                    operation: OperationName::new("A")?,
                    path: MemberPath::new("Id")?,
                })?;
                assert_eq!(
                    ServiceOverlay::new("example", vec![("A", a.clone()), ("B", b.clone())])?
                        .operations()
                        .len(),
                    2
                );
                let mut inherited = branch(BTreeMap::new(), BTreeMap::new());
                inherited.discriminator = "inherited".into();
                a.branches.push(inherited);
                let error = if parent == "A" {
                    invalid("operation", "self reference")
                } else {
                    invalid("bind", "cycle")
                };
                assert_eq!(
                    ServiceOverlay::new("example", vec![("A", a), ("B", b)])
                        .map(|overlay| overlay.operations().len()),
                    Err(error)
                );
            }
        }
        Ok(())
    }

    #[test]
    fn alternative_dependencies_detect_cycle_through_nonprimary_producer() -> TestResult {
        let a = operation_with(alternatives(&["B", "C"])?)?;
        let c = operation_with(Binding::From {
            operation: OperationName::new("A")?,
            path: MemberPath::new("Id")?,
        })?;
        let result = ServiceOverlay::new(
            "example",
            vec![("A", a), ("B", OperationOverlay::default()), ("C", c)],
        );
        assert_eq!(
            result.map(|overlay| overlay.operations().len()),
            Err(invalid("bind", "cycle"))
        );
        Ok(())
    }

    #[test]
    fn branch_and_alternative_bindings_reject_self_reference() -> TestResult {
        let bindings = [
            Binding::FromBranch {
                operation: OperationName::new("A")?,
                branch: "detail".into(),
                path: MemberPath::new("Id")?,
            },
            Binding::ContextFromBranch {
                operation: OperationName::new("A")?,
                branch: "detail".into(),
                source: ContextSource::Request,
                path: MemberPath::new("Id")?,
            },
            alternatives(&["B", "A"])?,
        ];
        for binding in bindings {
            let mut a = operation_with(binding)?;
            a.branches.push(branch(BTreeMap::new(), BTreeMap::new()));
            let result = ServiceOverlay::new("example", vec![("A", a)]);
            assert_eq!(
                result.map(|overlay| overlay.operations().len()),
                Err(invalid("operation", "self reference"))
            );
        }
        Ok(())
    }

    #[test]
    fn region_list_and_acyclic_alternatives_keep_all_operations_callable() -> TestResult {
        let operations = vec![
            ("A", operation_with(alternatives(&["B", "C"])?)?),
            ("B", operation_with(Binding::RegionList)?),
            ("C", OperationOverlay::default()),
        ];
        let result = ServiceOverlay::new("example", operations)?;
        assert_eq!(result.operations().len(), 3);
        Ok(())
    }

    fn staged_reference(
        operation: &str,
        target: &str,
        context: bool,
    ) -> Result<Binding, NameError> {
        let operation = OperationName::new(operation)?;
        let path = MemberPath::new("Id")?;
        Ok(if context {
            Binding::ContextFromBranch {
                operation,
                branch: target.into(),
                source: ContextSource::Request,
                path,
            }
        } else {
            Binding::FromBranch {
                operation,
                branch: target.into(),
                path,
            }
        })
    }

    fn staged_plan(binding: Binding) -> Result<OperationOverlay, NameError> {
        let path = MemberPath::new("Id")?;
        let mut seed = branch(
            BTreeMap::new(),
            BTreeMap::from([(path.clone(), FixedValue::String("seed".into()))]),
        );
        seed.discriminator = "seed".into();
        Ok(OperationOverlay {
            branches: vec![
                seed,
                branch(BTreeMap::from([(path, binding)]), BTreeMap::new()),
            ],
            ..OperationOverlay::default()
        })
    }

    #[test]
    fn explicit_stage_dags_allow_same_and_cross_operation_dependencies() -> TestResult {
        for context in [false, true] {
            let a = staged_plan(staged_reference("A", "seed", context)?)?;
            assert_eq!(
                ServiceOverlay::new("example", vec![("A", a)])?
                    .operations()
                    .len(),
                1
            );
            let a = staged_plan(Binding::From {
                operation: OperationName::new("B")?,
                path: MemberPath::new("Id")?,
            })?;
            let b = operation_with(staged_reference("A", "seed", context)?)?;
            assert_eq!(
                ServiceOverlay::new("example", vec![("A", a), ("B", b)])?
                    .operations()
                    .len(),
                2
            );
        }
        Ok(())
    }

    #[test]
    fn explicit_stage_self_cycles_missing_targets_and_real_cycles_are_rejected() -> TestResult {
        for context in [false, true] {
            let a = staged_plan(staged_reference("A", "detail", context)?)?;
            assert_eq!(
                ServiceOverlay::new("example", vec![("A", a)])
                    .map(|overlay| overlay.operations().len()),
                Err(invalid("operation", "self reference"))
            );
            let a = staged_plan(staged_reference("A", "missing", context)?)?;
            assert_eq!(
                ServiceOverlay::new("example", vec![("A", a)])
                    .map(|overlay| overlay.operations().len()),
                Err(invalid("bind.from_branch", "unknown branch"))
            );
            let mut a = staged_plan(staged_reference("A", "seed", context)?)?;
            let seed = a.branches.first_mut().ok_or("seed absent")?;
            seed.fixed.clear();
            seed.bind.insert(
                MemberPath::new("Id")?,
                staged_reference("A", "detail", context)?,
            );
            assert_eq!(
                ServiceOverlay::new("example", vec![("A", a)])
                    .map(|overlay| overlay.operations().len()),
                Err(invalid("bind", "cycle"))
            );
        }
        Ok(())
    }
}
