use super::super::{empty, name, service};
pub(super) use crate::classes::DenyReason;
use crate::overlay::{
    Binding, FixedValue, Identity, ItemSelector, MemberPath, NameError, OperationOverlay, OverlayClass,
    Paginator, Scope, ServiceOverlay, TokenPair,
};

type NamedOperation = (&'static str, OperationOverlay);

pub(super) fn finish(
    service_name: &str,
    approved: Vec<NamedOperation>,
    deferred: &[&'static str],
    reason: &str,
    denied: &[(DenyReason, &[&'static str])],
) -> Result<ServiceOverlay, NameError> {
    let decisions = approved.into_iter()
        .chain(deferred.iter().map(|&operation| {
            let mut value = empty();
            value.class = Some(OverlayClass::Defer);
            value.defer_reason = Some(reason.to_owned());
            (operation, value)
        }))
        .chain(denied.iter().flat_map(|(reason, operations)| {
            operations.iter().map(|&operation| {
                let mut value = empty();
                value.class = Some(OverlayClass::Deny(*reason));
                (operation, value)
            })
        }))
        .collect();
    service(service_name, decisions)
}

pub(super) fn arn(member: &str) -> Result<Identity, NameError> {
    Ok(Identity::Arn(MemberPath::selected(member)?))
}

pub(super) fn key(member: &str) -> Result<Identity, NameError> {
    Ok(Identity::Key(vec![MemberPath::selected(member)?]))
}

pub(super) fn enumerate(
    items: &str,
    identity: Identity,
    token: Option<(&str, &str)>,
) -> Result<OperationOverlay, NameError> {
    let mut value = empty();
    value.class = Some(OverlayClass::Enumerate);
    value.scope = Some(Scope::Region);
    value.items = Some(ItemSelector::parse(items)?);
    value.identity = Some(identity);
    value.paginator = Some(match token {
        Some((input, output)) => Paginator::Token(TokenPair {
            input: MemberPath::new(input)?, output: MemberPath::new(output)?, optional: false,
        }),
        None => Paginator::None,
    });
    Ok(value)
}

pub(super) fn bind(
    mut value: OperationOverlay,
    input: &str,
    parent: &str,
    member: &str,
) -> Result<OperationOverlay, NameError> {
    value.class = Some(OverlayClass::Expand);
    value.bind.insert(MemberPath::new(input)?, Binding::From {
        operation: name(parent)?, path: MemberPath::selected(member)?,
    });
    Ok(value)
}

pub(super) fn fixed(
    mut value: OperationOverlay,
    input: &str,
    fixed_value: &str,
) -> Result<OperationOverlay, NameError> {
    value.fixed.insert(MemberPath::new(input)?, FixedValue::String(fixed_value.to_owned()));
    Ok(value)
}

pub(super) fn scope(mut value: OperationOverlay, scope: Scope) -> OperationOverlay {
    value.scope = Some(scope);
    value
}

pub(super) fn query(
    mut value: OperationOverlay,
    members: &[&str],
) -> Result<OperationOverlay, NameError> {
    value.queryable = members.iter().map(|member| MemberPath::selected(member)).collect::<Result<_, _>>()?;
    Ok(value)
}

pub(super) fn tags(mut value: OperationOverlay, member: &str) -> Result<OperationOverlay, NameError> {
    value.tags = Some(MemberPath::selected(member)?);
    Ok(value)
}
