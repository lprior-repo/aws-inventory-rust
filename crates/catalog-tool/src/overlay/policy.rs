use super::{Binding, FixedValue, Identity, NameError, OperationName, OperationOverlay, OverlayClass, Paginator};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate(
    operations: &BTreeMap<OperationName, OperationOverlay>,
) -> Result<(), NameError> {
    operations.iter().try_for_each(|(name, plan)| operation(name, plan))?;
    let remaining = operations.keys().collect::<BTreeSet<_>>();
    let remaining = (0..operations.len()).try_fold(remaining, |mut remaining, _| {
        let ready = remaining.iter().copied().find(|name| {
            operations.get(*name).is_some_and(|plan| plan.bind.values().all(|binding| {
                binding_parent(binding).is_ok_and(|parent| parent.is_none_or(|operation| !remaining.contains(operation)))
            }))
        });
        match ready {
            Some(name) => { remaining.remove(name); Ok(remaining) }
            None => Err(invalid("bind", "cycle")),
        }
    })?;
    if !remaining.is_empty() { return Err(invalid("bind", "cycle")); }
    Ok(())
}

fn operation(name: &OperationName, plan: &OperationOverlay) -> Result<(), NameError> {
    disposition(plan)?;
    bounded(plan.exclude_from_hash.len(), 0, 64, "exclude_from_hash")?;
    bounded(plan.skip.len(), 0, 64, "skip")?;
    bounded(plan.queryable.len(), 0, 64, "queryable")?;
    if plan.page_size.is_some_and(|size| !(1..=100_000).contains(&size)) {
        return Err(invalid("page_size", "outside 1..=100000"));
    }
    if plan.quota_group.as_ref().is_some_and(|group| group.is_empty() || group.len() > 32
        || !group.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'-' | b'_'))) {
        return Err(invalid("quota_group", "invalid quota group"));
    }
    if let Some(identity) = &plan.identity { identity_rule(identity)?; }
    if let Some(paginator) = &plan.paginator { paginator_rule(paginator)?; }
    plan.fixed.values().try_for_each(fixed_rule)?;
    if plan.skip.iter().any(|skipped| skipped == name)
        || plan.bind.values().any(|binding| binding_parent(binding).map_or(true, |parent| parent.is_some_and(|operation| operation == name)))
        || plan.batch.as_ref().is_some_and(|batch| &batch.replaces == name) {
        return Err(invalid("operation", "self reference"));
    }
    if plan.batch.as_ref().is_some_and(|batch| !(1..=10_000).contains(&batch.max)) {
        return Err(invalid("batch.max", "outside 1..=10000"));
    }
    Ok(())
}

fn disposition(plan: &OperationOverlay) -> Result<(), NameError> {
    match (plan.class, plan.defer_reason.as_deref()) {
        (Some(OverlayClass::Defer), Some(reason)) if !reason.trim().is_empty()
            && !reason.chars().any(char::is_control) => {}
        (Some(OverlayClass::Defer), _) => return Err(invalid("defer_reason", "missing or invalid reason")),
        (_, Some(_)) => return Err(invalid("defer_reason", "only deferred operations carry a reason")),
        _ => {}
    }
    if matches!(plan.class, Some(OverlayClass::Deny(_) | OverlayClass::Defer))
        && (plan.scope.is_some() || plan.items.is_some() || plan.page_size.is_some()
            || !plan.exclude_from_hash.is_empty() || !plan.skip.is_empty()
            || plan.quota_group.is_some() || !plan.queryable.is_empty() || plan.tags.is_some()
            || !plan.fixed.is_empty() || plan.paginator.is_some() || plan.identity.is_some()
            || !plan.bind.is_empty() || plan.batch.is_some()) {
        return Err(invalid("class", "noncallable decision contains a plan"));
    }
    Ok(())
}

fn identity_rule(identity: &Identity) -> Result<(), NameError> {
    match identity {
        Identity::Key(paths) => {
            bounded(paths.len(), 1, 8, "identity.key")?;
            let unique = paths.iter().collect::<BTreeSet<_>>();
            if unique.len() != paths.len() { return Err(invalid("identity.key", "duplicate path")); }
            Ok(())
        }
        Identity::Template(template) => template_rule(template),
        Identity::Arn(_) => Ok(()),
        Identity::Choice(choice) => choice.validate(),
        Identity::CorrelatedKey(fields) => super::selection::validate_fields(fields),
        Identity::MapArn { key, resource_kind, request_url, .. } => {
            if key.is_empty() || key.len() > 4096 || key.chars().any(char::is_control)
                || matches!(resource_kind, super::ArnResourceKind::SqsQueue) != request_url.is_some() {
                Err(invalid("identity.map_arn", "invalid key or request correlation"))
            } else { Ok(()) }
        }
    }
}

fn template_rule(template: &str) -> Result<(), NameError> {
    let (open, seen) = template.char_indices().try_fold((None, false), |(open, seen), (index, character)| {
        match (open, character) {
            (None, '{') => Ok((Some(index.saturating_add(1)), seen)),
            (Some(start), '}') => {
                let field = template.get(start..index).ok_or_else(|| invalid("identity.template", template))?;
                if !matches!(field, "partition" | "region" | "account") {
                    super::MemberPath::selected(field)?;
                }
                Ok((None, true))
            }
            (Some(_), '{') | (None, '}') => Err(invalid("identity.template", template)),
            _ => Ok((open, seen)),
        }
    })?;
    if open.is_some() || !seen { return Err(invalid("identity.template", template)); }
    Ok(())
}

fn paginator_rule(paginator: &Paginator) -> Result<(), NameError> {
    let Paginator::Tokens { pairs, .. } = paginator else { return Ok(()); };
    bounded(pairs.len(), 1, 8, "paginator.pairs")?;
    let inputs = pairs.iter().map(|pair| &pair.input).collect::<BTreeSet<_>>();
    if inputs.len() != pairs.len() { return Err(invalid("paginator.pairs", "duplicate input")); }
    Ok(())
}

fn fixed_rule(value: &FixedValue) -> Result<(), NameError> {
    if super::fixed_input::bounded(value) { Ok(()) } else { Err(invalid("fixed", "literal bounds exceeded")) }
}

fn bounded(count: usize, min: usize, max: usize, key: &str) -> Result<(), NameError> {
    if !(min..=max).contains(&count) { return Err(invalid(key, &count.to_string())); }
    Ok(())
}

fn invalid(key: &str, value: &str) -> NameError {
    NameError { value: format!("{key}: {value}") }
}

fn binding_parent(binding: &Binding) -> Result<Option<&OperationName>, NameError> {
    let binding = match binding {
        Binding::Transformed { binding, transform: super::BindingTransform::StripPrefix(prefix) } => {
            if prefix.is_empty() || prefix.len() > 256 || prefix.chars().any(char::is_control)
                || !matches!(binding.as_ref(), Binding::From { .. } | Binding::FromArn { .. } | Binding::FromService { .. } | Binding::Context { .. }) {
                return Err(invalid("bind.transform", "invalid source or prefix"));
            }
            binding.as_ref()
        }
        binding => binding,
    };
    match binding {
        Binding::From { operation, .. } | Binding::FromArn { operation, .. } | Binding::FromUrl { operation, .. } | Binding::Context { operation, .. } => Ok(Some(operation)),
        Binding::FromService { service, .. } if service.is_empty() || service.len() > 64 => Err(invalid("bind.service", service)),
        Binding::FromService { .. } | Binding::Account | Binding::Region => Ok(None),
        Binding::Transformed { .. } => Err(invalid("bind.transform", "nested transform")),
    }
}
