use super::super::{empty, name, service};
use crate::overlay::MemberPath;
use crate::{
    classes::DenyReason,
    overlay::{
        Binding, ContextSource, FixedValue, Identity, ItemSelector, NameError, OperationOverlay,
        OverlayClass, Paginator, Scope, ServiceOverlay, TokenPair,
    },
};

pub(super) type Entry = (&'static str, OperationOverlay);

pub(super) fn inventory(
    operation: &'static str,
    items: &str,
    identity: Identity,
    paginator: Paginator,
) -> Result<Entry, NameError> {
    Ok((
        operation,
        OperationOverlay {
            class: Some(OverlayClass::Enumerate),
            scope: Some(Scope::Region),
            items: Some(ItemSelector::parse(items)?),
            identity: Some(identity),
            paginator: Some(paginator),
            ..empty()
        },
    ))
}

/// Single-item operation (Describe) with no pagination.
pub(super) fn describe(
    operation: &'static str,
    items: &str,
    identity: Identity,
) -> Result<Entry, NameError> {
    Ok((
        operation,
        OperationOverlay {
            class: Some(OverlayClass::Enumerate),
            scope: Some(Scope::Region),
            items: Some(ItemSelector::parse(items)?),
            identity: Some(identity),
            paginator: Some(Paginator::None),
            ..empty()
        },
    ))
}

pub(super) fn arn(member: &str) -> Result<Identity, NameError> {
    Ok(Identity::Arn(MemberPath::selected(member)?))
}

pub(super) fn key(members: &[&str]) -> Result<Identity, NameError> {
    members
        .iter()
        .map(|member| MemberPath::selected(member))
        .collect::<Result<Vec<_>, _>>()
        .map(Identity::Key)
}

pub(super) fn correlated(request: &[&str], selected: &[&str]) -> Result<Identity, NameError> {
    request
        .iter()
        .map(|member| MemberPath::new(member).map(crate::overlay::IdentityField::Request))
        .chain(selected.iter().map(|member| {
            MemberPath::selected(member).map(crate::overlay::IdentityField::Selected)
        }))
        .collect::<Result<Vec<_>, _>>()
        .map(Identity::CorrelatedKey)
}

pub(super) fn token(input: &str, output: &str) -> Result<Paginator, NameError> {
    Ok(Paginator::Token(TokenPair {
        input: MemberPath::new(input)?,
        output: MemberPath::new(output)?,
        optional: false,
    }))
}

pub(super) fn truncated(input: &str, output: &str, more: &str) -> Result<Paginator, NameError> {
    Ok(Paginator::Tokens {
        pairs: vec![TokenPair {
            input: MemberPath::new(input)?,
            output: MemberPath::new(output)?,
            optional: false,
        }],
        truncated: Some(MemberPath::new(more)?),
    })
}

pub(super) fn bound(mut entry: Entry, bindings: &[(&str, &str, &str)]) -> Result<Entry, NameError> {
    entry.1.class = Some(OverlayClass::Expand);
    entry.1.bind = bindings
        .iter()
        .map(|(input, operation, member)| {
            Ok((
                MemberPath::new(input)?,
                Binding::From {
                    operation: name(operation)?,
                    path: MemberPath::selected(member)?,
                },
            ))
        })
        .collect::<Result<_, NameError>>()?;
    Ok(entry)
}

pub(super) fn inherited(
    mut entry: Entry,
    input: &str,
    operation: &str,
    member: &str,
) -> Result<Entry, NameError> {
    entry.1.bind.insert(
        MemberPath::new(input)?,
        Binding::Context {
            operation: name(operation)?,
            source: ContextSource::Request,
            path: MemberPath::new(member)?,
        },
    );
    Ok(entry)
}

pub(super) fn fixed_integer(mut entry: Entry, input: &str, value: i64) -> Result<Entry, NameError> {
    entry
        .1
        .fixed
        .extend([(MemberPath::new(input)?, FixedValue::Integer(value))]);
    Ok(entry)
}

pub(super) fn fixed_boolean(
    mut entry: Entry,
    input: &str,
    value: bool,
) -> Result<Entry, NameError> {
    entry
        .1
        .fixed
        .extend([(MemberPath::new(input)?, FixedValue::Boolean(value))]);
    Ok(entry)
}

/// Omit migration settings (large, may contain customer data) via WithoutSettings=true.
pub(super) fn without_settings(entry: Entry) -> Result<Entry, NameError> {
    fixed_boolean(entry, "WithoutSettings", true)
}

pub(super) fn tagged(mut entry: Entry, member: &str) -> Result<Entry, NameError> {
    entry.1.tags = Some(MemberPath::selected(member)?);
    Ok(entry)
}

pub(super) fn queried(mut entry: Entry, members: &[&str]) -> Result<Entry, NameError> {
    entry.1.queryable = members
        .iter()
        .map(|member| MemberPath::selected(member))
        .collect::<Result<_, _>>()?;
    Ok(entry)
}

pub(super) fn scoped(mut entry: Entry, scope: Scope) -> Entry {
    entry.1.scope = Some(scope);
    entry
}

pub(super) fn deferred(operation: &'static str, reason: &str) -> Entry {
    (
        operation,
        OperationOverlay {
            class: Some(OverlayClass::Defer),
            defer_reason: Some(reason.to_owned()),
            ..empty()
        },
    )
}

pub(super) fn denied(operation: &'static str, reason: DenyReason) -> Entry {
    (
        operation,
        OperationOverlay {
            class: Some(OverlayClass::Deny(reason)),
            ..empty()
        },
    )
}

pub(super) fn reviewed(
    service_name: &str,
    callable: Vec<Entry>,
    dispositions: &[&[(&'static str, &'static str)]],
    denials: &[(&'static str, DenyReason)],
) -> Result<ServiceOverlay, NameError> {
    service(
        service_name,
        callable
            .into_iter()
            .chain(
                dispositions
                    .iter()
                    .flat_map(|chunk| chunk.iter())
                    .map(|(operation, reason)| deferred(operation, reason)),
            )
            .chain(
                denials
                    .iter()
                    .map(|(operation, reason)| denied(operation, *reason)),
            )
            .collect(),
    )
}
